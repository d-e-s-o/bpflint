//! Tests for the `perf-buff-map` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use crate::util::lint_report;


#[test]
fn perf_map_usage() {
    let code = indoc! { r#"
      struct {
        int a;
        __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
      } name;
    "# };

    let expected = indoc! { r#"
      warning: [perf-buff-map] Using ring buffers is preferred over perf buffers
        --> <stdin>:2:2
        | 
      2 |   __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
        |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        | 
    "# };
    assert_eq!(lint_report(code), expected);
}
