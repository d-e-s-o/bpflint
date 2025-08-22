use std::io;
use std::path::Path;

use anyhow::Result;

use crate::LintMatch;
use crate::lines::Lines;


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
pub fn report_terminal(
    r#match: &LintMatch,
    code: &[u8],
    path: &Path,
    writer: &mut dyn io::Write,
) -> Result<()> {
    let LintMatch {
        lint_name,
        message,
        range,
    } = r#match;

    writeln!(writer, "warning: [{lint_name}] {message}")?;
    let start_row = range.start_point.row;
    let end_row = range.end_point.row;
    let start_col = range.start_point.col;
    let end_col = range.end_point.col;
    writeln!(writer, "  --> {}:{start_row}:{start_col}", path.display())?;
    let width = end_row.to_string().len();

    if range.bytes.is_empty() {
        return Ok(())
    }

    // SANITY: It would be a tree-sitter bug the range does not
    //         map to a valid code location.
    let mut lines = Lines::new(code, range.bytes.start);
    // Use the end row here, as it's the largest number, so we end up
    // with a consistent indentation.
    let prefix = format!("{:width$} | ", "");
    writeln!(writer, "{prefix}")?;

    if start_row == end_row {
        let lprefix = format!("{start_row} | ");
        // SANITY: `Lines` will always report at least a single
        //          line.
        let line = lines.next().unwrap();
        writeln!(writer, "{lprefix}{}", String::from_utf8_lossy(line))?;
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
            let lprefix = format!("{row:width$} | ");
            let c = if idx == 0 { "/" } else { "|" };
            // SANITY: There will always be another line available,
            //         given that we are within the bounds of `range`,
            //         which maps to source code lines.
            let line = lines.next().unwrap();
            writeln!(writer, "{lprefix} {c} {}", String::from_utf8_lossy(line))?;
        }
        writeln!(writer, "{prefix} |{:_<width$}^", "", width = end_col)?;
    }

    writeln!(writer, "{prefix}")?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    use pretty_assertions::assert_eq;

    use crate::Point;
    use crate::Range;


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
        let mut report = Vec::new();
        let () =
            report_terminal(&m, code.as_bytes(), Path::new("./no_bytes.c"), &mut report).unwrap();
        let report = String::from_utf8(report).unwrap();
        let expected = indoc! { r#"
          warning: [bogus-file-extension] by convention BPF C code should use the file extension '.bpf.c'
            --> ./no_bytes.c:0:0
        "# };
        assert_eq!(report, expected);
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
        let mut report = Vec::new();
        let () = report_terminal(&m, code.as_bytes(), Path::new("<stdin>"), &mut report).unwrap();
        let report = String::from_utf8(report).unwrap();
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
        assert_eq!(report, expected);
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
        let mut report = Vec::new();
        let () = report_terminal(&m, code.as_bytes(), Path::new("<stdin>"), &mut report).unwrap();
        let report = String::from_utf8(report).unwrap();
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
        assert_eq!(report, expected);
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
        let mut report = Vec::new();
        let () = report_terminal(&m, code.as_bytes(), Path::new("<stdin>"), &mut report).unwrap();
        let report = String::from_utf8(report).unwrap();
        let expected = indoc! { r#"
          warning: [probe-read] bpf_probe_read() is deprecated
            --> <stdin>:6:4
            | 
          6 |     bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
            |     ^^^^^^^^^^^^^^
            | 
        "# };
        assert_eq!(report, expected);
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
        let mut report = Vec::new();
        let () = report_terminal(&m, code.as_bytes(), Path::new("<stdin>"), &mut report).unwrap();
        let report = String::from_utf8(report).unwrap();
        let expected = indoc! { r#"
          warning: [unstable-attach-point] kprobe/kretprobe/fentry/fexit are unstable
            --> <stdin>:0:4
            | 
          0 | SEC("kprobe/test")
            |     ^^^^^^^^^^^^^
            | 
        "# };
        assert_eq!(report, expected);
    }
}
