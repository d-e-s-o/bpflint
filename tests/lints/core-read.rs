//! Tests for the `core-read` lint.

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
            struct task_struct *next = (struct task_struct *)ctx[2];
            int prev_pid = 0;
            bpf_core_read(&prev_pid, sizeof(prev_pid), &prev->pid);
            int next_prev_pid = BPF_CORE_READ(next, pid);
            return 0;
        }
    "# };

    let expected = indoc! { r#"
        warning: [core-read] bpf_core_read() and BPF_CORE_READ() are deprecated and replaced by bpf_core_cast(); refer to https://docs.ebpf.io/ebpf-library/libbpf/ebpf/bpf_core_cast/
          --> <stdin>:6:4
          | 
        6 |     bpf_core_read(&prev_pid, sizeof(prev_pid), &prev->pid);
          |     ^^^^^^^^^^^^^
          | 
        warning: [core-read] bpf_core_read() and BPF_CORE_READ() are deprecated and replaced by bpf_core_cast(); refer to https://docs.ebpf.io/ebpf-library/libbpf/ebpf/bpf_core_cast/
          --> <stdin>:7:24
          | 
        7 |     int next_prev_pid = BPF_CORE_READ(next, pid);
          |                         ^^^^^^^^^^^^^
          | 
    "# };
    assert_eq!(lint_report(code), expected);
}
