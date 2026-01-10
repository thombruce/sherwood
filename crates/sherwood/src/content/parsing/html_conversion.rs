use anyhow::Result;
use markdown::{Options, to_html_with_options};

pub struct HtmlConverter {
    options: Options,
}

impl Default for HtmlConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlConverter {
    pub fn new() -> Self {
        let options = Options::gfm(); // GFM includes strikethrough, tables, footnotes
        Self { options }
    }

    /// Convert markdown string to HTML with semantic enhancements
    pub fn convert_markdown_to_html(&self, markdown: &str) -> Result<String> {
        let html_output = to_html_with_options(markdown, &self.options)
            .map_err(|e| anyhow::anyhow!("Failed to parse markdown: {}", e))?;

        Ok(enhance_semantics(&html_output))
    }
}

/// Apply semantic enhancements to HTML content (moved from renderer)
fn enhance_semantics(html: &str) -> String {
    let mut enhanced = html.to_string();

    // Wrap paragraphs in semantic sections if they seem like articles
    enhanced = wrap_articles(&enhanced);

    // Add semantic structure to lists
    enhanced = enhance_lists(&enhanced);

    enhanced
}

/// Wrap paragraphs in semantic sections if they seem like articles
fn wrap_articles(html: &str) -> String {
    // Simple heuristic: if content has multiple headings, wrap in article tags
    let heading_count = html.matches("<h").count();
    if heading_count > 1 {
        format!("<article>\n{}\n</article>", html)
    } else {
        html.to_string()
    }
}

/// Add semantic structure to lists
fn enhance_lists(html: &str) -> String {
    // Convert plain lists to more semantic versions when appropriate
    html.replace("<ul>", "<ul class=\"content-list\">")
        .replace("<ol>", "<ol class=\"numbered-list\">")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_markdown_to_html_simple() {
        let converter = HtmlConverter::new();
        let markdown = "# Hello World\n\nThis is a test.";

        let result = converter.convert_markdown_to_html(markdown).unwrap();

        assert!(result.contains("<h1>Hello World</h1>"));
        assert!(result.contains("<p>This is a test.</p>"));
    }

    #[test]
    fn test_convert_markdown_to_html_with_gfm_features() {
        let converter = HtmlConverter::new();
        let markdown = "~~strikethrough~~ and `code`";

        let result = converter.convert_markdown_to_html(markdown).unwrap();

        assert!(result.contains("<del>strikethrough</del>"));
        assert!(result.contains("<code>code</code>"));
    }

    #[test]
    fn test_enhance_semantics_wrap_articles() {
        let html = "<h1>First</h1>\n<p>Content</p>\n<h2>Second</h2>\n<p>More</p>";
        let result = enhance_semantics(html);

        assert!(result.contains("<article>"));
        assert!(result.contains("</article>"));
    }

    #[test]
    fn test_enhance_semantics_no_wrap_single_heading() {
        let html = "<h1>Only Heading</h1>\n<p>Content</p>";
        let result = enhance_semantics(html);

        assert!(!result.contains("<article>"));
        assert!(!result.contains("</article>"));
        assert_eq!(result, html);
    }

    #[test]
    fn test_enhance_lists_unordered() {
        let html = "<ul>\n<li>Item 1</li>\n<li>Item 2</li>\n</ul>";
        let result = enhance_lists(html);

        assert!(result.contains("<ul class=\"content-list\">"));
        assert!(result.contains("<li>Item 1</li>"));
        assert!(result.contains("<li>Item 2</li>"));
    }

    #[test]
    fn test_enhance_lists_ordered() {
        let html = "<ol>\n<li>First</li>\n<li>Second</li>\n</ol>";
        let result = enhance_lists(html);

        assert!(result.contains("<ol class=\"numbered-list\">"));
        assert!(result.contains("<li>First</li>"));
        assert!(result.contains("<li>Second</li>"));
    }

    #[test]
    fn test_enhance_lists_no_lists() {
        let html = "<p>No lists here</p>";
        let result = enhance_lists(html);

        assert_eq!(result, html);
    }

    #[test]
    fn test_enhance_lists_nested_lists() {
        let html = "<ul><li>Outer<ul><li>Nested</li></ul></li></ul>";
        let result = enhance_lists(html);

        assert!(result.contains("<ul class=\"content-list\"><li>Outer"));
        assert!(result.contains("<ul><li>Nested</li></ul>"));
    }

    #[test]
    fn test_conversion_error_handling() {
        let converter = HtmlConverter::new();
        // This should work fine - just testing it doesn't crash
        let result = converter.convert_markdown_to_html("Simple text");
        assert!(result.is_ok());
    }

    #[test]
    fn test_conversion_empty_content() {
        let converter = HtmlConverter::new();
        let result = converter.convert_markdown_to_html("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_conversion_only_whitespace() {
        let converter = HtmlConverter::new();
        let result = converter.convert_markdown_to_html("   \n  \n  ").unwrap();
        assert_eq!(result.trim(), "");
    }

    #[test]
    fn test_conversion_with_blockquotes() {
        let converter = HtmlConverter::new();
        let markdown = "> This is a quote";

        let result = converter.convert_markdown_to_html(markdown).unwrap();

        assert!(result.contains("<blockquote>"));
        assert!(result.contains("This is a quote"));
    }

    #[test]
    fn test_conversion_with_images() {
        let converter = HtmlConverter::new();
        let markdown = "![Alt text](/image.jpg)";

        let result = converter.convert_markdown_to_html(markdown).unwrap();

        assert!(result.contains("<img"));
        assert!(result.contains("alt=\"Alt text\""));
        assert!(result.contains("src=\"/image.jpg\""));
    }

    #[test]
    fn test_conversion_with_links() {
        let converter = HtmlConverter::new();
        let markdown = "[Link text](https://example.com)";

        let result = converter.convert_markdown_to_html(markdown).unwrap();

        assert!(result.contains("<a"));
        assert!(result.contains("href=\"https://example.com\""));
        assert!(result.contains("Link text"));
    }

    #[test]
    fn test_conversion_with_emphasis() {
        let converter = HtmlConverter::new();
        let markdown = "This has *italic* and **bold** text";

        let result = converter.convert_markdown_to_html(markdown).unwrap();

        assert!(result.contains("<em>italic</em>"));
        assert!(result.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_conversion_with_inline_code() {
        let converter = HtmlConverter::new();
        let markdown = "Use `printf()` function in C";

        let result = converter.convert_markdown_to_html(markdown).unwrap();

        assert!(result.contains("<code>printf()</code>"));
    }

    #[test]
    fn test_convert_markdown_to_html_with_tables() {
        let converter = HtmlConverter::new();
        let markdown = "| Header | Value |\n|--------|-------|\n| Cell 1 | Cell 2 |";

        let result = converter.convert_markdown_to_html(markdown).unwrap();

        assert!(result.contains("<table>"));
        assert!(result.contains("<th>Header</th>"));
        assert!(result.contains("<td>Cell 1</td>"));
        assert!(result.contains("<td>Cell 2</td>"));
    }

    #[test]
    fn test_convert_markdown_to_html_with_footnotes() {
        let converter = HtmlConverter::new();
        let markdown = "Here's a footnote reference.[^1]\n\n[^1]: This is footnote content.";

        let result = converter.convert_markdown_to_html(markdown).unwrap();

        assert!(result.contains("<sup>"));
        assert!(result.contains("This is footnote content"));
    }

    #[test]
    fn test_full_conversion_workflow() {
        let converter = HtmlConverter::new();
        let markdown = r#"# Article Title

## Section 1

This is the first section with a list:
- Item 1
- Item 2

## Section 2

This is the second section with a numbered list:
1. First item
2. Second item

And some ~~strikethrough~~ text with `inline code`."#;

        let result = converter.convert_markdown_to_html(markdown).unwrap();

        // Should have article wrapper due to multiple headings
        assert!(result.contains("<article>"));
        assert!(result.contains("</article>"));

        // Should have headings
        assert!(result.contains("<h1>Article Title</h1>"));
        assert!(result.contains("<h2>Section 1</h2>"));
        assert!(result.contains("<h2>Section 2</h2>"));

        // Should have enhanced lists
        assert!(result.contains("<ul class=\"content-list\">"));
        assert!(result.contains("<ol class=\"numbered-list\">"));

        // Should have GFM features
        assert!(result.contains("<del>strikethrough</del>"));
        assert!(result.contains("<code>inline code</code>"));

        // Should have paragraphs
        assert!(result.contains("<p>This is the first section"));
        assert!(result.contains("<p>This is the second section"));
    }
}
