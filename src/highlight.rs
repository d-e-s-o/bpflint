use anyhow::Result;

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
    use super::Highlighter;
    use anyhow::Result;
    use tree_sitter_highlight::Highlight;
    use tree_sitter_highlight::HighlightConfiguration;
    use tree_sitter_highlight::HighlightEvent;

    use super::NopHighlighter;

    struct TreeSitterHighlighter {
        highlight_config: tree_sitter_highlight::HighlightConfiguration,
    }

    impl TreeSitterHighlighter {
        pub fn new() -> Result<Self> {
            let c_language = tree_sitter_bpf_c::LANGUAGE.into();
            let mut highlight_config = tree_sitter_highlight::HighlightConfiguration::new(
                c_language,
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
        fn highlight(&self, code: &[u8]) -> anyhow::Result<String> {
            let mut highlighter = tree_sitter_highlight::Highlighter::new();
            let highlights = highlighter.highlight(&self.highlight_config, code, None, |_| None)?;
            let mut result = String::new();
            for event in highlights {
                match event.unwrap() {
                    HighlightEvent::Source { start, end } => {
                        result.push_str(&String::from_utf8_lossy(&code[start..end]));
                    },
                    HighlightEvent::HighlightStart(s) => {
                        result.push_str(&ansi_for_highlight(s, &self.highlight_config));
                    },
                    HighlightEvent::HighlightEnd => {
                        result.push_str(AnsiColor24::reset());
                    },
                }
            }
            Ok(result)
        }
    }

    pub fn create_highlighter(color: bool) -> Result<Box<dyn Highlighter>> {
        if !color {
            return Ok(Box::new(NopHighlighter));
        }

        TreeSitterHighlighter::new().map(|h| Box::new(h) as Box<dyn Highlighter>)
    }

    /// Represents a 24-bit (true color) ANSI color.
    /// Usage: emits \x1b[38;2;R;G;Bm for foreground color.
    #[derive(Copy, Clone, Debug)]
    struct AnsiColor24(pub u8, pub u8, pub u8);
    impl AnsiColor24 {
        /// Returns the ANSI escape code for this color (24-bit/true color).
        pub fn as_ansi_fg(&self) -> String {
            format!("\x1b[38;2;{};{};{}m", self.0, self.1, self.2)
        }
        /// Returns the ANSI reset code.
        pub fn reset() -> &'static str {
            "\x1b[0m"
        }
    }

    const GITHUB_PURPLE: AnsiColor24 = AnsiColor24(121, 93, 163); // #795da3
    const GITHUB_TEAL: AnsiColor24 = AnsiColor24(0, 134, 179); // #0086B3
    const GITHUB_PINK: AnsiColor24 = AnsiColor24(167, 29, 93); // #a71d5d
    const GITHUB_BLUE: AnsiColor24 = AnsiColor24(24, 54, 145); // #183691
    const GITHUB_GRAY: AnsiColor24 = AnsiColor24(150, 152, 150); // #969896
    const GITHUB_DARKGRAY: AnsiColor24 = AnsiColor24(51, 51, 51); // #333333

    /// Syntax highlight mapping for GitHub Sublime theme (24-bit colors)
    /// <https://github.com/AlexanderEkdahl/github-sublime-theme/blob/master/GitHub.tmTheme>
    static ANSI_HIGHLIGHT_ARRAY: [(&str, AnsiColor24); 15] = [
        ("function", GITHUB_PURPLE),
        ("function.builtin", GITHUB_TEAL),
        ("keyword", GITHUB_PINK),
        ("string", GITHUB_BLUE),
        ("comment", GITHUB_GRAY),
        ("type", GITHUB_PINK),
        ("constant", GITHUB_TEAL),
        ("variable", GITHUB_TEAL),
        ("number", GITHUB_TEAL),
        ("operator", GITHUB_PINK),
        ("attribute", GITHUB_PURPLE),
        ("property", GITHUB_TEAL),
        ("punctuation", GITHUB_DARKGRAY),
        ("macro", GITHUB_TEAL),
        ("namespace", GITHUB_DARKGRAY),
    ];
    /// A map of highlight group names to their corresponding ANSI color codes.
    ///
    /// If a highlight group name is not found in the map, it will return the ANSI color
    /// code reset.
    fn ansi_for_highlight(h: Highlight, highlight_config: &HighlightConfiguration) -> String {
        let group_name = *highlight_config.names().get(h.0).unwrap_or(&"unknown");
        ANSI_HIGHLIGHT_ARRAY
            .iter()
            .find(|(name, _)| *name == group_name)
            .map(|(_, color)| color.as_ansi_fg())
            .unwrap_or(AnsiColor24::reset().to_string())
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::Highlighter;
    use super::NopHighlighter;
    use anyhow::Result;

    pub fn create_highlighter(_color: bool) -> Result<Box<dyn Highlighter>> {
        // No-op highlighter for wasm
        return Ok(Box::new(NopHighlighter))
    }
}

// Re-export for use in your main code
pub use imp::create_highlighter;
