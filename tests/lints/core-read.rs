//! Tests for the `core-read` lint.

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
            struct task_struct *next = (struct task_struct *)ctx[2];
            int prev_pid = 0;
            bpf_core_read(&prev_pid, sizeof(prev_pid), &prev->pid);
            int next_prev_pid = BPF_CORE_READ(next, pid);
            return 0;
        }
    "# };

    let expected = indoc! { r#"
        warning: [core-read] bpf_core_read() and BPF_CORE_READ() have been subsumed by bpf_core_cast() -- consider using it instead; refer to https://docs.ebpf.io/ebpf-library/libbpf/ebpf/bpf_core_cast/
          --> <stdin>:6:4
          | 
        6 |     bpf_core_read(&prev_pid, sizeof(prev_pid), &prev->pid);
          |     ^^^^^^^^^^^^^
          | 
        warning: [core-read] bpf_core_read() and BPF_CORE_READ() have been subsumed by bpf_core_cast() -- consider using it instead; refer to https://docs.ebpf.io/ebpf-library/libbpf/ebpf/bpf_core_cast/
          --> <stdin>:7:24
          | 
        7 |     int next_prev_pid = BPF_CORE_READ(next, pid);
          |                         ^^^^^^^^^^^^^
          | 
    "# };

    let user_kernel_version = None;

    assert_eq!(lint_report(code, user_kernel_version), expected);
}

// Test where the user kernel is greater than or equal to lint kernel version
#[test]
fn run_kernel_in_scope() {
    let code = indoc! { r#"
        SEC("tp_btf/sched_switch")
        int handle__sched_switch(u64 *ctx)
        {
            struct task_struct *prev = (struct task_struct *)ctx[1];
            struct task_struct *next = (struct task_struct *)ctx[2];
            int prev_pid = 0;
            bpf_core_read(&prev_pid, sizeof(prev_pid), &prev->pid);
            int next_prev_pid = BPF_CORE_READ(next, pid);
            return 0;
        }
    "# };

    let expected = indoc! { r#"
        warning: [core-read] bpf_core_read() and BPF_CORE_READ() have been subsumed by bpf_core_cast() -- consider using it instead; refer to https://docs.ebpf.io/ebpf-library/libbpf/ebpf/bpf_core_cast/
          --> <stdin>:6:4
          | 
        6 |     bpf_core_read(&prev_pid, sizeof(prev_pid), &prev->pid);
          |     ^^^^^^^^^^^^^
          | 
        warning: [core-read] bpf_core_read() and BPF_CORE_READ() have been subsumed by bpf_core_cast() -- consider using it instead; refer to https://docs.ebpf.io/ebpf-library/libbpf/ebpf/bpf_core_cast/
          --> <stdin>:7:24
          | 
        7 |     int next_prev_pid = BPF_CORE_READ(next, pid);
          |                         ^^^^^^^^^^^^^
          | 
    "# };

    let user_kernel_version = Some(parse_kernel_version("5.2.0").unwrap());

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
            struct task_struct *next = (struct task_struct *)ctx[2];
            int prev_pid = 0;
            bpf_core_read(&prev_pid, sizeof(prev_pid), &prev->pid);
            int next_prev_pid = BPF_CORE_READ(next, pid);
            return 0;
        }
    "# };

    let user_kernel_version = Some(parse_kernel_version("4.0.0").unwrap());

    assert_eq!(lint_report(code, user_kernel_version), "");
}
