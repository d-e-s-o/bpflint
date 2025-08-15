Unreleased
----------
- Added `get-current-task` lint


0.1.2
-----
- Add support for "internal captures" (named `__xxx`) to lints
- Added `untyped-map-member` lint
- Added support for reporting multi-line matches to `report_terminal`
- Embed lint source code directly into build-time generated `lint.rs`
  module


0.1.1
-----
- Added `unstable-attach-point` lint
- Added `builtin_lints` function for retrieving list of built-in lints
- Added support for disabling lints via code comments of the form
  `bpflint: disable=<lint-name>`
- Ensured `lint` reports matches in source code order
- Fixed `report_terminal` to correctly handle matches on top most line


0.1.0
-----
- Initial release
