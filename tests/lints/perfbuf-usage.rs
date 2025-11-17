//! Tests for the `perfbuf-usage` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use bpflint::parse_kernel_version;
use crate::util::lint_report;


#[test]
fn basic() {
    let code = indoc! { r#"
        struct {
          int a;
          __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
        } name;
    "# };

    let expected = indoc! { r#"
        warning: [perfbuf-usage] Consider using a ringbuf over perfbuf as it is generally considered the superior data exchange primitive (refer to https://nakryiko.com/posts/bpf-ringbuf/ for details and exceptions)
          --> <stdin>:2:2
          | 
        2 |   __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
          |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
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
          __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
        } name;
    "# };

    let expected = indoc! { r#"
        warning: [perfbuf-usage] Consider using a ringbuf over perfbuf as it is generally considered the superior data exchange primitive (refer to https://nakryiko.com/posts/bpf-ringbuf/ for details and exceptions)
          --> <stdin>:2:2
          | 
        2 |   __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
          |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
          | 
    "# };

    let user_kernel_version = Some(parse_kernel_version("4.8.5").unwrap());

    assert_eq!(lint_report(code, user_kernel_version), expected);
}


// Test where the user kernel is less than lint kernel version
#[test]
fn no_run_kernel_out_of_scope() {
    let code = indoc! { r#"
        struct {
          int a;
          __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
        } name;
    "# };

    let user_kernel_version = Some(parse_kernel_version("3.2.7").unwrap());

    assert_eq!(lint_report(code, user_kernel_version), "");
}
