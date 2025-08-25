//! Tests for the `unstable-attach-point` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use crate::util::lint_report;


#[test]
fn basic() {
    let code = indoc! { r#"
        SEC("fentry/do_nanosleep")
        int nanosleep(void *ctx) {
        }
    "# };

    let expected = indoc! { r#"
        warning: [unstable-attach-point] kprobe/kretprobe/fentry/fexit are conceptually unstable and prone to changes between kernel versions; consider more stable attach points such as tracepoints or LSM hooks, if available
          --> <stdin>:0:4
          | 
        0 | SEC("fentry/do_nanosleep")
          |     ^^^^^^^^^^^^^^^^^^^^^
          | 
    "# };
    assert_eq!(lint_report(code), expected);
}


#[test]
fn basic2() {
    let code = indoc! { r#"
        SEC("kprobe/cap_capable")

        int BPF_KPROBE(kprobe__foobar, const struct cred *cred,
                       struct user_namespace *targ_ns, int cap, int cap_opt) {
    "# };

    let expected = indoc! { r#"
        warning: [unstable-attach-point] kprobe/kretprobe/fentry/fexit are conceptually unstable and prone to changes between kernel versions; consider more stable attach points such as tracepoints or LSM hooks, if available
          --> <stdin>:0:4
          | 
        0 | SEC("kprobe/cap_capable")
          |     ^^^^^^^^^^^^^^^^^^^^
          | 
    "# };
    assert_eq!(lint_report(code), expected);
}
