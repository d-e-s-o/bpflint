(call_expression
    function: (identifier) @function (#eq? @function "bpf_get_current_task")
    arguments: (argument_list) @__args (#match? @__args "^\\(\\s*\\)$")
    (#set! "min_kernel_version" "5.11.0")
)
