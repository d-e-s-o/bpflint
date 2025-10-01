use std::str;

use anyhow::Context as _;
use anyhow::Result;

use tracing::warn;

use tree_sitter::Node;
use tree_sitter::Parser;
use tree_sitter::Query;
use tree_sitter::QueryCursor;
use tree_sitter::Tree;
use tree_sitter_bpf_c::LANGUAGE;

use crate::Point;
use crate::Range;


mod lints {
    include!(concat!(env!("OUT_DIR"), "/lints.rs"));
}

impl From<tree_sitter::Point> for Point {
    fn from(other: tree_sitter::Point) -> Self {
        let tree_sitter::Point { row, column } = other;
        Self { row, col: column }
    }
}

impl From<tree_sitter::Range> for Range {
    fn from(other: tree_sitter::Range) -> Self {
        let tree_sitter::Range {
            start_byte,
            end_byte,
            start_point,
            end_point,
        } = other;
        Self {
            bytes: start_byte..end_byte,
            start_point: Point::from(start_point),
            end_point: Point::from(end_point),
        }
    }
}


/// A lint.
#[derive(Clone, Debug)]
pub struct Lint {
    /// The lint's name.
    pub name: String,
    /// The lints source code in the form of a [tree-sitter
    /// query][query].
    ///
    /// [query]: https://tree-sitter.github.io/tree-sitter/using-parsers/queries/
    pub code: String,
    /// The message reported in a [`LintMatch`][LintMatch::message].
    pub message: String,
}

impl AsRef<Lint> for Lint {
    #[inline]
    fn as_ref(&self) -> &Lint {
        self
    }
}


/// Retrieve the list of lints shipped with the library.
pub fn builtin_lints() -> impl ExactSizeIterator<Item = Lint> + DoubleEndedIterator {
    lints::LINTS.iter().map(|(name, code, message)| Lint {
        name: name.to_string(),
        code: code.to_string(),
        message: message.to_string(),
    })
}


/// Details about a lint match.
#[derive(Clone, Debug)]
pub struct LintMatch {
    /// The name of the lint that matched.
    pub lint_name: String,
    /// The lint's message.
    pub message: String,
    /// The code range that triggered the lint.
    pub range: Range,
}


/// Walk the syntax tree, checking if a comment node that disable the
/// given lint is present.
fn is_lint_disabled(lint_name: &str, mut node: Node, code: &[u8]) -> bool {
    loop {
        // Walk all previous siblings of the current node.
        if let Some(s) = node.prev_sibling() {
            if s.kind() == "comment" {
                let comment = &code[s.start_byte()..s.end_byte()];
                if let Ok(comment) = str::from_utf8(comment) {
                    // The comment node will still contain the actual
                    // comment syntax, unfortunately.
                    let comment = comment.trim_start_matches("//");
                    let comment = comment.trim_start_matches("/*");
                    let comment = comment.trim_end_matches("*/");
                    let comment = comment.trim();

                    if let Some(comment) = comment.strip_prefix("bpflint:") {
                        let directive = comment.trim();
                        match directive.strip_prefix("disable=") {
                            Some("all") => break true,
                            Some(disable) if disable == lint_name => break true,
                            _ => (),
                        }
                    }
                } else {
                    // If it's not valid UTF-8 it can't be a comment for
                    // us to consider.
                    warn!(
                        "encountered invalid UTF-8 in code comment at bytes `{}..{}`",
                        s.start_byte(),
                        s.end_byte()
                    );
                }
            }
        }

        // Move one level up and repeat.
        match node.parent() {
            Some(parent) => node = parent,
            None => break false,
        }
    }
}


fn lint_impl(tree: &Tree, code: &[u8], lint: &Lint) -> Result<Vec<LintMatch>> {
    let Lint {
        name: lint_name,
        code: lint_src,
        message: lint_msg,
    } = lint;

    let query =
        Query::new(&LANGUAGE.into(), lint_src).with_context(|| "failed to compile lint query")?;
    let mut query_cursor = QueryCursor::new();
    let mut results = Vec::new();
    let matches = query_cursor.matches(&query, tree.root_node(), code);
    for m in matches {
        for capture in m.captures {
            if is_lint_disabled(lint_name, capture.node, code) {
                continue;
            }

            // SANITY: It would be a tree-sitter bug if the capture
            //         index does not map to a valid capture name.
            let capture_name = query.capture_names()[capture.index as usize];
            // Captures starting with double underscore are considered
            // internal to the lint and are not reported.
            if capture_name.starts_with("__") {
                continue
            }

            let r#match = LintMatch {
                lint_name: lint_name.to_string(),
                message: lint_msg.to_string(),
                range: Range::from(capture.node.range()),
            };
            let () = results.push(r#match);
        }
    }

    if query_cursor.did_exceed_match_limit() {
        warn!("query exceeded maximum number of in-progress captures");
    }
    Ok(results)
}


/// Lint code using the provided set of lints.
///
/// Matches are reported in source code order.
///
/// - `code` is the source code in question, for example as read from a
///   file
/// - `lints` the lints to use for linting the provided source code
///
/// # Examples
/// ```rust
/// # use bpflint::builtin_lints;
/// # use bpflint::lint_custom;
/// # use bpflint::Lint;
/// let bpf_printk = Lint {
///     name: "bpf_printk-usage".to_string(),
///     code: r#"
///         (call_expression
///             function: (identifier) @function (#eq? @function "bpf_printk")
///         )
///       "#.to_string(),
///     message: "use bpf_printk only for debugging!".to_string(),
/// };
///
/// let code = br#"
///     SEC("tp_btf/sched_switch")
///     int handle__sched_switch(u64 *ctx) {
///         bpf_printk("context %p\n", ctx);
///         return 0;
///     }
/// "#;
///
/// // We want to include the built-in lints as well, not just our
/// // `bpf_printk` usage flagger.
/// let matches = lint_custom(code, builtin_lints().chain([bpf_printk])).unwrap();
/// assert_eq!(matches.len(), 1);
/// ```
pub fn lint_custom<'l, I, L>(code: &[u8], lints: I) -> Result<Vec<LintMatch>>
where
    I: IntoIterator<Item = L>,
    L: AsRef<Lint> + 'l,
{
    let mut parser = Parser::new();
    let () = parser
        .set_language(&LANGUAGE.into())
        .context("failed to load BPF C language parser")?;
    let tree = parser
        .parse(code, None)
        .context("failed to provided source code")?;
    let mut results = Vec::new();
    for lint in lints {
        let matches = lint_impl(&tree, code, lint.as_ref())?;
        let () = results.extend(matches);
    }

    // Sort results to ensure more consistent reporting with ascending
    // lines.
    let () = results.sort_by(|match1, match2| {
        // NB: We use an ad-hoc comparison rather than a proper
        // `PartialOrd` impl for `Range`, because the latter is a bit
        // harder to do correctly.
        match1
            .range
            .start_point
            .cmp(&match2.range.start_point)
            .then_with(|| match1.range.end_point.cmp(&match2.range.end_point))
    });
    Ok(results)
}

/// Lint code using the default ([built-in][builtin_lints]) set of lints.
///
/// Matches are reported in source code order.
///
/// - `code` is the source code in question, for example as read from a
///   file
pub fn lint(code: &[u8]) -> Result<Vec<LintMatch>> {
    lint_custom(code, builtin_lints())
}


#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    use crate::Point;


    fn lint_foo() -> Lint {
        Lint {
            name: "foo".to_string(),
            code: indoc! { r#"
                (call_expression
                    function: (identifier) @function (#eq? @function "foo")
                )
            "# }
            .to_string(),
            message: "foo".to_string(),
        }
    }


    /// Make sure that internal captures (named as "__xxx") are not
    /// reported as matches.
    #[test]
    fn internal_capture_reporting() {
        let code = indoc! { r#"
            bar();
        "# };
        let lint = Lint {
            name: "bar".to_string(),
            code: indoc! { r#"
                (call_expression
                    function: (identifier) @__function (#eq? @__function "bar")
                )
            "# }
            .to_string(),
            message: "a message".to_string(),
        };
        let matches = lint_custom(code.as_bytes(), [lint]).unwrap();
        assert!(matches.is_empty(), "{matches:?}");
    }

    /// Check that our built-in lints exhibit the expected set of
    /// properties.
    #[test]
    fn validate_lints() {
        for lint in builtin_lints() {
            let Lint {
                name,
                code,
                message,
            } = lint;
            let query = Query::new(&LANGUAGE.into(), &code).unwrap();
            assert_eq!(
                query.pattern_count(),
                1,
                "lint `{name}` has too many pattern matches: only a single one is supported currently"
            );

            let last = message.chars().last().unwrap();
            assert!(
                !['.', '!', '?', '\n'].contains(&last),
                "`message` property of lint `{name}` should be concise and not a fully blown sentence with punctuation"
            );
        }
    }

    /// Check that some basic linting works as expected.
    #[test]
    fn basic_linting() {
        let code = indoc! { r#"
            /* A handler for something */
            SEC("tp_btf/sched_switch")
            int handle__sched_switch(u64 *ctx)
            {
                struct task_struct *prev = (struct task_struct *)ctx[1];
                struct event event = {0};
                bpf_probe_read(event.comm, TASK_COMM_LEN, prev->comm);
                return 0;
            }
        "# };

        let matches = lint(code.as_bytes()).unwrap();
        assert_eq!(matches.len(), 1);

        let LintMatch {
            lint_name,
            message,
            range,
        } = &matches[0];
        assert_eq!(lint_name, "probe-read");
        assert!(
            message.starts_with("bpf_probe_read() is deprecated"),
            "{message}"
        );
        assert_eq!(&code[range.bytes.clone()], "bpf_probe_read");
        assert_eq!(range.start_point, Point { row: 6, col: 4 });
        assert_eq!(range.end_point, Point { row: 6, col: 18 });
    }

    /// Check that reported matches are sorted by line number.
    #[test]
    fn sorted_match_reporting() {
        let code = indoc! { r#"
            bar();
            foo();
        "# };
        let lint = Lint {
            name: "bar".to_string(),
            code: indoc! { r#"
                (call_expression
                    function: (identifier) @function (#eq? @function "bar")
                )
            "# }
            .to_string(),
            message: "bar".to_string(),
        };
        let matches = lint_custom(code.as_bytes(), [lint_foo(), lint]).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].lint_name, "bar");
        assert_eq!(matches[1].lint_name, "foo");
    }

    /// Check that we can disable lints by name for a given statement.
    #[test]
    fn lint_disabling() {
        let code = indoc! { r#"
            /* bpflint: disable=foo */
            foo();
            // bpflint: disable=foo
            foo();
            // bpflint: disable=all
            foo();
        "# };
        let matches = lint_custom(code.as_bytes(), [lint_foo()]).unwrap();
        assert_eq!(matches.len(), 0, "{matches:?}");
    }

    /// Check that we can disable lints by name for a given block.
    #[test]
    fn lint_disabling_recursive() {
        let code = indoc! { r#"
            /* bpflint: disable=foo */
            {
                {
                    foo();
                }
            }
        "# };
        let matches = lint_custom(code.as_bytes(), [lint_foo()]).unwrap();
        assert_eq!(matches.len(), 0, "{matches:?}");

        let code = indoc! { r#"
            /* bpflint: disable=foo */
            void test_fn(void) {
                foo();
            }
        "# };
        let matches = lint_custom(code.as_bytes(), [lint_foo()]).unwrap();
        assert_eq!(matches.len(), 0, "{matches:?}");
    }

    /// Check that erroneous disabling syntax is not accidentally recognized.
    #[test]
    fn lint_invalid_disabling() {
        let code = indoc! { r#"
            /* bpflint: disabled=foo */
            foo();
            /* disabled=foo */
            foo();
            // disabled=foo
            foo();
            // bpflint: foo
            foo();
            // bpflint: disable=bar
            foo();

            void test_fn(void) {
                /* bpflint: disable=foo */
                foobar();
                foo();
            }
        "# };
        let matches = lint_custom(code.as_bytes(), [lint_foo()]).unwrap();
        assert_eq!(matches.len(), 6, "{matches:?}");
    }
}
