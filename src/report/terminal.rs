//! Functionality for reporting lint matches on a terminal.

use std::io;
use std::path::Path;

use anyhow::Context as _;
use anyhow::Error;
use anyhow::Result;

use crate::LintMatch;
use crate::lines::Lines;

use super::highlight::create_highlighter;


/// Configuration options for terminal reporting.
#[derive(Default, Clone, Debug)]
pub struct Opts {
    /// Extra lines of context to report before and after a match.
    pub extra_lines: (u8, u8),
    /// Whether to colorize the output.
    pub color: bool,
    /// The struct is non-exhaustive and open to extension.
    #[doc(hidden)]
    pub _non_exhaustive: (),
}


/// Report a lint match in terminal style.
///
/// - `match` is the match to create a report for
/// - `code` is the source code in question, as passed to
///   [`lint`][crate::lint()]
/// - `path` should be the path to the file to which `code` corresponds
///   and is used to enhance the generated report
/// - `writer` is a reference to a [`io::Write`] to which to write the
///   report
///
/// # Example
/// ```text
/// warning: [probe-read] bpf_probe_read() is deprecated and replaced by
///          bpf_probe_user() and bpf_probe_kernel(); refer to bpf-helpers(7)
///   --> example.bpf.c:43:24
///    |
/// 43 |                         bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
///    |                         ^^^^^^^^^^^^^^
///    |
/// ```
pub fn report(
    r#match: &LintMatch,
    code: &[u8],
    path: &Path,
    writer: &mut dyn io::Write,
) -> Result<()> {
    report_opts(r#match, code, path, &Opts::default(), writer)
}

/// Report a lint match in terminal style with extra lines for context as configured.
///
/// - `match` is the match to create a report for
/// - `code` is the source code in question, as passed to
///   [`lint`][crate::lint()]
/// - `path` should be the path to the file to which `code` corresponds
///   and is used to enhance the generated report
/// - `opts` specifies the reporting options including context lines
/// - `writer` is a reference to a [`io::Write`] to which to write the
///   report
///
/// # Example
/// ```text
/// warning: [probe-read] bpf_probe_read() is deprecated and replaced by
///          bpf_probe_user() and bpf_probe_kernel(); refer to bpf-helpers(7)
///   --> example.bpf.c:43:24
///    |
/// 41 |     struct task_struct *prev = (struct task_struct *)ctx[1];
/// 42 |     struct event event = {0};
/// 43 |     bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
///    |     ^^^^^^^^^^^^^^
/// 44 |     return 0;
/// 45 | }
///    |
/// ```
pub fn report_opts(
    r#match: &LintMatch,
    code: &[u8],
    path: &Path,
    opts: &Opts,
    writer: &mut dyn io::Write,
) -> Result<()> {
    let LintMatch {
        lint_name,
        message,
        range,
    } = r#match;

    let highlighter = create_highlighter(opts.color)?;

    writeln!(writer, "warning: [{lint_name}] {message}")?;
    let start_row = range.start_point.row;
    let end_row = range.end_point.row;
    let start_col = range.start_point.col;
    let end_col = range.end_point.col;
    writeln!(writer, "  --> {}:{start_row}:{start_col}", path.display())?;
    let prefix_indent = (end_row + usize::from(opts.extra_lines.1))
        .to_string()
        .len();

    if range.bytes.is_empty() {
        return Ok(())
    }

    // Use the end row here, as it's the largest number, so we end up
    // with a consistent indentation.
    let prefix = format!("{:prefix_indent$} | ", "");
    writeln!(writer, "{prefix}")?;

    // Print source code context before the actual match. Need to
    // `collect` here, because the `Write` interface we work with forces
    // us to emit data in sequential order, but we necessary have to go
    // backwards from the match.
    // SANITY: It would be a tree-sitter bug the range does not
    //         map to a valid code location.
    let () = Lines::new(code, range.bytes.start)
        .rev()
        // Skip the line of the match.
        .skip(1)
        .take(opts.extra_lines.0.into())
        .collect::<Vec<&[u8]>>()
        .into_iter()
        .enumerate()
        .rev()
        .try_for_each(|(row_sub, line)| {
            let row = start_row - row_sub - 1;
            let lprefix = format!("{row:prefix_indent$} | ");
            let highlighted = highlighter
                .highlight(line)
                .context("failed to highlight source code line `{line}`")?;
            writeln!(writer, "{lprefix}{highlighted}").map_err(Error::from)
        })?;

    // SANITY: It would be a tree-sitter bug the range does not
    //         map to a valid code location.
    let mut lines = Lines::new(code, range.bytes.start);

    if start_row == end_row {
        let lprefix = format!("{start_row:prefix_indent$} | ");
        // SANITY: `Lines` will always report at least a single
        //          line.
        let line = lines.next().unwrap();
        let highlighted = highlighter
            .highlight(line)
            .context("failed to highlight source code line `{line}`")?;
        writeln!(writer, "{lprefix}{highlighted}")?;
        writeln!(
            writer,
            "{prefix}{:indent$}{:^<width$}",
            "",
            "",
            indent = start_col,
            width = end_col.saturating_sub(start_col)
        )?;
    } else {
        for (idx, row) in (start_row..=end_row).enumerate() {
            let lprefix = format!("{row:prefix_indent$} | ");
            let c = if idx == 0 { "/" } else { "|" };
            // Our `Lines` logic may not report a trailing newline if it
            // is completely empty, but `tree-sitter` may actually
            // report it. If that's the case just ignore this empty
            // line.
            let Some(line) = lines.next() else { break };
            let highlighted = highlighter
                .highlight(line)
                .context("failed to highlight source code line `{line}`")?;
            writeln!(writer, "{lprefix} {c} {highlighted}")?;
        }
        writeln!(writer, "{prefix} |{:_<width$}^", "", width = end_col)?;
    }

    let () = lines
        .take(opts.extra_lines.1.into())
        .enumerate()
        .try_for_each(|(row_add, line)| {
            let row = end_row + row_add + 1;
            let lprefix = format!("{row:prefix_indent$} | ");
            let highlighted = highlighter
                .highlight(line)
                .context("failed to highlight source code line `{line}`")?;
            writeln!(writer, "{lprefix}{highlighted}").map_err(Error::from)
        })?;

    writeln!(writer, "{prefix}")?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    use indoc::formatdoc;
    use indoc::indoc;

    use pretty_assertions::assert_eq;

    use crate::Point;
    use crate::Range;

    use super::super::ansi_color::COLOR_PINK;
    use super::super::ansi_color::COLOR_RESET;
    use super::super::ansi_color::COLOR_TEAL;


    /// Tests that a match with an empty range includes no code snippet.
    #[test]
    fn empty_range_reporting() {
        let code = indoc! { r#"
            int main() {}
        "# };

        let m = LintMatch {
            lint_name: "bogus-file-extension".to_string(),
            message: "by convention BPF C code should use the file extension '.bpf.c'".to_string(),
            range: Range {
                bytes: 0..0,
                start_point: Point::default(),
                end_point: Point::default(),
            },
        };
        let mut r = Vec::new();
        let () = report(&m, code.as_bytes(), Path::new("./no_bytes.c"), &mut r).unwrap();
        let r = String::from_utf8(r).unwrap();
        let expected = indoc! { r#"
            warning: [bogus-file-extension] by convention BPF C code should use the file extension '.bpf.c'
              --> ./no_bytes.c:0:0
        "# };
        assert_eq!(r, expected);
    }

    /// Make sure that multi-line matches are reported correctly.
    #[test]
    fn multi_line_report() {
        let code = indoc! { r#"
            SEC("tp_btf/sched_switch")
            int handle__sched_switch(u64 *ctx) {
                bpf_probe_read(
                  event.comm,
                  TASK_COMM_LEN,
                  prev->comm);
                return 0;
            }
        "# };

        let m = LintMatch {
            lint_name: "probe-read".to_string(),
            message: "bpf_probe_read() is deprecated".to_string(),
            range: Range {
                bytes: 68..140,
                start_point: Point { row: 2, col: 4 },
                end_point: Point { row: 5, col: 17 },
            },
        };
        let mut r = Vec::new();
        let () = report(&m, code.as_bytes(), Path::new("<stdin>"), &mut r).unwrap();
        let r = String::from_utf8(r).unwrap();
        let expected = indoc! { r#"
            warning: [probe-read] bpf_probe_read() is deprecated
              --> <stdin>:2:4
              | 
            2 |  /     bpf_probe_read(
            3 |  |       event.comm,
            4 |  |       TASK_COMM_LEN,
            5 |  |       prev->comm);
              |  |_________________^
              | 
        "# };
        assert_eq!(r, expected);
    }

    /// Make sure that multi-line matches that are straddling a power of
    /// ten line number are reported correctly.
    #[test]
    fn multi_line_report_line_numbers() {
        let code = indoc! { r#"
            /* A
             * bunch
             * of
             * filling
             */
            SEC("tp_btf/sched_switch")
            int handle__sched_switch(u64 *ctx) {
                bpf_probe_read(
                  event.comm,
                  TASK_COMM_LEN,
                  prev->comm);
                return 0;
            }
        "# };

        let m = LintMatch {
            lint_name: "probe-read".to_string(),
            message: "bpf_probe_read() is deprecated".to_string(),
            range: Range {
                bytes: 103..175,
                start_point: Point { row: 7, col: 4 },
                end_point: Point { row: 10, col: 17 },
            },
        };
        let mut r = Vec::new();
        let () = report(&m, code.as_bytes(), Path::new("<stdin>"), &mut r).unwrap();
        let r = String::from_utf8(r).unwrap();
        let expected = indoc! { r#"
            warning: [probe-read] bpf_probe_read() is deprecated
              --> <stdin>:7:4
               | 
             7 |  /     bpf_probe_read(
             8 |  |       event.comm,
             9 |  |       TASK_COMM_LEN,
            10 |  |       prev->comm);
               |  |_________________^
               | 
        "# };
        assert_eq!(r, expected);
    }

    /// Check that we "correctly" report matches effectively spanning
    /// the end of the file.
    ///
    /// This can happen for queries that use `preproc_def`, because it
    /// includes trailing newlines in its match.
    #[test]
    fn multi_line_trailing_line_empty() {
        let code = indoc! { r#"
            #define DONT_ENABLE 1
        "# };
        let m = LintMatch {
            lint_name: "lint".to_string(),
            message: "message".to_string(),
            range: Range {
                bytes: 0..21,
                start_point: Point { row: 0, col: 0 },
                end_point: Point { row: 1, col: 0 },
            },
        };

        let mut r = Vec::new();
        let () = report(&m, code.as_bytes(), Path::new("<stdin>"), &mut r).unwrap();
        let r = String::from_utf8(r).unwrap();

        // Note that ideally we'd fine a way to just highlight the
        // entire line instead of using the multi-line reporting path
        // here, but it's not trivial to do so.
        let expected = indoc! { r#"
            warning: [lint] message
              --> <stdin>:0:0
              | 
            0 |  / #define DONT_ENABLE 1
              |  |^
              | 
        "# };
        assert_eq!(r, expected);
    }

    /// Check that our "terminal" reporting works as expected.
    #[test]
    fn terminal_reporting() {
        let code = indoc! { r#"
            SEC("tp_btf/sched_switch")
            int handle__sched_switch(u64 *ctx)
            {
                struct task_struct *prev = (struct task_struct *)ctx[1];
                struct event event = {0};
                bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
                return 0;
            }
        "# };

        let m = LintMatch {
            lint_name: "probe-read".to_string(),
            message: "bpf_probe_read() is deprecated".to_string(),
            range: Range {
                bytes: 160..174,
                start_point: Point { row: 6, col: 4 },
                end_point: Point { row: 6, col: 18 },
            },
        };
        let mut r = Vec::new();
        let () = report(&m, code.as_bytes(), Path::new("<stdin>"), &mut r).unwrap();
        let r = String::from_utf8(r).unwrap();
        let expected = indoc! { r#"
            warning: [probe-read] bpf_probe_read() is deprecated
              --> <stdin>:6:4
              | 
            6 |     bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
              |     ^^^^^^^^^^^^^^
              | 
        "# };
        assert_eq!(r, expected);
    }

    /// Check that our "terminal" reporting logic can syntax highlight
    /// properly.
    #[test]
    fn terminal_reporting_highlighted() {
        let code = indoc! { r#"
            SEC("kprobe/test")
            int handle__test(void)
            {
            }
        "# };

        let m = LintMatch {
            lint_name: "unstable-attach-point".to_string(),
            message: "kprobe/kretprobe/fentry/fexit are unstable".to_string(),
            range: Range {
                bytes: 4..17,
                start_point: Point { row: 0, col: 4 },
                end_point: Point { row: 0, col: 17 },
            },
        };
        let mut r = Vec::new();
        let opts = Opts {
            color: true,
            ..Default::default()
        };
        let () = report_opts(&m, code.as_bytes(), Path::new("<stdin>"), &opts, &mut r).unwrap();
        let r = String::from_utf8(r).unwrap();
        let expected = formatdoc! { r#"
            warning: [unstable-attach-point] kprobe/kretprobe/fentry/fexit are unstable
              --> <stdin>:0:4
              | 
            0 | {teal}SEC{reset}({pink}"kprobe/test"{reset})
              |     ^^^^^^^^^^^^^
              | 
        "#, teal = COLOR_TEAL, pink = COLOR_PINK, reset = COLOR_RESET };
        assert_eq!(r, expected);
    }

    /// Check that reporting works properly when the match is on the
    /// very first line of input.
    #[test]
    fn report_top_most_line() {
        let code = indoc! { r#"
            SEC("kprobe/test")
            int handle__test(void)
            {
            }
        "# };

        let m = LintMatch {
            lint_name: "unstable-attach-point".to_string(),
            message: "kprobe/kretprobe/fentry/fexit are unstable".to_string(),
            range: Range {
                bytes: 4..17,
                start_point: Point { row: 0, col: 4 },
                end_point: Point { row: 0, col: 17 },
            },
        };
        let mut r = Vec::new();
        let () = report(&m, code.as_bytes(), Path::new("<stdin>"), &mut r).unwrap();
        let r = String::from_utf8(r).unwrap();
        let expected = indoc! { r#"
            warning: [unstable-attach-point] kprobe/kretprobe/fentry/fexit are unstable
              --> <stdin>:0:4
              | 
            0 | SEC("kprobe/test")
              |     ^^^^^^^^^^^^^
              | 
        "# };
        assert_eq!(r, expected);
    }

    /// Test that `report_opts` with `Opts::default()` behaves
    /// identically to `report`.
    #[test]
    fn report_terminal_opts_none_context() {
        let code = indoc! { r#"
            SEC("tp_btf/sched_switch")
            int handle__sched_switch(u64 *ctx)
            {
                struct task_struct *prev = (struct task_struct *)ctx[1];
                struct event event = {0};
                bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
                return 0;
            }
        "# };

        let m = LintMatch {
            lint_name: "probe-read".to_string(),
            message: "bpf_probe_read() is deprecated".to_string(),
            range: Range {
                bytes: 160..174,
                start_point: Point { row: 5, col: 4 },
                end_point: Point { row: 5, col: 18 },
            },
        };

        let mut report_old = Vec::new();
        let mut report_new = Vec::new();

        let () = report(&m, code.as_bytes(), Path::new("<stdin>"), &mut report_old).unwrap();
        let () = report_opts(
            &m,
            code.as_bytes(),
            Path::new("<stdin>"),
            &Opts::default(),
            &mut report_new,
        )
        .unwrap();

        assert_eq!(report_old, report_new);
    }

    /// Test `report_terminal_opts` with extra context lines.
    #[test]
    fn report_terminal_opts_with_context() {
        let code = indoc! { r#"
            SEC("tp_btf/sched_switch")
            int handle__sched_switch(u64 *ctx)
            {
                struct task_struct *prev = (struct task_struct *)ctx[1];
                struct event event = {0};
                bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
                return 0;
            }
        "# };

        let m = LintMatch {
            lint_name: "probe-read".to_string(),
            message: "bpf_probe_read() is deprecated".to_string(),
            range: Range {
                bytes: 160..174,
                start_point: Point { row: 5, col: 4 },
                end_point: Point { row: 5, col: 18 },
            },
        };
        let mut r = Vec::new();
        let () = report_opts(
            &m,
            code.as_bytes(),
            Path::new("<stdin>"),
            &Opts {
                extra_lines: (2, 1),
                ..Default::default()
            },
            &mut r,
        )
        .unwrap();
        let r = String::from_utf8(r).unwrap();

        let expected = indoc! { r#"
            warning: [probe-read] bpf_probe_read() is deprecated
              --> <stdin>:5:4
              | 
            3 |     struct task_struct *prev = (struct task_struct *)ctx[1];
            4 |     struct event event = {0};
            5 |     bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
              |     ^^^^^^^^^^^^^^
            6 |     return 0;
              | 
        "# };
        assert_eq!(r, expected);
    }

    /// Test context lines with multi-line matches.
    #[test]
    fn report_terminal_opts_multiline_with_context() {
        let code = indoc! { r#"
            SEC("tp_btf/sched_switch")
            int handle__sched_switch(u64 *ctx) {
                bpf_probe_read(
                  event.comm,
                  TASK_COMM_LEN,
                  prev->comm);
                return 0;
            }
        "# };

        let m = LintMatch {
            lint_name: "probe-read".to_string(),
            message: "bpf_probe_read() is deprecated".to_string(),
            range: Range {
                bytes: 68..140,
                start_point: Point { row: 2, col: 4 },
                end_point: Point { row: 5, col: 17 },
            },
        };
        let mut r = Vec::new();
        let () = report_opts(
            &m,
            code.as_bytes(),
            Path::new("<stdin>"),
            &Opts {
                extra_lines: (1, 1),
                ..Default::default()
            },
            &mut r,
        )
        .unwrap();
        let r = String::from_utf8(r).unwrap();

        let expected = indoc! { r#"
            warning: [probe-read] bpf_probe_read() is deprecated
              --> <stdin>:2:4
              | 
            1 | int handle__sched_switch(u64 *ctx) {
            2 |  /     bpf_probe_read(
            3 |  |       event.comm,
            4 |  |       TASK_COMM_LEN,
            5 |  |       prev->comm);
              |  |_________________^
            6 |     return 0;
              | 
        "# };
        assert_eq!(r, expected);
    }

    /// Test context lines when there aren't enough lines before the error.
    #[test]
    fn report_terminal_opts_insufficient_context_before() {
        let code = indoc! { r#"
            SEC("kprobe/test")
            int handle__test(void)
            {
            }
        "# };

        let m = LintMatch {
            lint_name: "unstable-attach-point".to_string(),
            message: "kprobe/kretprobe/fentry/fexit are unstable".to_string(),
            range: Range {
                bytes: 4..17,
                start_point: Point { row: 0, col: 4 },
                end_point: Point { row: 0, col: 17 },
            },
        };
        let mut r = Vec::new();
        let () = report_opts(
            &m,
            code.as_bytes(),
            Path::new("<stdin>"),
            &Opts {
                extra_lines: (5, 2),
                ..Default::default()
            },
            &mut r,
        )
        .unwrap();
        let r = String::from_utf8(r).unwrap();

        let expected = indoc! { r#"
            warning: [unstable-attach-point] kprobe/kretprobe/fentry/fexit are unstable
              --> <stdin>:0:4
              | 
            0 | SEC("kprobe/test")
              |     ^^^^^^^^^^^^^
            1 | int handle__test(void)
            2 | {
              | 
        "# };
        assert_eq!(r, expected);
    }

    /// Test context lines when there aren't enough lines after the error.
    #[test]
    fn report_terminal_opts_insufficient_context_after() {
        let code = indoc! { r#"
            SEC("tp_btf/sched_switch")
            int handle__sched_switch(u64 *ctx)
            {
                bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
            }
        "# };

        let m = LintMatch {
            lint_name: "probe-read".to_string(),
            message: "bpf_probe_read() is deprecated".to_string(),
            range: Range {
                bytes: 68..82,
                start_point: Point { row: 3, col: 4 },
                end_point: Point { row: 3, col: 18 },
            },
        };
        let mut r = Vec::new();
        let () = report_opts(
            &m,
            code.as_bytes(),
            Path::new("<stdin>"),
            &Opts {
                extra_lines: (1, 5),
                ..Default::default()
            },
            &mut r,
        )
        .unwrap();
        let r = String::from_utf8(r).unwrap();

        let expected = indoc! { r#"
            warning: [probe-read] bpf_probe_read() is deprecated
              --> <stdin>:3:4
              | 
            2 | {
            3 |     bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
              |     ^^^^^^^^^^^^^^
            4 | }
              | 
        "# };
        assert_eq!(r, expected);
    }
}
