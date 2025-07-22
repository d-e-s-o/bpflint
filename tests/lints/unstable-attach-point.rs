//! Tests for the `unstable-attach-point` lint.

use pretty_assertions::assert_eq;

use crate::util::lint_report;


#[test]
fn basic() {
    let code = r#"
SEC("fentry/do_nanosleep")
int nanosleep(void *ctx) {
}
"#;

    let expected = r#"warning: [unstable-attach-point] kprobe/kretprobe/fentry/fexit are conceptually unstable and prone to changes between kernel versions; consider more stable attach points such as tracepoints or LSM hooks, if available
  --> <stdin>:1:4
  | 
1 | SEC("fentry/do_nanosleep")
  |     ^^^^^^^^^^^^^^^^^^^^^
  | 
"#;
    assert_eq!(lint_report(code), expected);
}


#[test]
fn basic2() {
    let code = r#"
SEC("kprobe/cap_capable")

int BPF_KPROBE(kprobe__foobar, const struct cred *cred,
               struct user_namespace *targ_ns, int cap, int cap_opt) {
"#;

    let expected = r#"warning: [unstable-attach-point] kprobe/kretprobe/fentry/fexit are conceptually unstable and prone to changes between kernel versions; consider more stable attach points such as tracepoints or LSM hooks, if available
  --> <stdin>:1:4
  | 
1 | SEC("kprobe/cap_capable")
  |     ^^^^^^^^^^^^^^^^^^^^
  | 
"#;
    assert_eq!(lint_report(code), expected);
}
