//! Tests for the `pragma-unroll` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use crate::util::lint_report;

#[test]
fn basic_for_bounded() {
    let code = indoc! { r#"
        #include <linux/bpf.h>
        #include <bpf/bpf_helpers.h>
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
        warning: [pragma-unroll-for-loop-bounded] Consider using bpf_for instead as it is generally considered the superior loop primitive (refer to https://docs.ebpf.io/linux/concepts/loops/ for details and exceptions)
          --> <stdin>:6:4
          | 
        6 |  /     #pragma unroll
        7 |  |     for (int i = 0; i < 10; i++) {
          |  |^
          | 
    "# };
    assert_eq!(lint_report(code), expected);
}

#[test]
fn basic_for_unbounded() {
    let code = indoc! { r#"
        #include <linux/bpf.h>
        #include <bpf/bpf_helpers.h>
        SEC("xdp")
        int xdp_prog(struct xdp_md *ctx)
        {
            __u32 sum = 0;
            #pragma unroll
            for (;;) {
                sum += i * 2;
            }
            return XDP_PASS;
        }
    "# };

    // No match
    let expected = indoc! { r#""# };
    assert_eq!(lint_report(code), expected);
}

#[test]
fn basic_while_bounded() {
    let code = indoc! { r#"
        #include <linux/bpf.h>
        #include <bpf/bpf_helpers.h>
        SEC("xdp")
        int xdp_prog(struct xdp_md *ctx)
        {
            __u32 sum = 0;
            #pragma unroll
            while (sum < 10) {
                sum += 1;
            }
            return XDP_PASS;
        }
    "# };

    // Nothing matches
    let expected = indoc! { r#""# };
    assert_eq!(lint_report(code), expected);
}

#[test]
fn basic_while_unbounded() {
    let code = indoc! { r#"
        #include <linux/bpf.h>
        #include <bpf/bpf_helpers.h>
        SEC("xdp")
        int xdp_prog(struct xdp_md *ctx)
        {
            __u32 sum = 0;
            #pragma unroll
            while (true) {
                sum += 1;
            }
            return XDP_PASS;
        }
    "# };

    // Nothing matches
    let expected = indoc! { r#""# };
    assert_eq!(lint_report(code), expected);
}
