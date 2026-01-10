use anyhow::Result;
use sherwood::content::markdown_util::MarkdownProcessor;
use sherwood::content::{Frontmatter, MarkdownParser};
use sherwood::plugins::{ContentParser, ParsedContent};
use std::collections::HashMap;
use std::path::Path;

pub struct TomlContentParser;

impl TomlContentParser {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Box<dyn ContentParser> {
        Box::new(Self)
    }
}

impl Default for TomlContentParser {
    fn default() -> Self {
        Self
    }
}

impl ContentParser for TomlContentParser {
    fn name(&self) -> &'static str {
        "toml"
    }

    fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
        let parsed: toml::Value = toml::from_str(content)?;

        let title = parsed
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
            })
            .to_string();

        // Convert TOML to frontmatter structure
        let mut frontmatter = Frontmatter {
            title: parsed
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            date: parsed
                .get("date")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            list: parsed.get("list").and_then(|v| v.as_bool()),
            page_template: parsed
                .get("page_template")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            sort_by: parsed
                .get("sort_by")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            sort_order: parsed
                .get("sort_order")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            tags: parsed.get("tags").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            }),
            excerpt: parsed
                .get("excerpt")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        let mut metadata = HashMap::new();
        if let Some(description) = parsed.get("description").and_then(|v| v.as_str()) {
            metadata.insert("description".to_string(), description.to_string());
        }
        if let Some(author) = parsed.get("author").and_then(|v| v.as_str()) {
            metadata.insert("author".to_string(), author.to_string());
        }

        // Extract excerpt if not present
        if frontmatter.excerpt.is_none() {
            if let Some(content_field) = parsed.get("content").and_then(|v| v.as_str()) {
                if !content_field.trim().starts_with('<') {
                    // Content is markdown - use AST extraction
                    let markdown_parser = MarkdownParser::new();
                    frontmatter.excerpt =
                        markdown_parser.extract_excerpt_from_markdown(content_field);
                } else {
                    // Content is HTML or plain text - use plain text extraction
                    frontmatter.excerpt =
                        MarkdownParser::extract_excerpt_from_plain_text(content_field);
                }
            }
        }

        // Handle content: support both markdown and HTML
        let html_content =
            if let Some(content_field) = parsed.get("content").and_then(|v| v.as_str()) {
                if content_field.trim().starts_with('<')
                    && (content_field.contains("</") || content_field.contains("/>"))
                {
                    // Content is already HTML
                    content_field.to_string()
                } else {
                    // Convert markdown to HTML
                    MarkdownProcessor::process(content_field)?
                }
            } else {
                // No content field, generate empty HTML
                String::new()
            };

        Ok(ParsedContent {
            title,
            frontmatter,
            content: html_content, // Now always HTML
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_parser_with_excerpt_field() {
        let toml_content = r#"title = "Test Article"
excerpt = "Custom excerpt from TOML"
date = "2024-01-15"
content = '''
# Article Content

This is the first paragraph.

This is the second paragraph.
'''"#;

        let parser = TomlContentParser::default();
        let result = parser.parse(toml_content, Path::new("test.toml")).unwrap();

        assert_eq!(result.title, "Test Article");
        assert_eq!(
            result.frontmatter.excerpt,
            Some("Custom excerpt from TOML".to_string())
        );
        assert_eq!(result.frontmatter.title, Some("Test Article".to_string()));
        assert_eq!(result.frontmatter.date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_toml_parser_excerpt_extraction_from_markdown() {
        let toml_content = r#"title = "Test Article"
content = '''
# Article Content

This is the first paragraph that should be extracted.

This is the second paragraph.
'''"#;

        let parser = TomlContentParser::default();
        let result = parser.parse(toml_content, Path::new("test.toml")).unwrap();

        assert_eq!(result.title, "Test Article");
        assert_eq!(
            result.frontmatter.excerpt,
            Some("This is the first paragraph that should be extracted.".to_string())
        );
    }

    #[test]
    fn test_toml_parser_excerpt_extraction_with_formatting() {
        let toml_content = r#"title = "Test Article"
content = '''
# Article Content

This paragraph has **bold** and *italic* text.

Second paragraph.
'''"#;

        let parser = TomlContentParser::default();
        let result = parser.parse(toml_content, Path::new("test.toml")).unwrap();

        assert_eq!(
            result.frontmatter.excerpt,
            Some("This paragraph has bold and italic text.".to_string())
        );
    }

    #[test]
    fn test_toml_parser_excerpt_extraction_from_plain_text() {
        let toml_content = r#"title = "Test Article"
content = '''
This is plain text content.

This is the second paragraph.
'''"#;

        let parser = TomlContentParser::default();
        let result = parser.parse(toml_content, Path::new("test.toml")).unwrap();

        assert_eq!(
            result.frontmatter.excerpt,
            Some("This is plain text content.".to_string())
        );
    }

    #[test]
    fn test_toml_parser_no_content_field() {
        let toml_content = r#"title = "Test Article"
excerpt = "Custom excerpt""#;

        let parser = TomlContentParser::default();
        let result = parser.parse(toml_content, Path::new("test.toml")).unwrap();

        assert_eq!(result.title, "Test Article");
        assert_eq!(
            result.frontmatter.excerpt,
            Some("Custom excerpt".to_string())
        );
        assert_eq!(result.content, ""); // Empty HTML content
    }

    #[test]
    fn test_toml_parser_html_content() {
        let toml_content = r#"title = "Test Article"
content = '''
<p>This is HTML content.</p>

<p>Second paragraph.</p>
'''"#;

        let parser = TomlContentParser::default();
        let result = parser.parse(toml_content, Path::new("test.toml")).unwrap();

        assert_eq!(
            result.frontmatter.excerpt,
            Some("<p>This is HTML content.</p>".to_string())
        );
        assert!(result.content.contains("<p>This is HTML content.</p>"));
    }

    #[test]
    fn test_toml_parser_excerpt_priority() {
        let toml_content = r#"title = "Test Article"
excerpt = "Custom excerpt should take priority"
content = '''
# Article Content

This excerpt should be ignored because frontmatter has one.
'''"#;

        let parser = TomlContentParser::default();
        let result = parser.parse(toml_content, Path::new("test.toml")).unwrap();

        assert_eq!(
            result.frontmatter.excerpt,
            Some("Custom excerpt should take priority".to_string())
        );
    }

    #[test]
    fn test_toml_parser_with_complex_markdown() {
        let toml_content = r#"title = "Complex Article"
content = '''
# Complex Article

This paragraph has **bold**, *italic*, `code`, and [links](https://example.com).

Second paragraph here.
'''"#;

        let parser = TomlContentParser::default();
        let result = parser.parse(toml_content, Path::new("test.toml")).unwrap();

        assert_eq!(
            result.frontmatter.excerpt,
            Some("This paragraph has bold, italic, code, and links.".to_string())
        );
    }
}
