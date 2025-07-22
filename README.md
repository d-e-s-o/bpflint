[![pipeline](https://github.com/d-e-s-o/bpflint/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/d-e-s-o/bpflint/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/bpflint.svg)](https://crates.io/crates/bpflint)
[![Docs](https://docs.rs/bpflint/badge.svg)](https://docs.rs/bpflint)

bpflint
=======

- [Library documentation][docs-rs]
- [Library changelog](CHANGELOG.md)

Linting functionality for BPF C kernel programs. The Linux kernel's BPF
sub-system is continuously being improved and certain patterns
recommended in the past may no longer be state-of-the-art today.
Similarly, some "foot guns" exist that by definition may not be obvious
to new comers.

**bpflint** contains a linter for BPF C kernel programs that accepts
such a `.bpf.c` file as input and scans it for such known issues,
pointing them out and providing recommendations on how to fix them.

Provided is a Rust library, a [command line interface](cli/), as well as
[Web UI](https://d-e-s-o.github.io/bpflint/) for linting of BPF C
program.

### 📚 Frequently Asked Questions (FAQ)

#### ❓ **Q: Why are there so few lints?**
**A:** This repository provides basic infrastructure components to build
       on, but we hope for contributions from the community for best
       practices and how to formalize them.

#### ❓ **Q: I am interested in helping out. How can I get started?**
**A:** We have a list of [issues](https://github.com/d-e-s-o/bpflint/issues)
       with ideas for contributions, which mark a good starting point.
       For documentation on lints specifically and how to add a new one,
       please check out the [lints/](lints/) sub-directory. All other
       questions are probably best asked in one of the existing issues
       (or a new one).

#### ❓ **Q: I got a false-positive, what can I do?**
**A:** Some lints require context that is not possible or feasible for
       the linter to acquire. E.g., `kprobe` attach point usage may be
       flagged as being an unstable attach point, but the linter cannot
       know whether a better alternative, say, in the form of a
       tracepoint, exists. **bpflint** recognizes C comments of the
       following form on blocks and statements:
       ```
       /* bpflint: disable=<lint-name> */
       ```
       When encountered, the named lint will be disabled for the
       directly following item (block, statement, ...).

[docs-rs]: https://docs.rs/bpflint/latest
