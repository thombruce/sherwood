use anyhow::Result;
use sherwood::content::markdown_util::MarkdownProcessor;
use sherwood::content::{Frontmatter, MarkdownParser};
use sherwood::plugins::{ContentParser, ParsedContent};
use std::collections::HashMap;
use std::path::Path;

pub struct JsonContentParser;

impl JsonContentParser {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Box<dyn ContentParser> {
        Box::new(Self)
    }
}

impl Default for JsonContentParser {
    fn default() -> Self {
        Self
    }
}

impl ContentParser for JsonContentParser {
    fn name(&self) -> &'static str {
        "json"
    }

    fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
        let parsed: serde_json::Value = serde_json::from_str(content)?;

        let title = parsed
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
            })
            .to_string();

        // Convert JSON to frontmatter structure
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
        if frontmatter.excerpt.is_none()
            && let Some(content_field) = parsed.get("content").and_then(|v| v.as_str())
        {
            if !content_field.trim().starts_with('<') {
                // Content is markdown - use AST extraction
                let markdown_parser = MarkdownParser::new();
                frontmatter.excerpt = markdown_parser.extract_excerpt_from_markdown(content_field);
            } else {
                // Content is HTML or plain text - use plain text extraction
                frontmatter.excerpt =
                    MarkdownParser::extract_excerpt_from_plain_text(content_field);
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
    fn test_json_parser_with_excerpt_field() {
        let json_content = concat!(
            "{\n",
            "    \"title\": \"Test Article\",\n",
            "    \"excerpt\": \"Custom excerpt from JSON\",\n",
            "    \"date\": \"2024-01-15\",\n",
            "    \"content\": \"# Article Content\\n\\nThis is first paragraph.\\n\\nThis is second paragraph.\"\n",
            "}"
        );

        let parser = JsonContentParser::default();
        let result = parser.parse(json_content, Path::new("test.json")).unwrap();

        assert_eq!(result.title, "Test Article");
        assert_eq!(
            result.frontmatter.excerpt,
            Some("Custom excerpt from JSON".to_string())
        );
        assert_eq!(result.frontmatter.title, Some("Test Article".to_string()));
        assert_eq!(result.frontmatter.date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_json_parser_excerpt_priority() {
        let json_content = concat!(
            "{\n",
            "    \"title\": \"Test Article\",\n",
            "    \"excerpt\": \"Custom excerpt should take priority\",\n",
            "    \"content\": \"# Article Content\\n\\nThis excerpt should be ignored because frontmatter has one.\"\n",
            "}"
        );

        let parser = JsonContentParser::default();
        let result = parser.parse(json_content, Path::new("test.json")).unwrap();

        assert_eq!(
            result.frontmatter.excerpt,
            Some("Custom excerpt should take priority".to_string())
        );
    }

    #[test]
    fn test_json_parser_excerpt_extraction_with_formatting() {
        let json_content = concat!(
            "{\n",
            "    \"title\": \"Test Article\",\n",
            "    \"content\": \"# Article Content\\n\\nThis paragraph has **bold** and *italic* text.\\n\\nSecond paragraph.\"\n",
            "}"
        );

        let parser = JsonContentParser::default();
        let result = parser.parse(json_content, Path::new("test.json")).unwrap();

        assert_eq!(
            result.frontmatter.excerpt,
            Some("This paragraph has bold and italic text.".to_string())
        );
    }

    #[test]
    fn test_json_parser_excerpt_extraction_from_plain_text() {
        let json_content = concat!(
            "{\n",
            "    \"title\": \"Test Article\",\n",
            "    \"content\": \"This is plain text content.\\n\\nThis is second paragraph.\"\n",
            "}"
        );

        let parser = JsonContentParser::default();
        let result = parser.parse(json_content, Path::new("test.json")).unwrap();

        assert_eq!(
            result.frontmatter.excerpt,
            Some("This is plain text content.".to_string())
        );
    }

    #[test]
    fn test_json_parser_no_content_field() {
        let json_content = r#"{
    "title": "Test Article",
    "excerpt": "Custom excerpt"
}"#;

        let parser = JsonContentParser::default();
        let result = parser.parse(json_content, Path::new("test.json")).unwrap();

        assert_eq!(result.title, "Test Article");
        assert_eq!(
            result.frontmatter.excerpt,
            Some("Custom excerpt".to_string())
        );
        assert_eq!(result.content, ""); // Empty HTML content
    }

    #[test]
    fn test_json_parser_html_content() {
        let json_content = concat!(
            "{\n",
            "    \"title\": \"Test Article\",\n",
            "    \"content\": \"<p>This is HTML content.</p>\\n\\n<p>Second paragraph.</p>\"\n",
            "}"
        );

        let parser = JsonContentParser::default();
        let result = parser.parse(json_content, Path::new("test.json")).unwrap();

        assert_eq!(
            result.frontmatter.excerpt,
            Some("<p>This is HTML content.</p>".to_string())
        );
        assert!(result.content.contains("<p>This is HTML content.</p>"));
    }

    #[test]
    fn test_json_parser_with_metadata_and_excerpt() {
        let json_content = concat!(
            "{\n",
            "    \"title\": \"Article with Metadata\",\n",
            "    \"excerpt\": \"Article with excerpt\",\n",
            "    \"author\": \"John Doe\",\n",
            "    \"description\": \"Article description\",\n",
            "    \"content\": \"# Content\\n\\nFirst paragraph.\\n\\nSecond paragraph.\"\n",
            "}"
        );

        let parser = JsonContentParser::default();
        let result = parser.parse(json_content, Path::new("test.json")).unwrap();

        assert_eq!(result.title, "Article with Metadata");
        assert_eq!(
            result.frontmatter.excerpt,
            Some("Article with excerpt".to_string())
        );
        assert_eq!(result.metadata.get("author"), Some(&"John Doe".to_string()));
        assert_eq!(
            result.metadata.get("description"),
            Some(&"Article description".to_string())
        );
    }

    #[test]
    fn test_json_parser_with_complex_markdown() {
        let json_content = concat!(
            "{\n",
            "    \"title\": \"Complex Article\",\n",
            "    \"content\": \"# Complex Article\\n\\nThis paragraph has **bold** and *italic* formatting.\\n\\nSecond paragraph here.\"\n",
            "}"
        );

        let parser = JsonContentParser::default();
        let result = parser.parse(json_content, Path::new("test.json")).unwrap();

        assert_eq!(
            result.frontmatter.excerpt,
            Some("This paragraph has bold and italic formatting.".to_string())
        );
    }
}
