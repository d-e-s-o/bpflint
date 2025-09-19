[![pipeline](https://github.com/d-e-s-o/bpflint/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/d-e-s-o/bpflint/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/bpflint.svg)](https://crates.io/crates/bpflint)
[![Docs](https://docs.rs/bpflint/badge.svg)](https://docs.rs/bpflint)

bpflint <img src="var/logo.png" style="height:1em">
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

Provided is a Rust library, a [command line interface](cli/), a [Web
UI][web-ui], as well as a [GitHub Action][gh-action] for linting of BPF
C programs.

### 📚 Frequently Asked Questions (FAQ)

#### ❓ **Q: I am interested in helping out. How can I get started?**
**A:** We have a list of [issues](https://github.com/d-e-s-o/bpflint/issues)
       with ideas for contributions, which mark a good starting point.
       For documentation on lints specifically and how to add a new one,
       please check out the [lints/](lints/) sub-directory. All other
       questions are probably best asked in one of the existing issues
       (or a new one).

#### ❓ **Q: I have a repository with BPF code, can I lint contributions easily?**
**A:** If you are using GitHub Actions as the continuous integration
       solution of choice, you can add a separate job using the
       [`lint-bpf` Action][gh-action].
       For other CI systems some manual plumbing will be necessary. We
       provide statically linked `bpflinter` CLI binaries that can be
       downloaded from each `cli-vX.Y.Z` release and used directly on
       any Linux. E.g.,
       https://github.com/d-e-s-o/bpflint/releases/download/cli-v0.1.2/bpflinter-x86_64-unknown-linux-musl

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
[gh-action]: https://github.com/d-e-s-o/lint-bpf
[web-ui]: https://d-e-s-o.github.io/bpflint/
