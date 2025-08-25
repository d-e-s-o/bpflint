//! Tests for the `probe-read` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use crate::util::lint_report;


#[test]
fn basic() {
    let code = indoc! { r#"
        SEC("tp_btf/sched_switch")
        int handle__sched_switch(u64 *ctx)
        {
            struct task_struct *prev = (struct task_struct *)ctx[1];
            struct event event = {0};
            bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
            return 0;
        }
    "# };

    let expected = indoc! { r#"
        warning: [probe-read] bpf_probe_read() is deprecated and replaced by bpf_probe_user() and bpf_probe_kernel(); refer to bpf-helpers(7)
          --> <stdin>:5:4
          | 
        5 |     bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
          |     ^^^^^^^^^^^^^^
          | 
    "# };
    assert_eq!(lint_report(code), expected);
}


/// Make sure that we don't match a function with the same name but a
/// different signature.
#[test]
fn no_match_different_signature() {
    let code = indoc! { r#"
        bpf_probe_read("foobar");
    "# };
    assert_eq!(lint_report(code), "");
}
