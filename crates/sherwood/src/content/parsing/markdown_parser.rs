use anyhow::Result;
use markdown::to_mdast;
use std::path::Path;

use crate::content::parsing::excerpt_extraction::ExcerptExtractor;
use crate::content::parsing::frontmatter_parsing::{Frontmatter, FrontmatterParser};
use crate::content::parsing::html_conversion::HtmlConverter;
use crate::content::parsing::title_extraction::resolve_title;
use crate::core::markdown_config;

#[derive(Debug, Clone)]
pub struct MarkdownFile {
    pub path: std::path::PathBuf,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub title: String,
}

pub struct MarkdownParser {
    frontmatter_parser: FrontmatterParser,
    excerpt_extractor: ExcerptExtractor,
    html_converter: HtmlConverter,
    parse_options: markdown::ParseOptions,
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    pub fn new() -> Self {
        let parse_options = markdown_config::with_frontmatter();

        Self {
            frontmatter_parser: FrontmatterParser::new(),
            excerpt_extractor: ExcerptExtractor::new(),
            html_converter: HtmlConverter::new(),
            parse_options,
        }
    }

    /// Parse a markdown file from disk
    pub fn parse_markdown_file(file_path: &Path) -> Result<MarkdownFile> {
        let content = std::fs::read_to_string(file_path)?;
        let parser = Self::new();
        parser.parse_content(&content, file_path)
    }

    /// Parse markdown content string
    fn parse_content(&self, content: &str, file_path: &Path) -> Result<MarkdownFile> {
        // Parse AST once for both frontmatter and title extraction
        let root = to_mdast(content, &self.parse_options)
            .map_err(|e| anyhow::anyhow!("Failed to parse markdown: {}", e))?;

        // Extract frontmatter and clean content using AST
        let (mut frontmatter, markdown_content) = match &root {
            markdown::mdast::Node::Root(root_node) => self
                .frontmatter_parser
                .extract_frontmatter_from_root(root_node, content),
            _ => Ok((Frontmatter::default(), content.to_string())),
        }?;

        // Extract title from frontmatter, first h1 from AST, or filename
        let title = resolve_title(frontmatter.title.clone(), &root, file_path);

        // Extract excerpt if not present in frontmatter
        if frontmatter.excerpt.is_none() {
            frontmatter.excerpt = self
                .excerpt_extractor
                .extract_excerpt_from_markdown(&markdown_content);
        }

        // Convert markdown content to HTML immediately
        let html_content = self
            .html_converter
            .convert_markdown_to_html(&markdown_content)?;

        Ok(MarkdownFile {
            path: file_path.to_path_buf(),
            content: html_content, // Now always HTML
            frontmatter,
            title,
        })
    }

    /// Parse frontmatter only (for legacy compatibility)
    pub fn parse_frontmatter(&self, content: &str) -> Result<(Frontmatter, String)> {
        self.frontmatter_parser.parse_frontmatter(content)
    }

    /// Extract excerpt from markdown content
    pub fn extract_excerpt_from_markdown(&self, markdown: &str) -> Option<String> {
        self.excerpt_extractor
            .extract_excerpt_from_markdown(markdown)
    }

    /// Extract excerpt from plain text content
    pub fn extract_excerpt_from_plain_text(content: &str) -> Option<String> {
        ExcerptExtractor::extract_excerpt_from_plain_text(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_markdown_file_parsing() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"+++
title = "File Test"
date = "2024-01-20"
+++

# Test File

This is a test file."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "File Test");
        assert_eq!(result.frontmatter.title, Some("File Test".to_string()));
        assert_eq!(result.frontmatter.date, Some("2024-01-20".to_string()));
        // Now content is HTML, not markdown
        assert!(result.content.contains("<h1>Test File</h1>"));
        assert!(result.content.contains("<p>This is a test file.</p>"));
        assert_eq!(result.path, file_path);

        Ok(())
    }

    #[test]
    fn test_title_extraction_from_h1() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"
# Simple Title

This content has no frontmatter title."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "Simple Title");
        assert_eq!(result.frontmatter.title, None);
        // Content should be HTML now
        assert!(result.content.contains("<h1>Simple Title</h1>"));

        Ok(())
    }

    #[test]
    fn test_title_extraction_from_filename() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("my-filename.md");

        let content = r#"Some content without H1 heading."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "my-filename");

        Ok(())
    }

    #[test]
    fn test_title_priority() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"+++
title = "Frontmatter Title"
+++

# H1 Title

Content."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        // Frontmatter title should have highest priority
        assert_eq!(result.title, "Frontmatter Title");

        Ok(())
    }

    #[test]
    fn test_excerpt_parsing_with_frontmatter() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"+++
title = "Test Title"
+++

# First Title

This is the first paragraph that should be extracted as an excerpt.

This is the second paragraph."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(
            result.frontmatter.excerpt,
            Some("This is the first paragraph that should be extracted as an excerpt.".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_excerpt_priority_frontmatter_over_extraction() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"+++
title = "Test Title"
excerpt = "Custom excerpt from frontmatter"
+++

# First Title

This is the first paragraph that should NOT be extracted because frontmatter has an excerpt.

This is the second paragraph."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(
            result.frontmatter.excerpt,
            Some("Custom excerpt from frontmatter".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_complex_markdown_parsing() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("complex.md");

        let content = r#"+++
title = "Complex Document"
date = "2024-01-20"
list = true
tags = ["rust", "markdown"]
+++

# Introduction

This is a **complex** document with *multiple* formatting options.

## Section 2

Here's a list:
- Item 1
- Item 2
- Item 3

And a numbered list:
1. First
2. Second

## Code Examples

```rust
fn main() {
    println!("Hello, world!");
}
```

## Links and Images

Visit [Rust website](https://rust-lang.org) for more info.

![Rust Logo](https://rust-lang.org/static/images/rust-logo-256x256.png)

## Conclusion

This document demonstrates all major markdown features."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        // Check frontmatter
        assert_eq!(
            result.frontmatter.title,
            Some("Complex Document".to_string())
        );
        assert_eq!(result.frontmatter.date, Some("2024-01-20".to_string()));
        assert_eq!(result.frontmatter.list, Some(true));
        assert_eq!(
            result.frontmatter.tags,
            Some(vec!["rust".to_string(), "markdown".to_string()])
        );

        // Check title extraction
        assert_eq!(result.title, "Complex Document");

        // Check HTML content contains expected elements
        assert!(result.content.contains("<article>")); // Multiple headings
        assert!(result.content.contains("<h1>Introduction</h1>"));
        assert!(result.content.contains("<h2>Section 2</h2>"));
        assert!(result.content.contains("<strong>complex</strong>"));
        assert!(result.content.contains("<em>multiple</em>"));
        assert!(result.content.contains("<ul class=\"content-list\">"));
        assert!(result.content.contains("<ol class=\"numbered-list\">"));
        assert!(result.content.contains("<pre><code")); // Allow for class attribute
        assert!(result.content.contains("println!"));
        assert!(result.content.contains("<a href=\"https://rust-lang.org\""));
        assert!(
            result
                .content
                .contains("<img src=\"https://rust-lang.org/static/images/rust-logo-256x256.png\"")
        );

        Ok(())
    }

    #[test]
    fn test_minimal_markdown_parsing() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("minimal.md");

        let content = "Just some simple text without any special formatting.";

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        // Check defaults
        assert_eq!(result.frontmatter.title, None);
        assert_eq!(result.frontmatter.date, None);
        assert_eq!(result.frontmatter.list, None);
        assert_eq!(result.frontmatter.tags, None);
        // Excerpt should be auto-extracted from content
        assert_eq!(
            result.frontmatter.excerpt,
            Some("Just some simple text without any special formatting.".to_string())
        );

        // Title should be from filename
        assert_eq!(result.title, "minimal");

        // Content should be simple paragraph (may or may not be wrapped in p tags)
        let has_paragraph = result
            .content
            .contains("<p>Just some simple text without any special formatting.</p>")
            || result
                .content
                .contains("Just some simple text without any special formatting.");
        assert!(has_paragraph, "Content: {}", result.content);

        Ok(())
    }

    #[test]
    fn test_markdown_with_gfm_features() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("gfm.md");

        let content = r#"+++
title = "GFM Features"
+++

# GitHub Flavored Markdown

This document tests GFM features:

## Strikethrough
This text is ~~strikethrough~~.

## Tables
| Feature | Supported |
|---------|----------|
| Tables  | Yes |
| Strikethrough | Yes |
| Footnotes | Yes |

## Footnotes
Here's a footnote reference.[^1]

[^1]: This is the footnote content."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "GFM Features");

        // Check GFM features are preserved
        assert!(result.content.contains("<del>strikethrough</del>"));
        assert!(result.content.contains("<table>"));
        assert!(result.content.contains("<th>Feature</th>"));
        assert!(result.content.contains("<td>Tables</td>"));
        assert!(result.content.contains("<sup>")); // Footnote reference

        Ok(())
    }

    #[test]
    fn test_unicode_content_parsing() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("unicode.md");

        let content = r#"+++
title = "Юникод Документ"
tags = ["ру́сский", "тест"]
+++

# Привет, мир!

Это документ с **юникодным** содержимым и *кириллицей*.

中文内容也支持。

العربية أيضا."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(
            result.frontmatter.title,
            Some("Юникод Документ".to_string())
        );
        assert_eq!(result.title, "Юникод Документ");
        assert_eq!(
            result.frontmatter.tags,
            Some(vec!["ру́сский".to_string(), "тест".to_string()])
        );

        // Check content is preserved correctly
        assert!(result.content.contains("<h1>Привет, мир!</h1>"));
        assert!(result.content.contains("<strong>юникодным</strong>"));
        assert!(result.content.contains("<em>кириллицей</em>"));
        assert!(result.content.contains("中文内容也支持"));
        assert!(result.content.contains("العربية أيضا"));

        Ok(())
    }

    #[test]
    fn test_legacy_parse_frontmatter() {
        let content = r#"+++
title = "Legacy Test"
date = "2024-01-20"
list = true
+++

# Content

Some markdown content."#;

        let parser = MarkdownParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Legacy Test".to_string()));
        assert_eq!(frontmatter.date, Some("2024-01-20".to_string()));
        assert_eq!(frontmatter.list, Some(true));
        assert!(markdown_content.contains("# Content"));
    }

    #[test]
    fn test_legacy_excerpt_extraction() {
        let content = r#"
# Title

This is the first paragraph with **bold** text.

This is the second paragraph."#;

        let parser = MarkdownParser::new();
        let excerpt = parser.extract_excerpt_from_markdown(content);
        assert_eq!(
            excerpt,
            Some("This is the first paragraph with bold text.".to_string())
        );
    }

    #[test]
    fn test_legacy_plain_text_excerpt() {
        assert_eq!(
            MarkdownParser::extract_excerpt_from_plain_text(
                "First paragraph.\n\nSecond paragraph."
            ),
            Some("First paragraph.".to_string())
        );
    }

    #[test]
    fn test_empty_file_parsing() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("empty.md");

        fs::write(&file_path, "")?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "empty");
        assert_eq!(result.content, "");

        Ok(())
    }

    #[test]
    fn test_file_with_only_frontmatter() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("frontmatter-only.md");

        let content = r#"+++
title = "Frontmatter Only"
date = "2024-01-20"
+++"#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "Frontmatter Only");
        assert_eq!(
            result.frontmatter.title,
            Some("Frontmatter Only".to_string())
        );
        assert_eq!(result.frontmatter.date, Some("2024-01-20".to_string()));
        assert_eq!(result.content, ""); // No content after frontmatter

        Ok(())
    }
}
