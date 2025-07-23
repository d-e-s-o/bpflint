use std::io;
use std::path::Path;

use anyhow::Result;

use tracing::warn;

use crate::LintMatch;


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
    if range.start_point.row == range.end_point.row {
        let row = range.start_point.row;
        let col = range.start_point.col;
        writeln!(writer, "  --> {}:{row}:{col}", path.display())?;
        if !range.bytes.is_empty() {
            let row_str = row.to_string();
            let lprefix = format!("{row} | ");
            let prefix = format!("{:width$} | ", "", width = row_str.len());
            writeln!(writer, "{prefix}")?;
            let line_start = code[..range.bytes.end]
                .iter()
                .rposition(|&b| b == b'\n')
                .map(|idx| idx + 1)
                .unwrap_or(0);
            // TODO: `end_byte` seems to be exclusive, meaning we may end up
            //       panicking here.
            let line_end = range.bytes.end
                + code[range.bytes.end..]
                    .iter()
                    .position(|&b| b == b'\n')
                    .unwrap_or(0);
            let line = &code[line_start..line_end];
            writeln!(writer, "{lprefix}{}", String::from_utf8_lossy(line))?;
            writeln!(
                writer,
                "{prefix}{:indent$}{:^<width$}",
                "",
                "",
                indent = range.start_point.col,
                width = range.end_point.col.saturating_sub(range.start_point.col)
            )?;
            writeln!(writer, "{prefix}")?;
        }
    } else {
        // TODO: Implement.
        warn!("multi-line reporting is not yet supported");
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::Point;
    use crate::Range;


    /// Tests that a match with an empty range includes no code snippet.
    #[test]
    fn empty_range_reporting() {
        let code = r#"int main(){}"#;

        let m = LintMatch {
            lint_name: "bogus-file-extension".to_string(),
            message: "by convention BPF C code should use the file extension '.bpf.c'".to_string(),
            range: Range {
                bytes: 0..0,
                start_point: Point { row: 0, col: 0 },
                end_point: Point { row: 0, col: 0 },
            },
        };
        let mut report = Vec::new();
        let () =
            report_terminal(&m, code.as_bytes(), Path::new("./no_bytes.c"), &mut report).unwrap();
        let report = String::from_utf8(report).unwrap();
        let expected = r#"warning: [bogus-file-extension] by convention BPF C code should use the file extension '.bpf.c'
  --> ./no_bytes.c:0:0
"#;
        assert_eq!(report, expected);
    }

    /// Check that our "terminal" reporting works as expected.
    #[test]
    fn terminal_reporting() {
        let code = r#"
SEC("tp_btf/sched_switch")
int handle__sched_switch(u64 *ctx)
{
    struct task_struct *prev = (struct task_struct *)ctx[1];
    struct event event = {0};
    bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
    return 0;
}
"#;

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
        let expected = r#"warning: [probe-read] bpf_probe_read() is deprecated
  --> <stdin>:6:4
  | 
6 |     bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
  |     ^^^^^^^^^^^^^^
  | 
"#;
        assert_eq!(report, expected);
    }

    /// Check that reporting works properly when the match is on the
    /// very first line of input.
    #[test]
    fn report_top_most_line() {
        let code = r#"SEC("kprobe/test")
int handle__test(void)
{
}
"#;

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
        let expected = r#"warning: [unstable-attach-point] kprobe/kretprobe/fentry/fexit are unstable
  --> <stdin>:0:4
  | 
0 | SEC("kprobe/test")
  |     ^^^^^^^^^^^^^
  | 
"#;
        assert_eq!(report, expected);
    }
}
