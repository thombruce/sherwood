use anyhow::Result;
use markdown::{Options, to_html_with_options};

/// Utility for parsers that need to convert markdown to HTML
pub struct MarkdownProcessor;

impl MarkdownProcessor {
    /// Convert markdown string to HTML with semantic enhancements
    pub fn process(markdown: &str) -> Result<String> {
        let options = Options::gfm();
        let html = to_html_with_options(markdown, &options)
            .map_err(|e| anyhow::anyhow!("Failed to process markdown: {}", e))?;
        Ok(Self::enhance_semantics(&html))
    }

    /// Simple semantic enhancements for markdown-derived HTML
    fn enhance_semantics(html: &str) -> String {
        // Basic semantic improvements (same as existing renderer)
        html.replace("<ul>", "<ul class=\"content-list\">")
            .replace("<ol>", "<ol class=\"numbered-list\">")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html_conversion() {
        let markdown = "# Hello\n\nThis is a test.\n\n- Item 1\n- Item 2";
        let result = MarkdownProcessor::process(markdown).unwrap();

        assert!(result.contains("<h1>Hello</h1>"));
        assert!(result.contains("<p>This is a test.</p>"));
        assert!(result.contains("<ul class=\"content-list\">"));
    }

    #[test]
    fn test_empty_markdown() {
        let markdown = "";
        let result = MarkdownProcessor::process(markdown).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_code_blocks_preserved() {
        let markdown = "```rust\nlet x = 1;\n```";
        let result = MarkdownProcessor::process(markdown).unwrap();
        assert!(result.contains("<pre>"));
        assert!(result.contains("<code"));
    }
}
