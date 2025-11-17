//! Tests for the `probe-read` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use bpflint::parse_kernel_version;
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
    assert_eq!(lint_report(code, None), expected);
}


/// Make sure that we don't match a function with the same name but a
/// different signature.
#[test]
fn no_match_different_signature() {
    let code = indoc! { r#"
        bpf_probe_read("foobar");
    "# };
    assert_eq!(lint_report(code, None), "");
}

// Test where the user kernel is greater than or equal to user specified kernel
#[test]
fn run_kernel_in_scope() {
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

    let user_kernel_version = Some(parse_kernel_version("5.7.3").unwrap());

    assert_eq!(lint_report(code, user_kernel_version), expected);
}

// Test where the user kernel is less than lint kernel version
#[test]
fn no_run_kernel_out_of_scope() {
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

    let user_kernel_version = Some(parse_kernel_version("4.10.8").unwrap());

    assert_eq!(lint_report(code, user_kernel_version), "");
}
