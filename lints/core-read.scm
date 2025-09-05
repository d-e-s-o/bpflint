(call_expression
    function: (identifier) @function (#any-of? @function "bpf_core_read" "BPF_CORE_READ")
    (#set! "message" "bpf_core_read() and BPF_CORE_READ() are deprecated and replaced by bpf_core_cast(); refer to https://docs.ebpf.io/ebpf-library/libbpf/ebpf/bpf_core_cast/")
)
