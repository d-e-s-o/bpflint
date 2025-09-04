(call_expression
    function: (identifier) @function (#eq? @function "bpf_core_read")
    (#set! "message" "bpf_core_read() is deprecated and replaced by bpf_core_cast(); refer to https://docs.ebpf.io/ebpf-library/libbpf/ebpf/bpf_core_cast/")
)

(call_expression
    function: (identifier) @function (#eq? @function "BPF_CORE_READ")
    (#set! "message" "BPF_CORE_READ() is deprecated and replaced by bpf_core_cast(); refer to https://docs.ebpf.io/ebpf-library/libbpf/ebpf/bpf_core_cast/")
)
