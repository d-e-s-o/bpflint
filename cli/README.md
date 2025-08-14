bpflinter
=========

- [Changelog](CHANGELOG.md)

**bpflinter** is a command line based linter for BPF C code. It is
powered by the [**bpflint**][bpflint] library.

Installation
------------

The program is self-contained with only the binary necessary. The
easiest way to install it is by downloading a pre-built, statically
linked binary attached to the latest [`cli-*` release][cli-releases].

For subsequent approaches, please be advised that a Rust toolchain is
necessary.

If you want to build and install the most recent release from
`crates.io`, use
```sh
$ cargo install bpflinter
```

Alternatively, you can build the program from the source contained in
this repository, use
```sh
$ cargo build --package bpflinter
```

Usage
-----

To lint a `*.bpf.c` file, just provide it's path as argument. E.g.,
```
$ bpflinter ../examples/task_longrun.bpf.c
warning: [probe-read] bpf_probe_read() is deprecated and replaced by bpf_probe_user() and bpf_probe_kernel(); refer to bpf-helpers(7)
  --> ../examples/task_longrun.bpf.c:43:24
   |
43 |                         bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
   |                         ^^^^^^^^^^^^^^
   |
warning: [probe-read] bpf_probe_read() is deprecated and replaced by bpf_probe_user() and bpf_probe_kernel(); refer to bpf-helpers(7)
  --> ../examples/task_longrun.bpf.c:44:24
   |
44 |                         bpf_probe_read(event.bt, sizeof(t->bt), t->bt);
   |                         ^^^^^^^^^^^^^^
   |
```

For additional information, please refer to [USAGE.md](USAGE.md) or the
program's help text (`bpflinter --help`).

[cli-releases]: https://github.com/d-e-s-o/bpflint/releases
[bpflint]: https://github.com/d-e-s-o/bpflint
