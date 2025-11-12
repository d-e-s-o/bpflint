//! Tests for the `bpf-loop` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use crate::util::lint_report;

#[test]
fn basic_for_bounded() {
    let code = indoc! { r#"
        #include <linux/bpf.h>
        #include <bpf/bpf_helpers.h>

        __u64 iterations_inner = 10000;

        static int recurse_loop(__u64 idx, void *ctx)
        {
            return 0;
        }

        SEC("xdp")
        int xdp_prog(struct xdp_md *ctx)
        {
            bpf_loop(iterations_inner, recurse_loop, NULL, 0);
            return XDP_PASS;
        }
    "# };

    let expected = indoc! { r#"
        warning: [bpf-loop] Consider using bpf_for instead as it is generally considered the superior loop primitive (refer to https://docs.ebpf.io/linux/concepts/loops/ for details and exceptions)
          --> <stdin>:13:4
           | 
        13 |     bpf_loop(iterations_inner, recurse_loop, NULL, 0);
           |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
           | 
    "# };
    assert_eq!(lint_report(code), expected);
}
