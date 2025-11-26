//! Helpers for testing the linting functionality.

use bpflint::lint;
use bpflint::terminal::report;


/// Lint `code` and report matches as a string created using
/// [`terminal::report`].
pub fn lint_report<C>(code: C) -> String
where
    C: AsRef<[u8]>,
{
    let mut r = Vec::new();
    let () = lint(code.as_ref())
        .unwrap()
        .into_iter()
        .try_for_each(|m| report(&m, code.as_ref(), "<stdin>", &mut r))
        .unwrap();
    let r = String::from_utf8(r).unwrap();
    r
}
