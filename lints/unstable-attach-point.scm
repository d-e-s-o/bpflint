(function_definition
    (sec_specifier
        value: (string_literal) @probe
        (#match? @probe "^\"(k(ret)?probe|f(entry|exit))/[^\"\\n]+\"$")
    )
)
