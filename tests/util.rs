//! Helpers for testing the linting functionality.

use std::path::Path;

use bpflint::LintOpts;
use bpflint::lint;
use bpflint::terminal::report;


/// Lint `code` and report matches as a string created using
/// [`terminal::report`].
pub fn lint_report<C>(code: C) -> String
where
    C: AsRef<[u8]>,
{
    lint_report_opts(code, &LintOpts::default())
}

/// Lint `code` and report matches as a string created using
/// [`report_terminal`] with custom lint options.
pub fn lint_report_opts<C>(code: C, lint_opts: &LintOpts) -> String
where
    C: AsRef<[u8]>,
{
    let mut r = Vec::new();
    let () = lint(code.as_ref(), lint_opts)
        .unwrap()
        .into_iter()
        .try_for_each(|m| report(&m, code.as_ref(), Path::new("<stdin>"), &mut r))
        .unwrap();
    let r = String::from_utf8(r).unwrap();
    r
}
