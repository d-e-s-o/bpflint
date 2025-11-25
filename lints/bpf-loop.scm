(
  (call_expression
    function: (identifier) @__identifier
    arguments: (argument_list)
    (#eq? @__identifier "bpf_loop")
    (#set! "min_kernel_version" "6.4.0")
  ) @bpf_loop
)
