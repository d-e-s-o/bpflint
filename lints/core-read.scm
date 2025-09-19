(call_expression
    function: (identifier) @function (#any-of? @function "bpf_core_read" "BPF_CORE_READ")
    (#set! "message" "bpf_core_read() and BPF_CORE_READ() have been subsumed by bpf_core_cast() -- consider using it instead; refer to https://docs.ebpf.io/ebpf-library/libbpf/ebpf/bpf_core_cast/")
)
