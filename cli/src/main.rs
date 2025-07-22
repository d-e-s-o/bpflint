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

use bpflint::LintMatch;
use bpflint::Point;
use bpflint::Range;
use std::path::Path;

fn has_bpf_c_ext(path: &Path) -> bool {
    if let Some(file_name) = path.file_name() {
        if file_name
            .to_str()
            .map(|s| s.ends_with(".bpf.c"))
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

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

    let m_ext_is_c = LintMatch {
        lint_name: "bogus-file-extension".to_string(),
        message: "by convention BPF C code should use the file extension '.bpf.c'".to_string(),
        range: Range {
            bytes: 0..0,
            start_point: Point { row: 0, col: 0 },
            end_point: Point { row: 0, col: 0 },
        },
    };

    if print_lints {
        for lint in builtin_lints() {
            writeln!(&mut stdout, "{}", lint.name)?;
        }
    } else {
        for src_path in srcs.into_iter().flatten() {
            let code = read(&src_path)
                .with_context(|| format!("failed to read `{}`", src_path.display()))?;

            if !has_bpf_c_ext(&src_path) {
                let () = report_terminal(&m_ext_is_c, &code, &src_path, &mut stdout)?;
            }

            let matches =
                lint(&code).with_context(|| format!("failed to lint `{}`", src_path.display()))?;
            for m in matches {
                let () = report_terminal(&m, &code, &src_path, &mut stdout)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    /// tests whether `has_bpf_c_ext` correctly returns true
    /// for *.bpf.c files else false
    #[test]
    fn test_has_bpf_c_ext() {
        assert!(has_bpf_c_ext(Path::new("file.bpf.c")));
        assert!(has_bpf_c_ext(Path::new("/path/to/file.bpf.c")));
        assert!(has_bpf_c_ext(Path::new("C:\\Windows\\file.bpf.c")));

        assert!(!has_bpf_c_ext(Path::new("file.c")));
        assert!(!has_bpf_c_ext(Path::new("file.bpf.h")));
        assert!(!has_bpf_c_ext(Path::new("filebpfc")));
    }
}
