(call_expression
    function: (identifier) @function (#eq? @function "bpf_get_current_task")
    arguments: (argument_list) @__args (#match? @__args "^\\(\\s*\\)$")
)
