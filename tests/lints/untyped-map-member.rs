//! Tests for the `untyped-map-member` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use crate::util::lint_report;


#[test]
fn basic_sizeof() {
    let code = indoc! { r#"
        struct {
            int a;
            __uint(key_size, sizeof(b));
        } name;
    "# };

    let expected = indoc! { r#"
        warning: [untyped-map-member] __uint(<a>_size, sizeof(<b>)) does not contain potentially relevant type information, consider using __type(<a>, <b>) instead
          --> <stdin>:2:4
          | 
        2 |     __uint(key_size, sizeof(b));
          |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
          | 
    "# };
    assert_eq!(lint_report(code), expected);
}
