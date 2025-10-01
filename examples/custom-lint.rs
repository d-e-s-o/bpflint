//! An example illustrating the definition and usage of a custom lint.
//!
//! Run via:
//! ```sh
//! $ cargo run --example custom-lint
//! ```

use std::io::stdout;
use std::path::Path;

use indoc::indoc;

use bpflint::Lint;
use bpflint::lint_custom;
use bpflint::terminal::Opts;
use bpflint::terminal::report_opts;


fn main() {
    // Example lint flagging the usage of `bpf_ktime_get_ns`, just for
    // kicks.
    let lint = Lint {
        name: "bpf-stackid-usage".to_string(),
        code: indoc! { r#"
          (call_expression
              function: (identifier) @function (#eq? @function "bpf_get_stackid")
          )
      "# }
        .to_string(),
        message: "Please don't use bpf_get_stackid() in this example.".to_string(),
    };

    let code = include_bytes!("task_longrun.bpf.c");

    // At this point we are only interested in checking for our custom
    // `bpf-stackid-usage` lint. But we could also include the built-in
    // lints by chaining to `bpflint::builtin_lints`.
    let matches = lint_custom(code, &[lint]).expect("failed to lint");
    assert_eq!(matches.len(), 1);

    let opts = Opts {
        color: true,
        extra_lines: (2, 2),
        ..Default::default()
    };

    report_opts(
        &matches[0],
        code,
        Path::new("task_longrun.bpf.c"),
        &opts,
        &mut stdout().lock(),
    )
    .expect("failed to report matches");
}
