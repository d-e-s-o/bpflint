## How to Write a Lint
**bpflint** uses [`tree-sitter`][tree-sitter-docs] to drive code
analysis. A lint is basically a `tree-sitter` Query. **bpflint** uses
such queries to find matches on source code and then reports those
alongside location information and some other meta data.

To better understand queries and the language that they are written in,
please refer to the official ["Queries" chapter][tree-sitter-queries]
covering them.

A lint is a regular Query, with the added requirement that it contains a
`message` [directive][tree-sitter-directives] that explains to the user
why the pattern being matched on is problematic. For an example please
refer to the [`probe-read` lint][probe-read-message].

A good introduction that to how a Query interfaces with the underlying
language grammar can be found in the ["Code Navigation Systems"
chapter][tree-sitter-code-nav].

In our case, the underlying grammar is that of the BPF C language,
available [here][tree-sitter-bpf-c-grammar].

## How to Extend the Grammar
From a `tree-sitter` parser perspective (which does not perform any
pre-processing), certain BPF C constructs relying on macros in various
positions cannot be parsed, as they constitute an invalid syntax tree
from an ANSI C perspective.

BPF C is, thus, an extension of C that provides first class support for
these BPF specific constructs.

If the grammar still lacks support for a certain construct, it should be
extended. As a general introduction to `tree-sitter` grammars, please
refer to the ["Creating Parsers" chapter][tree-sitter-parsers] of the
`tree-sitter` documentation, as it details prerequisites and necessary
background.

With the preliminaries out of the way and assumed covered, the following
workflow may be useful to quickly iterate on the BPF C parser:
- install the `tree-sitter` CLI; [instructions][tree-sitter-cli]
- clone the `tree-sitter-bpf-c` repository
```sh
$ git clone https://github.com/d-e-s-o/tree-sitter-bpf-c.git
```
- add a test capturing the new syntax (likely in
  [`test/corpus/bpf.txt`][tree-sitter-bpf-c-bpf.txt])
- adjust [`grammar.js`][tree-sitter-bpf-c-grammar] with your changes
- regenerate the parser and run tests
```sh
$ tree-sitter generate tree-sitter-bpf-c/
$ tree-sitter test
```
- as an intermediate step, you may want to get an idea of the produced
  syntax tree; this can be done on an example, for example via
```sh
$ tree-sitter parse --cst --grammar-path tree-sitter-bpf-c/ <test.bpf.c-file>
```

If you just want to know how the pre-existing C grammar parses
something, the [Playground][tree-sitter-playground] may also be of use
and requires no additional tools installed.

[tree-sitter-docs]: https://tree-sitter.github.io/tree-sitter/
[tree-sitter-directives]: https://tree-sitter.github.io/tree-sitter/using-parsers/queries/3-predicates-and-directives.html#directives
[tree-sitter-code-nav]: https://tree-sitter.github.io/tree-sitter/4-code-navigation.html
[tree-sitter-bpf-c-grammar]: https://github.com/d-e-s-o/tree-sitter-bpf-c/blob/main/grammar.js
[tree-sitter-cli]: https://github.com/tree-sitter/tree-sitter/tree/master/crates/cli
[tree-sitter-parsers]: https://tree-sitter.github.io/tree-sitter/creating-parsers/index.html
[tree-sitter-queries]: https://tree-sitter.github.io/tree-sitter/using-parsers/queries/index.html
[tree-sitter-bpf-c-bpf.txt]: https://github.com/d-e-s-o/tree-sitter-bpf-c/blob/main/test/corpus/bpf.txt
[tree-sitter-playground]: https://tree-sitter.github.io/tree-sitter/7-playground.html
[probe-read-message]: https://github.com/d-e-s-o/bpflint/blob/b8716d24fb133de0371152705bd33a1c56f51bfe/lints/probe-read.scm#L8
