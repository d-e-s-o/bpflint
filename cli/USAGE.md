```
A command line interface for bpflint

Usage: bpflinter [OPTIONS] <[@]SRCS>...

Arguments:
  <[@]SRCS>...
          The BPF C source files to lint.
          
          Use '@file' syntax to include a (newline separated) list of files from 'file'.

Options:
  -B, --before <BEFORE>
          Number of lines to show before the lint match

  -A, --after <AFTER>
          Number of lines to show after the lint match

  -C, --context <CONTEXT>
          Number of lines to show before and after the lint match

      --color <COLOR>
          Whether or not to color output and or syntax highlighting for code snippets
          
          [possible values: true, false]

      --print-lints
          Print a list of available lints

  -v, --verbose...
          Increase verbosity (can be supplied multiple times)

  -k, --kernel_version <KERNEL_VERSION>
          User kernel version to check lints against

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
