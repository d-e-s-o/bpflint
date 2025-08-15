(call_expression
    function: (identifier) @function (#eq? @function "bpf_get_current_task")
    arguments: (argument_list) @__args (#match? @__args "^\\(\\s*\\)$")
    (#set! "message" "bpf_get_current_task() is difficult to use; consider using the stricter typed bpf_get_current_task_btf() instead; refer to bpf-helpers(7)")
)
