(call_expression
    function: (identifier) @function (#eq? @function "bpf_probe_read")
    arguments: (argument_list
                  (expression)
                  (expression)
                  (expression)
               )
    (#set! "message" "bpf_probe_read() is deprecated and replaced by bpf_probe_user() and bpf_probe_kernel(); refer to bpf-helpers(7)")
)
