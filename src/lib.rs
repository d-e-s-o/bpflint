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
use std::str::FromStr;

use anyhow::Context as _;

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

/// A version in the form of a (major, minor, patch) triple.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u8, pub u8, pub u8);

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() != 3 {
            anyhow::bail!(
                "version must be in format 'major.minor.patch' (e.g., '5.4.0'), got '{s}'"
            );
        }

        let major = parts[0].parse::<u8>().with_context(|| {
            format!(
                "invalid major version: '{}' (must be a non-negative integer)",
                parts[0]
            )
        })?;

        let minor = parts[1].parse::<u8>().with_context(|| {
            format!(
                "invalid minor version: '{}' (must be a non-negative integer)",
                parts[1]
            )
        })?;

        let patch = parts[2].parse::<u8>().with_context(|| {
            format!(
                "invalid patch version: '{}' (must be a non-negative integer)",
                parts[2]
            )
        })?;

        Ok(Version(major, minor, patch))
    }
}

pub use crate::lint::Lint;
pub use crate::lint::LintMatch;
pub use crate::lint::LintOpts;
pub use crate::lint::builtin_lints;
pub use crate::lint::lint;
pub use crate::lint::lint_custom;
pub use crate::lint::lint_custom_opts;
pub use crate::report::terminal;


#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::borrow::Cow;
    use std::io::Write as _;
    use std::path::Path;

    use anyhow::Context as _;
    use anyhow::Error;

    use wasm_bindgen::prelude::wasm_bindgen;

    use super::*;

    /// Escape HTML of the provided input.
    fn escape_html(text: &str) -> Cow<'_, str> {
        html_escape::encode_safe(text)
    }

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
        fn lint_impl(code: Vec<u8>, path: String, context: u8) -> Result<String, Error> {
            let opts = terminal::Opts {
                extra_lines: (context, context),
                color: true,
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

                // Let's now make the match and other input to the
                // terminal HTML safe.
                let LintMatch {
                    lint_name,
                    message,
                    range,
                } = m;
                let m = LintMatch {
                    lint_name: escape_html(&lint_name).into_owned(),
                    message: escape_html(&message).into_owned(),
                    range,
                };
                let path = escape_html(&path);
                let escaped_path = Path::new(path.as_ref());

                let () = terminal::report_opts(&m, &code, escaped_path, &opts, &mut report)?;
            }
            let report =
                String::from_utf8(report).context("generated report contains invalid UTF-8")?;
            Ok(report)
        }

        lint_impl(code, path, context).map_err(|err| format!("{err:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    /// Make sure that we can parse valid version strings.
    #[test]
    fn version_parsing_valid() {
        let version = Version::from_str("5.4.0").unwrap();
        assert_eq!((version.0, version.1, version.2), (5, 4, 0));

        let version = Version::from_str("84.71.23").unwrap();
        assert_eq!((version.0, version.1, version.2), (84, 71, 23));
    }

    /// Check for expected version parsing errors due to an invalid
    /// number.
    #[test]
    fn version_parsing_invalid() {
        let version = Version::from_str("5.bfp.0");
        assert!(version.is_err());
    }

    /// Check for expected version parsing errors due to too many parts.
    #[test]
    fn version_parsing_too_many_parts() {
        let version = Version::from_str("5.1.0.9");
        assert!(version.is_err());
    }

    /// Check for expected version parsing errors due to too few parts.
    #[test]
    fn version_parsing_too_few_parts() {
        let version = Version::from_str("4.8");
        assert!(version.is_err());
    }

    /// Check that version equality comparisons work as they should.
    #[test]
    fn version_equality() {
        assert_eq!(Version(0, 0, 0), Version(0, 0, 0));
        assert_eq!(Version(1, 1, 1), Version(1, 1, 1));

        assert_ne!(Version(0, 0, 0), Version(0, 0, 1));
        assert_ne!(Version(0, 0, 0), Version(0, 1, 0));
        assert_ne!(Version(0, 0, 0), Version(1, 0, 0));
        assert_ne!(Version(0, 0, 1), Version(0, 0, 0));
        assert_ne!(Version(0, 1, 0), Version(0, 0, 0));
        assert_ne!(Version(1, 0, 0), Version(0, 0, 0));
    }

    /// Make sure that versions can be ordered properly.
    #[test]
    fn version_ordering() {
        let mut versions = [
            Version(20, 20, 1),
            Version(20, 1, 10),
            Version(1, 1, 10),
            Version(1, 1, 1),
            Version(1, 1, 0),
            Version(1, 0, 0),
            Version(0, 0, 0),
        ];

        let () = versions.sort();

        let expected = [
            Version(0, 0, 0),
            Version(1, 0, 0),
            Version(1, 1, 0),
            Version(1, 1, 1),
            Version(1, 1, 10),
            Version(20, 1, 10),
            Version(20, 20, 1),
        ];
        assert_eq!(versions, expected);
    }
}
