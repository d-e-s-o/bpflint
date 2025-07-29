(function_definition
    (sec_specifier
        value: (string_literal) @probe
        (#match? @probe "^\"(k(ret)?probe|f(entry|exit))/[^\"\\n]+\"$")
    )
    (#set! "message" "kprobe/kretprobe/fentry/fexit are conceptually unstable and prone to changes between kernel versions; consider more stable attach points such as tracepoints or LSM hooks, if available")
)
