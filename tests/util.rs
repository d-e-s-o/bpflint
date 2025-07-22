//! Helpers for testing the linting functionality.

use std::path::Path;

use bpflint::lint;
use bpflint::report_terminal;


/// Lint `code` and report matches as a string created using
/// [`report_terminal`].
pub fn lint_report<C>(code: C) -> String
where
    C: AsRef<[u8]>,
{
    let mut report = Vec::new();
    let () = lint(code.as_ref())
        .unwrap()
        .into_iter()
        .try_for_each(|m| report_terminal(&m, code.as_ref(), Path::new("<stdin>"), &mut report))
        .unwrap();
    let report = String::from_utf8(report).unwrap();
    report
}
