use crate::content::parsing::ast_utils::extract_text_from_nodes;
use crate::core::markdown_config;
use markdown::mdast::Node;
use markdown::to_mdast;

pub struct ExcerptExtractor {
    parse_options: markdown::ParseOptions,
}

impl Default for ExcerptExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExcerptExtractor {
    pub fn new() -> Self {
        let parse_options = markdown_config::with_frontmatter_and_gfm();

        Self { parse_options }
    }

    /// Extract plain text excerpt from markdown AST (first paragraph)
    /// Strips all formatting, returns full paragraph text
    pub fn extract_excerpt_from_markdown(&self, markdown: &str) -> Option<String> {
        let root = to_mdast(markdown, &self.parse_options).ok()?;
        self.extract_first_paragraph_from_ast(&root)
    }

    /// Extract first paragraph text from AST, stripping formatting
    fn extract_first_paragraph_from_ast(&self, root: &Node) -> Option<String> {
        if let Node::Root(root_node) = root {
            for child in &root_node.children {
                if let Node::Paragraph(para) = child {
                    let text = extract_text_from_nodes(&para.children);
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
        None
    }

    /// Extract plain text excerpt from content (for non-markdown parsers)
    /// Splits by double newlines to find first non-empty paragraph
    pub fn extract_excerpt_from_plain_text(content: &str) -> Option<String> {
        // Split by double newlines and find first non-empty paragraph
        for para in content.split("\n\n") {
            let trimmed = para.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excerpt_extraction_from_markdown() {
        let content = r#"
# Title

This is the first paragraph with **bold** and *italic* text.

This is the second paragraph."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This is the first paragraph with bold and italic text.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_empty_content() {
        let content = "";
        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(excerpt, None);
    }

    #[test]
    fn test_excerpt_extraction_no_paragraphs() {
        let content = "# Just a heading";
        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(excerpt, None);
    }

    #[test]
    fn test_excerpt_extraction_with_code() {
        let content = r#"
# Title

This paragraph has `inline code` and **bold** text.

More content."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This paragraph has inline code and bold text.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_with_links() {
        let content = r#"
# Title

This paragraph has a [link](https://example.com) and more text.

More content."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This paragraph has a link and more text.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_complex_markdown() {
        let content = r#"
# Title

This paragraph has **bold**, *italic*, `code`, and [links](https://example.com) all mixed together.

Second paragraph here."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some(
                "This paragraph has bold, italic, code, and links all mixed together.".to_string()
            )
        );
    }

    #[test]
    fn test_excerpt_extraction_with_frontmatter() {
        let content = r#"+++
title = "Test Title"
+++

# First Title

This is the first paragraph that should be extracted as an excerpt.

This is the second paragraph."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This is the first paragraph that should be extracted as an excerpt.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_with_images() {
        let content = r#"
# Title

This paragraph has ![alt text](image.jpg) an image and text.

More content."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This paragraph has alt text an image and text.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_nested_formatting() {
        let content = r#"
# Title

This has **bold with *italic* inside** and `code` text.

More content."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This has bold with italic inside and code text.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_with_strikethrough() {
        let content = r#"
# Title

This has ~~strikethrough~~ and regular text.

More content."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This has strikethrough and regular text.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_whitespace_only_paragraph() {
        let content = r#"
# Title


   
This has actual content after empty paragraph.

More content."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This has actual content after empty paragraph.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_leading_whitespace() {
        let content = r#"
# Title
   
   This paragraph has leading whitespace.

More content."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This paragraph has leading whitespace.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_trailing_whitespace() {
        let content = r#"
# Title

This paragraph has trailing whitespace.   
   

More content."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This paragraph has trailing whitespace.".to_string())
        );
    }

    #[test]
    fn test_plain_text_excerpt_extraction() {
        assert_eq!(
            ExcerptExtractor::extract_excerpt_from_plain_text(
                "First paragraph.\n\nSecond paragraph."
            ),
            Some("First paragraph.".to_string())
        );
    }

    #[test]
    fn test_plain_text_excerpt_single_paragraph() {
        let content = "Just one paragraph without double newlines.";
        assert_eq!(
            ExcerptExtractor::extract_excerpt_from_plain_text(content),
            Some("Just one paragraph without double newlines.".to_string())
        );
    }

    #[test]
    fn test_plain_text_excerpt_empty() {
        assert_eq!(ExcerptExtractor::extract_excerpt_from_plain_text(""), None);
    }

    #[test]
    fn test_plain_text_excerpt_whitespace_only() {
        assert_eq!(
            ExcerptExtractor::extract_excerpt_from_plain_text("   \n\n   "),
            None
        );
    }

    #[test]
    fn test_plain_text_excerpt_single_newlines() {
        let content = "First paragraph.\nSecond line.\n\nThird paragraph.";
        assert_eq!(
            ExcerptExtractor::extract_excerpt_from_plain_text(content),
            Some("First paragraph.\nSecond line.".to_string())
        );
    }

    #[test]
    fn test_plain_text_excerpt_leading_whitespace() {
        let content = "   \n\nFirst paragraph after whitespace.";
        assert_eq!(
            ExcerptExtractor::extract_excerpt_from_plain_text(content),
            Some("First paragraph after whitespace.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_with_blockquote() {
        let content = r#"
# Title

> This is a blockquote
> with multiple lines.

This is the first real paragraph.

This is the second paragraph."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This is the first real paragraph.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_with_code_block() {
        let content = r#"
# Title

```rust
fn main() {
    println!("Hello");
}
```

This is the first paragraph after code block.

This is the second paragraph."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This is the first paragraph after code block.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_with_list() {
        let content = r#"
# Title

- First item
- Second item
- Third item

This is the first paragraph after list.

This is the second paragraph."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This is the first paragraph after list.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_complex_document() {
        let content = r#"+++
title = "Complex Document"
excerpt = "This should be ignored"
+++

# Document Title

> This is a quote
> with multiple lines

## Introduction

This is the first real paragraph with **bold** text and `inline code`.

- List item 1
- List item 2

This is the second paragraph with [a link](https://example.com).

## Conclusion

Final paragraph here."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This is the first real paragraph with bold text and inline code.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_unicode_content() {
        let content = r#"
# Заголовок

Это первый абзац с **жирным** текстом и *курсивом*.

Второй абзац здесь."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("Это первый абзац с жирным текстом и курсивом.".to_string())
        );
    }

    #[test]
    fn test_excerpt_extraction_mixed_content() {
        let content = r#"
# Title

First paragraph.

```code
Some code here
```

Second paragraph with **bold**."#;

        let extractor = ExcerptExtractor::new();
        let excerpt = extractor.extract_excerpt_from_markdown(content);
        assert_eq!(excerpt, Some("First paragraph.".to_string()));
    }
}
