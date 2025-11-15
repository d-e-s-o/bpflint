(function_definition
    (sec_specifier
        value: (string_literal) @probe
        (#match? @probe "^\"(k(ret)?probe|f(entry|exit))/[^\"\\n]+\"$")
    )
    (#set! "min_kernel_version" "4.7.0")
)
