//! Tests for the `perfbuf-usage` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use crate::util::lint_report;


#[test]
fn perfbuf_usage() {
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
    assert_eq!(lint_report(code), expected);
}
