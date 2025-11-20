use std::fs::File;
use std::io::BufRead as _;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::Result;

use clap::ArgAction;
use clap::Parser;


fn parse_files(s: &str) -> Result<Vec<PathBuf>> {
    if let Some(rest) = s.strip_prefix('@') {
        let file =
            File::open(rest).with_context(|| format!("failed to open file list `{rest}`"))?;
        let reader = BufReader::new(file);
        let mut paths = vec![];
        for (i, line) in reader.lines().enumerate() {
            let line = line.with_context(|| format!("failed to read line {i} from `{rest}`"))?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                paths.push(PathBuf::from(trimmed));
            }
        }
        Ok(paths)
    } else {
        Ok(vec![PathBuf::from(s)])
    }
}

/// Parse a context line count for CLI arguments, with enhanced error reporting.
///
/// This function is used as a value parser for line count arguments (e.g. --before, --after, --context)
/// It converts a string input to a u8 value while providing clear error messages
/// when the input is invalid.
///
/// # Arguments
/// * `s` - Raw input string, expected to be a number from 0 to 255 inclusive.
///
/// # Returns
/// * `Ok(u8)` - The parsed line count if valid
/// * `Err(anyhow::Error)` - If the input cannot be parsed as u8 or is out of range
///
/// # Examples
/// ```
/// assert_eq!(parse_context_line_count("5").unwrap(), 5);
/// assert!(parse_context_line_count("256").is_err());
/// assert!(parse_context_line_count("abc").is_err());
/// ```
fn parse_context_line_count(s: &str) -> Result<u8> {
    let line_count = s
        .parse::<u8>()
        .with_context(|| format!("invalid context line count: '{s}' (must be 0-255)"))?;
    Ok(line_count)
}

/// A command line interface for bpflint.
#[derive(Debug, Parser)]
#[command(version = env!("VERSION"))]
pub struct Args {
    /// The BPF C source files to lint.
    ///
    /// Use '@file' syntax to include a (newline separated) list of
    /// files from 'file'.
    #[arg(required = true, value_name = "[@]SRCS", value_parser = parse_files)]
    pub srcs: Vec<Vec<PathBuf>>,
    /// Number of lines to show before the lint match.
    #[arg(short = 'B', long = "before", value_parser = parse_context_line_count)]
    pub before: Option<u8>,
    /// Number of lines to show after the lint match.
    #[arg(short = 'A', long = "after", value_parser = parse_context_line_count)]
    pub after: Option<u8>,
    /// Number of lines to show before and after the lint match.
    #[arg(short = 'C', long = "context", value_parser = parse_context_line_count, conflicts_with_all = ["before", "after"])]
    pub context: Option<u8>,
    /// Color output and use syntax highlighting for code snippets.
    #[arg(long = "color", default_value_t = false)]
    pub color: bool,
    /// Print a list of available lints.
    #[arg(long, exclusive = true)]
    pub print_lints: bool,
    /// Increase verbosity (can be supplied multiple times).
    #[arg(short = 'v', long = "verbose", global = true, action = ArgAction::Count)]
    pub verbosity: u8,
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsString;
    use std::io::Write as _;

    use tempfile::NamedTempFile;

    fn try_parse<I, T>(srcs: I) -> Result<Args, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let args = [OsString::from("executable")]
            .into_iter()
            .chain(srcs.into_iter().map(T::into));
        Args::try_parse_from(args)
    }

    /// Make sure that we can recognize file list inputs as expected.
    #[test]
    fn source_file_parsing() {
        // Single file by path.
        let srcs = ["foobar"];
        let args = try_parse(srcs).unwrap();
        assert_eq!(args.srcs, vec![vec![PathBuf::from("foobar")]]);

        // Two files by path.
        let srcs = ["foo", "bar"];
        let args = try_parse(srcs).unwrap();
        assert_eq!(
            args.srcs,
            vec![vec![PathBuf::from("foo")], vec![PathBuf::from("bar")]]
        );

        // Single file containing paths.
        let mut file = NamedTempFile::new().unwrap();
        writeln!(&mut file, "1st").unwrap();
        writeln!(&mut file, "2nd").unwrap();
        let () = file.flush().unwrap();

        let srcs = [format!("@{}", file.path().display())];
        let args = try_parse(srcs).unwrap();
        assert_eq!(
            args.srcs,
            vec![vec![PathBuf::from("1st"), PathBuf::from("2nd")]]
        );

        // Regular path and file containing paths.
        let srcs = ["foobar", &format!("@{}", file.path().display())];
        let args = try_parse(srcs).unwrap();
        assert_eq!(
            args.srcs,
            vec![
                vec![PathBuf::from("foobar")],
                vec![PathBuf::from("1st"), PathBuf::from("2nd")]
            ]
        );
    }

    /// Test that -C cannot be combined with -A or -B using clap groups.
    #[test]
    fn context_conflict_validation() {
        // -C with -B should fail parsing (clap will reject it)
        assert!(try_parse(["test.c", "-C", "3", "-B", "2"]).is_err());

        // -C with -A should fail parsing (clap will reject it)
        assert!(try_parse(["test.c", "-C", "3", "-A", "4"]).is_err());

        // -C with both -A and -B should fail parsing (clap will reject it)
        assert!(try_parse(["test.c", "-C", "3", "-B", "2", "-A", "4"]).is_err());

        // -A and -B without -C should pass parsing
        assert!(try_parse(["test.c", "-B", "2", "-A", "4"]).is_ok());
    }

    /// Test `parse_context_line_count` function directly.
    #[test]
    fn parse_context_line_count_validation() {
        // Valid values
        assert_eq!(parse_context_line_count("0").unwrap(), 0);
        assert_eq!(parse_context_line_count("1").unwrap(), 1);
        assert_eq!(parse_context_line_count("255").unwrap(), 255);

        // Invalid values - out of range
        assert!(parse_context_line_count("256").is_err());
        assert!(parse_context_line_count("1000").is_err());
        assert!(parse_context_line_count("-1").is_err());

        // Invalid values - not numbers
        assert!(parse_context_line_count("abc").is_err());
        assert!(parse_context_line_count("").is_err());
    }
}
