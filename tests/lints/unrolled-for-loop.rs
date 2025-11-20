//! Tests for the `unrolled-for-loop` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use crate::util::lint_report;


/// Check basic functioning of the `unrolled-for-loop` lint.
#[test]
fn basic_for_bounded() {
    let code = indoc! { r#"
        SEC("xdp")
        int xdp_prog(struct xdp_md *ctx)
        {
            __u32 sum = 0;
            #pragma unroll
            for (int i = 0; i < 10; i++) {
                sum += i * 2;
            }
            return XDP_PASS;
        }
    "# };

    let expected = indoc! { r#"
        warning: [unrolled-for-loop] Consider using bpf_for instead as it is generally considered the superior loop primitive (refer to https://docs.ebpf.io/linux/concepts/loops/ for details and exceptions)
          --> <stdin>:4:4
          | 
        4 |  /     #pragma unroll
        5 |  |     for (int i = 0; i < 10; i++) {
          |  |^
          | 
    "# };
    assert_eq!(lint_report(code), expected);
}


/// Make sure that we do not flag unbounded `for` and `while` loops.
#[test]
fn unbounded() {
    let code = indoc! { r#"
        SEC("xdp")
        int xdp_prog(struct xdp_md *ctx)
        {
            __u32 sum = 0;
            #pragma unroll
            for (;;) {
                sum += i * 2;
            }

            #pragma unroll
            while (true) {
                sum += 1;
            }
            return XDP_PASS;
        }
    "# };

    // No match
    let expected = indoc! { r#""# };
    assert_eq!(lint_report(code), expected);
}
