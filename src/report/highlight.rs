use anyhow::Result;

use super::ansi_color::COLOR_BLUE;
use super::ansi_color::COLOR_DARKGRAY;
use super::ansi_color::COLOR_GRAY;
use super::ansi_color::COLOR_PINK;
use super::ansi_color::COLOR_PURPLE;
use super::ansi_color::COLOR_RESET;
use super::ansi_color::COLOR_TEAL;


pub(crate) trait Highlighter {
    fn highlight(&self, code: &[u8]) -> Result<String>;
}


struct NopHighlighter;

impl Highlighter for NopHighlighter {
    fn highlight(&self, code: &[u8]) -> Result<String> {
        Ok(String::from_utf8_lossy(code).to_string())
    }
}


#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::*;

    use tree_sitter_bpf_c::LANGUAGE;
    use tree_sitter_highlight::Highlight;
    use tree_sitter_highlight::HighlightConfiguration;
    use tree_sitter_highlight::HighlightEvent;
    use tree_sitter_highlight::Highlighter as TsHighlighter;


    struct TreeSitterHighlighter {
        highlight_config: HighlightConfiguration,
    }

    impl TreeSitterHighlighter {
        fn new() -> Result<Self> {
            let mut highlight_config = HighlightConfiguration::new(
                LANGUAGE.into(),
                "bpf-c",
                tree_sitter_bpf_c::HIGHLIGHTS_QUERY,
                "",
                "",
            )?;
            highlight_config.configure(
                &ANSI_HIGHLIGHT_ARRAY
                    .iter()
                    .map(|(name, _)| *name)
                    .collect::<Vec<&str>>(),
            );
            Ok(Self { highlight_config })
        }
    }

    impl Highlighter for TreeSitterHighlighter {
        fn highlight(&self, code: &[u8]) -> Result<String> {
            let mut highlighter = TsHighlighter::new();
            let highlights = highlighter.highlight(&self.highlight_config, code, None, |_| None)?;
            let mut result = String::new();
            for event in highlights {
                match event.unwrap() {
                    HighlightEvent::Source { start, end } => {
                        result.push_str(&String::from_utf8_lossy(&code[start..end]));
                    },
                    HighlightEvent::HighlightStart(s) => {
                        result.push_str(ansi_for_highlight(s, &self.highlight_config));
                    },
                    HighlightEvent::HighlightEnd => {
                        result.push_str(COLOR_RESET);
                    },
                }
            }
            Ok(result)
        }
    }

    pub(crate) fn create_highlighter(color: bool) -> Result<Box<dyn Highlighter>> {
        if !color {
            return Ok(Box::new(NopHighlighter));
        }

        TreeSitterHighlighter::new().map(|h| Box::new(h) as Box<dyn Highlighter>)
    }


    /// Syntax highlight mapping for GitHub Sublime theme (24-bit colors)
    /// <https://github.com/AlexanderEkdahl/github-sublime-theme/blob/master/GitHub.tmTheme>
    static ANSI_HIGHLIGHT_ARRAY: [(&str, &str); 15] = [
        ("function", COLOR_PURPLE),
        ("function.builtin", COLOR_TEAL),
        ("keyword", COLOR_PINK),
        ("string", COLOR_BLUE),
        ("comment", COLOR_GRAY),
        ("type", COLOR_PINK),
        ("constant", COLOR_TEAL),
        ("variable", COLOR_TEAL),
        ("number", COLOR_TEAL),
        ("operator", COLOR_PINK),
        ("attribute", COLOR_PURPLE),
        ("property", COLOR_TEAL),
        ("punctuation", COLOR_DARKGRAY),
        ("macro", COLOR_TEAL),
        ("namespace", COLOR_DARKGRAY),
    ];

    /// A map of highlight group names to their corresponding ANSI color codes.
    ///
    /// If a highlight group name is not found in the map, it will return the ANSI color
    /// code reset.
    fn ansi_for_highlight(h: Highlight, highlight_config: &HighlightConfiguration) -> &'static str {
        let group_name = *highlight_config.names().get(h.0).unwrap_or(&"unknown");
        ANSI_HIGHLIGHT_ARRAY
            .iter()
            .find(|(name, _)| *name == group_name)
            .map(|(_, color_str)| *color_str)
            .unwrap_or(COLOR_RESET)
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::*;

    pub(crate) fn create_highlighter(_color: bool) -> Result<Box<dyn Highlighter>> {
        // No-op highlighter for wasm
        return Ok(Box::new(NopHighlighter))
    }
}

// Re-export for use in your main code
pub(crate) use imp::create_highlighter;
