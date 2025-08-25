//! Tests for the `get-current-task` lint.

use indoc::indoc;

use pretty_assertions::assert_eq;

use crate::util::lint_report;


/// Check basic functioning of the `get-current-task` lint.
#[test]
fn basic() {
    let code = indoc! { r#"
        SEC("tp_btf/irq_handler_entry")
        int on_irq_handler_entry(u64 *cxt)
        {
          struct task_struct *task;

          task = (struct task_struct *)bpf_get_current_task();
          if (!task)
            return 0;

          return 1;
        }
    "# };

    let expected = indoc! { r#"
        warning: [get-current-task] bpf_get_current_task() is difficult to use; consider using the stricter typed bpf_get_current_task_btf() instead; refer to bpf-helpers(7)
          --> <stdin>:5:31
          | 
        5 |   task = (struct task_struct *)bpf_get_current_task();
          |                                ^^^^^^^^^^^^^^^^^^^^
          | 
    "# };
    assert_eq!(lint_report(code), expected);
}


/// Make sure that we would match a call with empty parentheses, but
/// some white spaces.
#[test]
fn whitespace_call() {
    let code = indoc! { r#"
        bpf_get_current_task(  );
    "# };
    let expected = indoc! { r#"
        warning: [get-current-task] bpf_get_current_task() is difficult to use; consider using the stricter typed bpf_get_current_task_btf() instead; refer to bpf-helpers(7)
          --> <stdin>:0:0
          | 
        0 | bpf_get_current_task(  );
          | ^^^^^^^^^^^^^^^^^^^^
          | 
    "# };
    assert_eq!(lint_report(code), expected);
}


/// Make sure that we don't match a function with the same name but a
/// different signature.
#[test]
fn no_match_different_signature() {
    let code = indoc! { r#"
        task = (struct task_struct *)bpf_get_current_task("foobar");
    "# };
    assert_eq!(lint_report(code), "");

    // TODO: This construct should actually be accepted. Sigh.
    let code = indoc! { r#"
        task = (struct task_struct *)bpf_get_current_task(/* WRONG */);
    "# };
    assert_eq!(lint_report(code), "");
}
