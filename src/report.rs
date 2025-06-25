use std::io;
use std::path::Path;
#[cfg(target_arch = "wasm32")]
use std::path::PathBuf;

use anyhow::Result;

use tracing::warn;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::LintMatch;


/// Report a lint match in terminal style.
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
        let col = range.start_point.column;
        writeln!(writer, "  --> {}:{row}:{col}", path.display())?;
        let row_str = row.to_string();
        let lprefix = format!("{row} | ");
        let prefix = format!("{:width$} | ", "", width = row_str.len());
        writeln!(writer, "{prefix}")?;
        let line_start = code[..range.start_byte]
            .iter()
            .rposition(|&b| b == b'\n')
            .map(|idx| idx + 1)
            .unwrap_or(range.start_byte);
        // TODO: `end_byte` seems to be exclusive, meaning we may end up
        //       panicking here.
        let line_end = range.end_byte
            + code[range.end_byte..]
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
            indent = range.start_point.column,
            width = range
                .end_point
                .column
                .saturating_sub(range.start_point.column)
        )?;
        writeln!(writer, "{prefix}")?;
    } else {
        // TODO: Implement.
        warn!("multi-line reporting is not yet supported");
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn report_terminal_wasm(
    r#match: LintMatch,
    code: Vec<u8>,
    path: PathBuf,
    writer: &mut dyn io::Write,
) -> Result<(), JsValue> {
    report_terminal(&r#match, &code, &path, writer)
        .map_err(|err| JsValue::from_str(&format!("{:?}", err)))
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::Point;
    use crate::Range;


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
                start_byte: 160,
                end_byte: 174,
                start_point: Point { row: 6, column: 4 },
                end_point: Point { row: 6, column: 18 },
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
}
