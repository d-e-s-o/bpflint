//! A linter for BPF C code.

mod args;

use std::env::var_os;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fs::read;
use std::io;
use std::io::Write as _;
use std::io::stderr;
use std::ops::Not as _;
use std::path::Path;
use std::process::ExitCode;
use std::process::Termination;

use anyhow::Context as _;
use anyhow::Error;
use anyhow::Result;

use clap::Parser as _;

use tracing::Level;
use tracing::subscriber::set_global_default as set_global_subscriber;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::time::ChronoLocal;

use bpflint::LintMatch;
use bpflint::Point;
use bpflint::Range;
use bpflint::builtin_lints;
use bpflint::lint;
use bpflint::terminal;


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


enum ExitError {
    Anyhow(Error),
    ExitCode(ExitCode),
}

impl Debug for ExitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Anyhow(error) => Debug::fmt(error, f),
            Self::ExitCode(..) => Ok(()),
        }
    }
}

impl<E> From<E> for ExitError
where
    Error: From<E>,
{
    fn from(error: E) -> Self {
        Self::Anyhow(Error::from(error))
    }
}


fn main_impl() -> Result<(), ExitError> {
    let args = args::Args::parse();
    let args::Args {
        srcs,
        print_lints,
        verbosity,
        ..
    } = &args;

    let additional_opts = args.additional_options();

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

    if *print_lints {
        for lint in builtin_lints() {
            writeln!(&mut stdout, "{}", lint.name)?;
        }
        Ok(())
    } else {
        let mut result = Ok(());
        for src_path in srcs.iter().flatten() {
            let code = read(src_path)
                .with_context(|| format!("failed to read `{}`", src_path.display()))?;

            let mut first = true;
            let match_ext = has_bpf_c_ext(src_path).not().then_some(&m_ext_is_c);
            let matches =
                lint(&code).with_context(|| format!("failed to lint `{}`", src_path.display()))?;
            for m in match_ext.into_iter().chain(matches.iter()) {
                if !first {
                    writeln!(&mut stdout)?;
                } else {
                    first = false;
                }

                let () = terminal::report_opts(m, &code, src_path, &additional_opts, &mut stdout)?;
                if result.is_ok() {
                    result = Err(ExitError::ExitCode(ExitCode::FAILURE));
                }
            }
        }
        result
    }
}


#[derive(Debug)]
enum ExitResult {
    Ok(()),
    Err(ExitError),
}

impl Termination for ExitResult {
    fn report(self) -> ExitCode {
        match self {
            ExitResult::Ok(val) => val.report(),
            ExitResult::Err(err) => match err {
                ExitError::Anyhow(error) => {
                    let _result = writeln!(stderr(), "{error:?}");
                    ExitCode::FAILURE
                },
                ExitError::ExitCode(exit_code) => exit_code,
            },
        }
    }
}


fn main() -> ExitResult {
    match main_impl() {
        Ok(()) => ExitResult::Ok(()),
        Err(err) => ExitResult::Err(err),
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Test that [`has_bpf_c_ext`] works correctly for various
    /// paths/extensions.
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
