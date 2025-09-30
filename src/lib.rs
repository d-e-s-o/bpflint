//! A linter for BPF C code.
//!
//! At the source code level, individual lints can be disabled with
//! source code comments of the form
//! ```c
//! /* bpflint: disable=probe-read */
//! bpf_probe_read(/* ... */);
//! ```
//!
//! In this instance, `probe-read` is the name of the lint to disable.
//!
//! Entire blocks can be annotated as well:
//! ```c
//! /* bpflint: disable=probe-read */
//! void handler(void) {
//!     void *dst = /* ... */
//!     bpf_probe_read(dst, /* ... */);
//! }
//! ```
//!
//! In the above examples, none of the instances of `bpf_probe_read`
//! will be flagged.
//!
//! The directive `bpflint: disable=all` acts as a catch-all, disabling
//! reporting of all lints.

#[cfg(target_arch = "wasm32")]
#[macro_use]
mod redefine;

mod lines;
mod lint;
mod report;

use std::ops;


/// A position in a multi-line text document, in terms of rows and columns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point {
    /// A row number in source code (zero-based).
    pub row: usize,
    /// A column number in source code (zero-based).
    pub col: usize,
}

/// A range of positions in a multi-line text document, both in terms of bytes
/// and of rows and columns.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Range {
    /// The byte range in the source code.
    pub bytes: ops::Range<usize>,
    /// The logical start point of the represented range.
    pub start_point: Point,
    /// The logical end point of the represented range.
    pub end_point: Point,
}

pub use crate::lint::Lint;
pub use crate::lint::LintMatch;
pub use crate::lint::builtin_lints;
pub use crate::lint::lint;
pub use crate::report::Opts;
pub use crate::report::report_terminal;
pub use crate::report::report_terminal_opts;


#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::io::Write as _;
    use std::path::PathBuf;

    use anyhow::Context as _;
    use anyhow::Error;

    use wasm_bindgen::prelude::wasm_bindgen;

    use super::*;

    /// Lint source code `code` representing a file at `path` and
    /// produce a report, end-to-end. `context` describes the number of
    /// lines of source code context to include in the report.
    ///
    /// This function exists mostly because exposing something like our
    /// `LintMatch` type across WASM's ABI is a major PITA and our
    /// interactive service only cares about the end-to-end workflow
    /// anyway.
    #[wasm_bindgen]
    pub fn lint_html(code: Vec<u8>, path: String, context: u8) -> Result<String, String> {
        fn lint_impl(code: Vec<u8>, path: PathBuf, context: u8) -> Result<String, Error> {
            let opts = Opts {
                extra_lines: (context, context),
                ..Default::default()
            };
            let mut first = true;
            let mut report = Vec::new();
            let matches = lint(&code)?;
            for m in matches {
                if !first {
                    writeln!(&mut report)?;
                } else {
                    first = false;
                }

                let () = report_terminal_opts(&m, &code, &path, &opts, &mut report)?;
            }
            let report =
                String::from_utf8(report).context("generated report contains invalid UTF-8")?;
            Ok(report)
        }

        lint_impl(code, PathBuf::from(path), context).map_err(|err| format!("{err:?}"))
    }
}
