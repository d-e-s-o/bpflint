//! A linter for BPF C code.

mod args;

use std::env::var_os;
use std::fs::read;
use std::io;
use std::io::Write as _;

use anyhow::Context as _;
use anyhow::Result;

use clap::Parser as _;

use tracing::Level;
use tracing::subscriber::set_global_default as set_global_subscriber;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::time::ChronoLocal;

use bpflint::builtin_lints;
use bpflint::lint;
use bpflint::report_terminal;


fn main() -> Result<()> {
    let args::Args {
        srcs,
        print_lints,
        verbosity,
    } = args::Args::parse();

    let level = match verbosity {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let builder = FmtSubscriber::builder()
        .with_timer(ChronoLocal::new("%Y-%m-%dT%H:%M:%S%.3f%:z".to_string()));

    if let Some(directive) = var_os(EnvFilter::DEFAULT_ENV) {
        let directive = directive
            .to_str()
            .with_context(|| format!("env var `{}` is not valid UTF-8", EnvFilter::DEFAULT_ENV))?;

        let subscriber = builder.with_env_filter(EnvFilter::new(directive)).finish();
        let () = set_global_subscriber(subscriber)
            .with_context(|| "failed to set tracing subscriber")?;
    } else {
        let subscriber = builder.with_max_level(level).finish();
        let () = set_global_subscriber(subscriber)
            .with_context(|| "failed to set tracing subscriber")?;
    };

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if print_lints {
        for lint in builtin_lints() {
            write!(&mut stdout, "{}", lint.name)?;
        }
    } else {
        for src_path in srcs {
            let code = read(&src_path)
                .with_context(|| format!("failed to read `{}`", src_path.display()))?;
            let matches =
                lint(&code).with_context(|| format!("failed to lint `{}`", src_path.display()))?;
            for m in matches {
                let () = report_terminal(&m, &code, &src_path, &mut stdout)?;
            }
        }
    }
    Ok(())
}
