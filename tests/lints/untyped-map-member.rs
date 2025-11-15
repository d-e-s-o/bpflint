//! Tests for the `untyped-map-member` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use bpflint::parse_kernel_version;
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
    assert_eq!(lint_report(code, None), expected);
}

// Test where the user kernel is greater than or equal to user specified kernel
#[test]
fn run_kernel_in_scope() {
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

    let user_kernel_version = Some(parse_kernel_version("4.18.0").unwrap());

    assert_eq!(lint_report(code, user_kernel_version), expected);
}

// Test where the user kernel is less than lint kernel version
#[test]
fn no_run_kernel_out_of_scope() {
    let code = indoc! { r#"
        struct {
            int a;
            __uint(key_size, sizeof(b));
        } name;
    "# };

    let user_kernel_version = Some(parse_kernel_version("3.56.1").unwrap());

    assert_eq!(lint_report(code, user_kernel_version), "");
}
