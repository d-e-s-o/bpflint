(preproc_call_expression
    macro_name: (identifier) @__name (#eq? @__name "__uint")
    arg1: (identifier) @__arg1 (#any-of? @__arg1 "key_size" "value_size")
    (sizeof_expression)
    (#set! "message" "__uint(a, sizeof(b)) does not contain potentially relevant type information, consider using __type(a, b) instead")
) @call
