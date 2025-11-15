(preproc_call_expression
    macro_name: (identifier) @__name (#eq? @__name "__uint")
    arg1: (identifier) @__arg1 (#any-of? @__arg1 "key_size" "value_size")
    (sizeof_expression)
    (#set! "min_kernel_version" "4.18.0")
) @call
