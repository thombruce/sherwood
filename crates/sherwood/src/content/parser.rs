use anyhow::Result;
use markdown::mdast::{Node, Root};
use markdown::{to_mdast, Constructs, ParseOptions};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub date: Option<String>,
    pub list: Option<bool>,
    pub page_template: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct MarkdownFile {
    pub path: std::path::PathBuf,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub title: String,
}

pub struct MarkdownParser {
    parse_options: ParseOptions,
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    pub fn new() -> Self {
        let parse_options = ParseOptions {
            constructs: Constructs {
                frontmatter: true,
                ..Default::default()
            },
            ..ParseOptions::default()
        };

        Self { parse_options }
    }

    pub fn parse_markdown_file(file_path: &Path) -> Result<MarkdownFile> {
        let content = std::fs::read_to_string(file_path)?;
        let parser = Self::new();
        parser.parse_content(&content, file_path)
    }

    fn parse_content(&self, content: &str, file_path: &Path) -> Result<MarkdownFile> {
        // Parse AST once for both frontmatter and title extraction
        let root = to_mdast(content, &self.parse_options)
            .map_err(|e| anyhow::anyhow!("Failed to parse markdown: {}", e))?;

        // Extract frontmatter and clean content using AST
        let (frontmatter, markdown_content) = match &root {
            Node::Root(root_node) => self.extract_frontmatter_from_root(root_node, content),
            _ => Ok((Frontmatter::default(), content.to_string())),
        }?;

        // Extract title from frontmatter, first h1 from AST, or filename
        let title = frontmatter
            .title
            .clone()
            .or_else(|| Self::extract_title_from_ast(&root))
            .unwrap_or_else(|| {
                file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string()
            });

        Ok(MarkdownFile {
            path: file_path.to_path_buf(),
            content: markdown_content,
            frontmatter,
            title,
        })
    }

    fn extract_frontmatter_from_root(
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

    #[allow(dead_code)]
    fn parse_frontmatter(&self, content: &str) -> Result<(Frontmatter, String)> {
        let root = to_mdast(content, &self.parse_options)
            .map_err(|e| anyhow::anyhow!("Failed to parse markdown: {}", e))?;

        match root {
            Node::Root(root) => self.extract_frontmatter_from_root(&root, content),
            _ => Ok((Frontmatter::default(), content.to_string())),
        }
    }

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

    /// Extract plain text content from AST nodes recursively
    fn extract_text_from_nodes(nodes: &[Node]) -> String {
        nodes
            .iter()
            .map(|node| match node {
                Node::Text(text) => text.value.clone(),
                Node::Emphasis(emphasis) => Self::extract_text_from_nodes(&emphasis.children),
                Node::Strong(strong) => Self::extract_text_from_nodes(&strong.children),
                Node::InlineCode(code) => code.value.clone(),
                Node::Delete(delete) => Self::extract_text_from_nodes(&delete.children),
                Node::Link(link) => Self::extract_text_from_nodes(&link.children),
                Node::Image(image) => {
                    // Use alt text for images in headings
                    image.alt.clone()
                }
                Node::InlineMath(math) => math.value.clone(),
                // MDX nodes
                Node::MdxTextExpression(_) | Node::MdxJsxTextElement(_) => {
                    // For MDX content, we'll extract text if possible or skip
                    String::new()
                }
                _ => String::new(),
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Extract title from AST by finding the first H1 heading
    fn extract_title_from_ast(root: &Node) -> Option<String> {
        if let Node::Root(root_node) = root {
            for child in &root_node.children {
                if let Node::Heading(heading) = child {
                    if heading.depth == 1 {
                        let title_text = Self::extract_text_from_nodes(&heading.children);
                        if !title_text.trim().is_empty() {
                            return Some(title_text.trim().to_string());
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_toml_frontmatter_parsing() {
        let content = r#"+++
title = "Test Title"
date = "2024-01-15"
list = true
+++

# Content

This is the markdown content."#;

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Test Title".to_string()));
        assert_eq!(frontmatter.page_template, Some("custom.stpl".to_string()));
        assert!(markdown_content.contains("# Content"));
    }

    #[test]
    fn test_malformed_delimiters() {
        let content = r#"+++
title = "Test Title"
---
# Content"#;

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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
        assert!(result.content.contains("# Test File"));
        assert_eq!(result.path, file_path);

        Ok(())
    }

    #[test]
    fn test_title_extraction_from_h1() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"
# Extracted Title

This content has no frontmatter title."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "Extracted Title");
        assert_eq!(result.frontmatter.title, None);

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
    fn test_sort_fields_parsing() {
        let content = r#"+++
title = "Test Title"
date = "2024-01-15"
list = true
sort_by = "date"
sort_order = "desc"
+++

# Content"#;

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, _) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Test Title".to_string()));
        assert_eq!(frontmatter.tags, Some(vec![]));
    }

    #[test]
    fn test_gray_matter_toml_delimiters() {
        let content = r#"+++
title = "Delimiter Test"
+++

# Testing TOML delimiters with markdown crate"#;

        let parser = MarkdownParser::new();
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

        let parser = MarkdownParser::new();
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
        let parser = MarkdownParser::new();

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
    fn test_ast_title_extraction_simple() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"# Simple Title

This content has a simple H1 title."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "Simple Title");
        assert_eq!(result.frontmatter.title, None);

        Ok(())
    }

    #[test]
    fn test_ast_title_extraction_with_emphasis() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"# Title with *emphasis* and **bold**

This content has a complex H1 title."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "Title with emphasis and bold");
        assert_eq!(result.frontmatter.title, None);

        Ok(())
    }

    #[test]
    fn test_ast_title_extraction_with_inline_code() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"# Title with `code` and more text

This content has inline code in the title."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "Title with code and more text");
        assert_eq!(result.frontmatter.title, None);

        Ok(())
    }

    #[test]
    fn test_ast_title_extraction_with_link() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"# Title with [a link](https://example.com) text

This content has a link in the title."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "Title with a link text");
        assert_eq!(result.frontmatter.title, None);

        Ok(())
    }

    #[test]
    fn test_ast_title_extraction_complex_formatting() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"# Title with *italic*, **bold**, `code`, and [links](https://example.com)

This content has all types of inline formatting."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        assert_eq!(result.title, "Title with italic, bold, code, and links");
        assert_eq!(result.frontmatter.title, None);

        Ok(())
    }

    #[test]
    fn test_ast_title_extraction_ignores_h2_and_below() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"## H2 Title
### H3 Title

This content has no H1 title."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        // Should fall back to filename since no H1 found
        assert_eq!(result.title, "test");
        assert_eq!(result.frontmatter.title, None);

        Ok(())
    }

    #[test]
    fn test_ast_title_extraction_first_h1_only() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"# First Title
# Second Title

This content has multiple H1 titles."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        // Should extract the first H1 only
        assert_eq!(result.title, "First Title");
        assert_eq!(result.frontmatter.title, None);

        Ok(())
    }

    #[test]
    fn test_ast_title_extraction_with_frontmatter_priority() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"+++
title = "Frontmatter Title"
+++

# H1 Title

This content has both frontmatter and H1 title."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        // Frontmatter title should take priority
        assert_eq!(result.title, "Frontmatter Title");
        assert_eq!(
            result.frontmatter.title,
            Some("Frontmatter Title".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_ast_title_extraction_empty_heading() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = r#"#

This content has an empty H1 heading."#;

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        // Should fall back to filename since H1 is empty
        assert_eq!(result.title, "test");
        assert_eq!(result.frontmatter.title, None);

        Ok(())
    }

    #[test]
    fn test_ast_title_extraction_whitespace_only() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.md");

        let content = "#
   
This content has a whitespace-only H1 heading.";

        fs::write(&file_path, content)?;

        let result = MarkdownParser::parse_markdown_file(&file_path)?;

        // Should fall back to filename since H1 contains only whitespace
        assert_eq!(result.title, "test");
        assert_eq!(result.frontmatter.title, None);

        Ok(())
    }

    #[test]
    fn test_ast_vs_string_parsing_compatibility() -> Result<()> {
        let temp_dir = tempdir()?;

        // Test cases that should work the same for both methods
        let test_cases = vec![
            ("simple", "# Simple Title\nContent here.", "Simple Title"),
            (
                "with-space",
                "# Title with space\nContent here.",
                "Title with space",
            ),
            (
                "with-punctuation",
                "# Title, with punctuation!\nContent here.",
                "Title, with punctuation!",
            ),
        ];

        for (filename, content, expected_title) in test_cases {
            let file_path = temp_dir.path().join(format!("{}.md", filename));
            fs::write(&file_path, content)?;

            let result = MarkdownParser::parse_markdown_file(&file_path)?;
            assert_eq!(
                result.title, expected_title,
                "Failed for case: {}",
                filename
            );
        }

        Ok(())
    }
}
