(
  (preproc_call
    directive: (preproc_directive) @__directive
    argument: (preproc_arg) @__arg
    (#eq? @__directive "#pragma")
    (#eq? @__arg "unroll")
    (#set! "min_kernel_version" "6.4.0")
  ) @pragma_unroll
  .
  (for_statement
    condition: (binary_expression)
  )
)
