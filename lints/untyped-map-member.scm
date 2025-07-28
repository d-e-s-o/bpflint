
(preproc_call_expression
  macro_name: (identifier) @name (#eq? @name "__uint")
  (identifier)
  (sizeof_expression
    (_)*)
    (#set! "message" "__uint(a, sizeof(b)) does not contain potentially relevant type information, consider using __type(a, b) instead")
)
