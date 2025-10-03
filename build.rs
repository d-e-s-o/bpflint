//! Build script for `bpflint`.

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use anyhow::Error;
use anyhow::Result;
#[cfg(feature = "deploy")]
use anyhow::anyhow;


fn generate_lints(manifest_dir: &Path) -> Result<()> {
    let out_dir =
        env::var_os("OUT_DIR").context("failed to find `OUT_DIR` environment variable")?;
    let lints_rs_path = Path::new(&out_dir).join("lints.rs");
    let mut lints_rs_file = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&lints_rs_path)
        .with_context(|| format!("failed to open `{}` for writing", lints_rs_path.display()))?;
    let lint_dir = Path::new(&manifest_dir).join("lints");
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

        let lint_src = read_to_string(&lint_path)
            .with_context(|| format!("failed to read lint `{}`", lint_path.display()))?;
        let mut lint_msg_path = lint_path.clone();
        let _result = lint_msg_path.set_extension("txt");
        let lint_msg = read_to_string(&lint_msg_path).with_context(|| {
            format!("failed to read lint message `{}`", lint_msg_path.display())
        })?;
        let lint_name = entry.file_name();
        let lint_name = lint_name.to_str().with_context(|| {
            format!(
                "lint `{}` does not have valid UTF-8 name",
                lint_path.display()
            )
        })?;
        let lint_name = lint_name.trim_end_matches(".scm");
        let lint_name_upper = lint_name.to_ascii_uppercase().replace('-', "_");
        let lint_var = format!("LINT_{lint_name_upper}_SRC");
        writeln!(
            &mut lints_rs_file,
            r####"pub static {lint_var}: (&str, &str, &str) = (r###"{lint_name}"###, r###"{lint_src}"###, r###"{}"###);"####,
            lint_msg.trim_end_matches('\n'),
        )?;
        let () = lint_vars.push(lint_var);
    }

    writeln!(
        &mut lints_rs_file,
        r#"pub static LINTS: [(&str, &str, &str); {}] = ["#,
        lint_vars.len()
    )?;
    for lint_var in lint_vars {
        writeln!(&mut lints_rs_file, "    {lint_var},")?;
    }
    writeln!(&mut lints_rs_file, r#"];"#)?;
    Ok(())
}

/// Generate the final WASM bindings package in `output_dir` based on
/// the provided .wasm file supplied as `input_wasm`.
#[cfg(feature = "deploy")]
fn generate_pkg(input_wasm: &Path, output_dir: &Path, debug: bool) -> Result<()> {
    use wasm_bindgen_cli_support::Bindgen;

    // This invocation roughly maps to the following command (which would
    // require wasm-bindgen-cli installed):
    // $ wasm-bindgen --out-dir <output_dir> --target web <input_wasm>
    Bindgen::new()
        .input_path(input_wasm)
        .web(true)?
        .browser(false)?
        .debug(debug)
        .keep_debug(debug)
        .remove_name_section(!debug)
        .remove_producers_section(!debug)
        .typescript(false)
        .generate(output_dir)
}

#[cfg(feature = "deploy")]
fn deploy_package(manifest_dir: &Path) -> Result<()> {
    let name = env::var("CARGO_PKG_NAME")
        .context("failed to read CARGO_PKG_NAME variable")?
        .replace("-", "_");
    // Cargo's OUT_DIR is where it stores build artifacts and so that is
    // where we can find the created .*wasm we need.
    let out_dir = env::var_os("OUT_DIR").context("failed to read `OUT_DIR` variable")?;

    // OUT_DIR is something like
    // <some-dir>/target/<target>/debug/build/<name>-<hash>/out
    // but the generated .wasm resides in
    // <some-dir>/target/<target>/debug/deps/
    // directly. So pop the last three directories.
    let mut input_wasm = PathBuf::from(out_dir);
    input_wasm.pop();
    input_wasm.pop();
    input_wasm.pop();
    input_wasm.push("deps");
    input_wasm.push(name);
    input_wasm.set_extension("wasm");

    let mut output_dir = manifest_dir.to_path_buf();
    output_dir.push("www");
    output_dir.push("pkg");

    let debug = env::var("DEBUG").context("failed to read DEBUG variable")?;
    let debug = match debug.as_ref() {
        "true" => true,
        "false" => false,
        _ => {
            return Err(anyhow!(
                "encountered unexpected value in DEBUG variable: {debug}"
            ))
        },
    };

    generate_pkg(&input_wasm, &output_dir, debug)
        .context("failed to generate web assembly module")?;

    println!("cargo:rerun-if-changed={}", input_wasm.as_path().display());
    Ok(())
}

#[cfg(not(feature = "deploy"))]
fn deploy_package(_manifest_dir: &Path) -> Result<()> {
    unimplemented!()
}

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").context("failed to read CARGO_MANIFEST_DIR variable")?,
    );

    let () = generate_lints(&manifest_dir)?;

    if cfg!(feature = "deploy") {
        let () = deploy_package(&manifest_dir)?;
    }

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RUN");
    println!("cargo:rerun-if-env-changed=CARGO_MANIFEST_DIR");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_NAME");
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    Ok(())
}
