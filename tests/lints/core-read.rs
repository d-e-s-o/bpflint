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
        // TP_PROTO(bool preempt, struct task_struct *prev, struct task_struct *next, struct rq_flags *rf)
        struct task_struct *prev = (struct task_struct *)ctx[1];
        struct task_struct *next = (struct task_struct *)ctx[2];
        s32 cpu = bpf_get_smp_processor_id();
        u64 now = bpf_ktime_get_ns();
        struct running_task *t;
        if (!(t = bpf_map_lookup_elem(&running_tasks, &cpu)))
                return 0;
        if (t->running_at && prev->pid) {
                s64 dur = now - t->running_at;
                if (dur > runtime_thresh_ns) {
                        struct event event = {0};
                        bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
                        bpf_probe_read(event.bt, sizeof(t->bt), t->bt);
                        event.pid = prev->pid;
                        event.duration = dur;
                        event.bt_sample_cnt = t->bt_sample_cnt;
                        bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU, &event, sizeof(event));
                }
        }
        t->running_at = 0;
        t->bt_at = 0;
        if (kthread_only && !(next->flags & PF_KTHREAD))
                return 0;
        if (percpu_only && next->nr_cpus_allowed != 1)
                return 0;
        t->running_at = now;
        t->bt_at = now;
        t->bt_sample_cnt = 0;
        return 0;
    }
    "# };

    let expected = indoc! { r#"
    warning: [probe-read] bpf_probe_read() is deprecated and replaced by bpf_probe_user() and bpf_probe_kernel(); refer to bpf-helpers(7)
      --> <stdin>:15:20
       | 
    15 |                     bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
       |                     ^^^^^^^^^^^^^^
       | 
    warning: [probe-read] bpf_probe_read() is deprecated and replaced by bpf_probe_user() and bpf_probe_kernel(); refer to bpf-helpers(7)
      --> <stdin>:16:20
       | 
    16 |                     bpf_probe_read(event.bt, sizeof(t->bt), t->bt);
       |                     ^^^^^^^^^^^^^^
       | 
   "# };
    assert_eq!(lint_report(code), expected);
}
