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


/// A command line interface for `bpflint`.
#[derive(Debug, Parser)]
#[command(version = env!("VERSION"))]
pub struct Args {
    /// The BPF C source files to lint.
    ///
    /// Use '@file' syntax to include a file list contained in 'file'.
    #[arg(required = true, value_name = "[@]SRCS", value_parser = parse_files)]
    pub srcs: Vec<Vec<PathBuf>>,
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


    /// Make sure that we can recognize file list inputs as expected.
    #[test]
    fn source_file_parsing() {
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
}
