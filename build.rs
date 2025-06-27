//! Build script for `bpflint`.

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::fs::read_dir;
use std::io::Write as _;
use std::path::Path;

use anyhow::Context as _;
use anyhow::Error;
use anyhow::Result;
use anyhow::ensure;


fn is_alphanumeric_minus(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn generate_lints() -> Result<()> {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir =
        env::var_os("OUT_DIR").context("failed to find `OUT_DIR` environment variable")?;
    let lints_rs_path = Path::new(&out_dir).join("lints.rs");
    let mut lints_rs_file = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&lints_rs_path)
        .with_context(|| format!("failed to open `{}` for writing", lints_rs_path.display()))?;
    let lint_dir = Path::new(&dir).join("lints");
    println!("cargo::rerun-if-changed={}", lint_dir.display());

    let mut lint_vars = Vec::new();
    for result in read_dir(&lint_dir)
        .with_context(|| format!("failed to read directory `{}`", lint_dir.display()))?
    {
        let entry = result.map_err(Error::from)?;
        let lint_path = entry.path();
        if lint_path.extension() != Some(OsStr::new("scm")) {
            continue
        }

        let lint_name = entry.file_name();
        let lint_name = lint_name.to_str().with_context(|| {
            format!(
                "lint `{}` does not have valid UTF-8 name",
                lint_path.display()
            )
        })?;
        let lint_name = lint_name.trim_end_matches(".scm");
        // TODO: Should probably do some more sanity checking here.
        //       E.g., no `-` at start/end.
        ensure!(
            is_alphanumeric_minus(lint_name),
            "lint name `{lint_name}` contains unsupported characters"
        );

        let lint_name_upper = lint_name.to_ascii_uppercase().replace('-', "_");
        let lint_var = format!("LINT_{lint_name_upper}_SRC");
        writeln!(
            &mut lints_rs_file,
            r#"pub static {lint_var}: (&str, &str) = ("{}", include_str!("{}"));"#,
            lint_name,
            lint_path.display(),
        )?;
        let () = lint_vars.push(lint_var);
    }

    writeln!(
        &mut lints_rs_file,
        r#"pub static LINTS: [(&str, &str); {}] = ["#,
        lint_vars.len()
    )?;
    for lint_var in lint_vars {
        writeln!(&mut lints_rs_file, "    {lint_var},")?;
    }
    writeln!(&mut lints_rs_file, r#"];"#)?;
    Ok(())
}

fn main() -> Result<()> {
    let () = generate_lints()?;
    Ok(())
}
