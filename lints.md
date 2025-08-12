# 1
🔍 Lint: Unsafe or Unsupported Pointer Dereference in BPF Programs
❗ Problem:
Due to eBPF verifier restrictions, dereferencing arbitrary pointers (especially user space memory or kernel structures) without proper bounds checks or helper usage leads to program rejection by the verifier or, worse, silent logic errors.

✅ Example of good usage:
c
Copy
Edit
struct task_struct *task = (struct task_struct *)bpf_get_current_task();
bpf_probe_read_kernel(&pid, sizeof(pid), &task->pid);
❌ Example of problematic usage:
c
Copy
Edit
struct task_struct *task = (struct task_struct *)bpf_get_current_task();
int pid = task->pid; // direct dereference - unsafe!

Suggested lint rule:
"Direct pointer dereference outside helper calls or bpf_probe_read*-like wrappers is disallowed."

🔧 Lint message:
⚠️ Avoid direct dereferencing of pointers in BPF programs. Use bpf_probe_read_kernel, bpf_probe_read_user, or similar helpers to access memory safely.

🚀 Why this matters:
Helps developers pass the BPF verifier more reliably.

Makes code portable across kernel versions by avoiding assumptions about memory layout or access safety.

Formalizes one of the most common stumbling blocks for new BPF developers.


# 2
Suggested lint rule:
"Always check return values from BPF helper functions that can fail or return NULL."

🔧 Lint message:
⚠️ Return value from bpf_map_lookup_elem (or similar helper) is unchecked. Verifier requires null check before dereference.

Why this matters:
Prevents verifier rejections due to unguarded pointer dereference.

Improves code robustness and debuggability.

Encourages best practices and readability for complex eBPF programs.


# 3
🔍 Lint: Non-Canonical or Inconsistent SEC() Section Annotations
❗ Problem:
SEC annotations define the type and attachment point of eBPF programs, but:

They are free-form strings, so typos or inconsistencies (e.g., kprobe/ vs kret_probe/) are easy to introduce.

Inconsistent naming hinders readability and portability.

Certain tools (like bpftool, libbpf, or CO-RE loaders) expect canonical forms.

✅ Example of good usage:
c
Copy
Edit
SEC("kprobe/sys_execve")
int bpf_prog(void *ctx) {
    // ...
}
❌ Examples of problematic usage:
c
Copy
Edit
SEC("kret_probe/sys_execve")   // wrong form, should be "kretprobe/"
SEC("KPROBE/sys_execve")       // uppercase convention violation
SEC("tracepoint/sched:sched_switch ")  // trailing space
🎯 Suggested lint rule:
"SEC() section names should follow canonical lowercase, slash-delimited, no-extra-whitespace conventions for known types."

🔧 Lint message:
⚠️ Non-canonical section name "kret_probe/sys_execve" — expected "kretprobe/sys_execve".
