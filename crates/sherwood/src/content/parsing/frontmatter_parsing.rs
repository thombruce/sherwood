use anyhow::Result;
use markdown::mdast::{Node, Root};
use markdown::{ParseOptions, to_mdast};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub date: Option<String>,
    pub list: Option<bool>,
    pub page_template: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub tags: Option<Vec<String>>,
    pub excerpt: Option<String>,
}

pub struct FrontmatterParser {
    parse_options: ParseOptions,
}

impl Default for FrontmatterParser {
    fn default() -> Self {
        Self::new()
    }
}

impl FrontmatterParser {
    pub fn new() -> Self {
        let parse_options = ParseOptions {
            constructs: markdown::Constructs {
                frontmatter: true,
                ..Default::default()
            },
            ..ParseOptions::default()
        };

        Self { parse_options }
    }

    /// Parse frontmatter from content string and return parsed frontmatter with cleaned markdown content
    pub fn parse_frontmatter(&self, content: &str) -> Result<(Frontmatter, String)> {
        let root = to_mdast(content, &self.parse_options)
            .map_err(|e| anyhow::anyhow!("Failed to parse markdown: {}", e))?;

        match root {
            Node::Root(root) => self.extract_frontmatter_from_root(&root, content),
            _ => Ok((Frontmatter::default(), content.to_string())),
        }
    }

    /// Extract frontmatter from AST root node and clean content using position information
    pub fn extract_frontmatter_from_root(
        &self,
        root: &Root,
        original_content: &str,
    ) -> Result<(Frontmatter, String)> {
        let mut frontmatter = Frontmatter::default();
        let mut frontmatter_end_byte = None;

        #[allow(clippy::never_loop)]
        for child in root.children.iter() {
            match child {
                Node::Toml(toml_node) => {
                    if let Ok(parsed) = toml::from_str::<Frontmatter>(&toml_node.value) {
                        frontmatter = parsed;
                    }

                    // Get position information for content extraction
                    if let Some(position) = &toml_node.position {
                        frontmatter_end_byte = Some(position.end.offset);
                    }
                    break;
                }
                Node::Yaml(yaml_node) => {
                    if let Ok(parsed) = serde_yaml::from_str::<Frontmatter>(&yaml_node.value) {
                        frontmatter = parsed;
                    }

                    if let Some(position) = &yaml_node.position {
                        frontmatter_end_byte = Some(position.end.offset);
                    }
                    break;
                }
                _ => break,
            }
        }

        // Use AST position information for clean content extraction
        let markdown_content =
            self.extract_content_using_ast_position(original_content, frontmatter_end_byte);

        Ok((frontmatter, markdown_content))
    }

    /// Extract markdown content using AST position information
    /// This ensures clean separation between frontmatter and content
    fn extract_content_using_ast_position(
        &self,
        original_content: &str,
        frontmatter_end_byte: Option<usize>,
    ) -> String {
        match frontmatter_end_byte {
            Some(end_byte) => {
                // Convert byte offset to char offset safely
                let content_bytes = original_content.as_bytes();

                if end_byte >= content_bytes.len() {
                    // Frontmatter extends to end of content, return empty
                    return String::new();
                }

                // Find the content after frontmatter
                let remaining_bytes = &content_bytes[end_byte..];

                // Convert back to string and clean up leading whitespace
                let content_str = String::from_utf8_lossy(remaining_bytes);

                // Trim leading newlines and whitespace
                content_str.trim_start().to_string()
            }
            None => {
                // No frontmatter found, return original content
                original_content.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_frontmatter_parsing() {
        let content = r#"+++
title = "Test Title"
date = "2024-01-15"
list = true
+++

# Content

This is the markdown content."#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Test Title".to_string()));
        assert_eq!(frontmatter.date, Some("2024-01-15".to_string()));
        assert_eq!(frontmatter.list, Some(true));
        assert_eq!(frontmatter.page_template, None);
        assert!(markdown_content.contains("# Content"));
    }

    #[test]
    fn test_yaml_frontmatter_parsing() {
        let content = r#"---
title: "Test Title"
date: "2024-01-15"
list: true
---

# Content

This is the markdown content."#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Test Title".to_string()));
        assert_eq!(frontmatter.date, Some("2024-01-15".to_string()));
        assert_eq!(frontmatter.list, Some(true));
        assert_eq!(frontmatter.page_template, None);
        assert!(markdown_content.contains("# Content"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = r#"# Simple Content

This content has no frontmatter."#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, None);
        assert_eq!(frontmatter.date, None);
        assert_eq!(frontmatter.list, None);
        assert_eq!(markdown_content, content);
    }

    #[test]
    fn test_invalid_toml_frontmatter() {
        let content = r#"+++
title = "Test Title"
invalid toml syntax
+++

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok()); // Should fall back to default

        let (frontmatter, _) = result.unwrap();
        assert_eq!(frontmatter.title, None);
    }

    #[test]
    fn test_invalid_yaml_frontmatter() {
        let content = r#"---
title: "Test Title"
invalid: yaml: syntax::
---

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok()); // Should fall back to default

        let (frontmatter, _) = result.unwrap();
        assert_eq!(frontmatter.title, None);
    }

    #[test]
    fn test_partial_frontmatter_toml() {
        let content = r#"+++
title = "Only Title"
+++

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, _) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Only Title".to_string()));
        assert_eq!(frontmatter.date, None);
        assert_eq!(frontmatter.list, None);
        assert_eq!(frontmatter.page_template, None);
    }

    #[test]
    fn test_page_template_field_toml() {
        let content = r#"+++
title = "Test Title"
page_template = "custom.stpl"
+++

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Test Title".to_string()));
        assert_eq!(frontmatter.page_template, Some("custom.stpl".to_string()));
        assert!(markdown_content.contains("# Content"));
    }

    #[test]
    fn test_page_template_field_yaml() {
        let content = r#"---
title: "Test Title"
page_template: "custom.stpl"
---

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Test Title".to_string()));
        assert_eq!(frontmatter.page_template, Some("custom.stpl".to_string()));
        assert!(markdown_content.contains("# Content"));
    }

    #[test]
    fn test_sort_fields_parsing() {
        let content = r#"+++
title = "Test Title"
date = "2024-01-15"
list = true
sort_by = "date"
sort_order = "desc"
+++

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, _) = result.unwrap();
        assert_eq!(frontmatter.sort_by, Some("date".to_string()));
        assert_eq!(frontmatter.sort_order, Some("desc".to_string()));
    }

    #[test]
    fn test_sort_fields_yaml_parsing() {
        let content = r#"---
title: "Test Title"
date: "2024-01-15"
list: true
sort_by: "title"
sort_order: "asc"
---

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, _) = result.unwrap();
        assert_eq!(frontmatter.sort_by, Some("title".to_string()));
        assert_eq!(frontmatter.sort_order, Some("asc".to_string()));
    }

    #[test]
    fn test_tags_field_toml_parsing() {
        let content = r#"+++
title = "Test Title"
tags = ["rust", "web-development", "ssg"]
+++

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, _) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Test Title".to_string()));
        assert_eq!(
            frontmatter.tags,
            Some(vec![
                "rust".to_string(),
                "web-development".to_string(),
                "ssg".to_string()
            ])
        );
    }

    #[test]
    fn test_tags_field_yaml_parsing() {
        let content = r#"---
title: "Test Title"
tags:
  - rust
  - web-development
  - ssg
---

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, _) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Test Title".to_string()));
        assert_eq!(
            frontmatter.tags,
            Some(vec![
                "rust".to_string(),
                "web-development".to_string(),
                "ssg".to_string()
            ])
        );
    }

    #[test]
    fn test_empty_tags_field() {
        let content = r#"+++
title = "Test Title"
tags = []
+++

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, _) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Test Title".to_string()));
        assert_eq!(frontmatter.tags, Some(vec![]));
    }

    #[test]
    fn test_malformed_delimiters() {
        let content = r#"+++
title = "Test Title"
---
# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, None); // Should not parse as valid frontmatter
        assert_eq!(
            markdown_content,
            "+++\ntitle = \"Test Title\"\n---\n# Content"
        ); // Markdown crate treats malformed frontmatter as regular content
    }

    #[test]
    fn test_empty_frontmatter_toml() {
        let content = r#"+++
+++

# Content"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, None);
        assert_eq!(frontmatter.date, None);
        assert_eq!(frontmatter.list, None);
        assert_eq!(frontmatter.page_template, None);
        assert!(markdown_content.contains("# Content"));
    }

    #[test]
    fn test_gray_matter_toml_delimiters() {
        let content = r#"+++
title = "Delimiter Test"
+++

# Testing TOML delimiters with markdown crate"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Delimiter Test".to_string()));

        // Verify frontmatter is completely removed from markdown content
        assert_eq!(
            markdown_content.trim(),
            "# Testing TOML delimiters with markdown crate"
        );
    }

    #[test]
    fn test_gray_matter_yaml_delimiters() {
        let content = r#"---
title: "Delimiter Test"
---

# Testing YAML delimiters with markdown crate"#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Delimiter Test".to_string()));

        // Verify frontmatter is completely removed from markdown content
        assert_eq!(
            markdown_content.trim(),
            "# Testing YAML delimiters with markdown crate"
        );
    }

    #[test]
    fn test_ast_guided_frontmatter_extraction() {
        let parser = FrontmatterParser::new();

        let content = r#"+++
title = "Test Article"
date = "2023-01-01"
tags = ["test", "extraction"]
+++

# Main Content

This is the main content of the article.

## Subsection

More content here."#;

        let (frontmatter, markdown_content) = parser.parse_frontmatter(content).unwrap();

        // Verify frontmatter is parsed correctly
        assert_eq!(frontmatter.title, Some("Test Article".to_string()));
        assert_eq!(frontmatter.date, Some("2023-01-01".to_string()));
        assert_eq!(
            frontmatter.tags,
            Some(vec!["test".to_string(), "extraction".to_string()])
        );

        // Verify frontmatter is completely removed from markdown content
        assert!(!markdown_content.contains("title = \"Test Article\""));
        assert!(!markdown_content.contains("date = \"2023-01-01\""));
        assert!(!markdown_content.contains("+++"));

        // Verify content structure is preserved
        let markdown_lines: Vec<&str> = markdown_content.trim().lines().collect();
        assert_eq!(markdown_lines[0], "# Main Content");
        assert!(markdown_content.contains("## Subsection"));
        assert!(markdown_content.contains("More content here."));
    }

    #[test]
    fn test_excerpt_in_frontmatter() {
        let content = r#"+++
title = "Test"
excerpt = "This is a custom excerpt"
+++

# Content

More content here."#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content).unwrap();
        assert_eq!(
            result.0.excerpt,
            Some("This is a custom excerpt".to_string())
        );
    }

    #[test]
    fn test_excerpt_in_yaml_frontmatter() {
        let content = r#"---
title: "Test"
excerpt: "YAML excerpt"
---

# Content

More content here."#;

        let parser = FrontmatterParser::new();
        let result = parser.parse_frontmatter(content).unwrap();
        assert_eq!(result.0.excerpt, Some("YAML excerpt".to_string()));
    }
}
