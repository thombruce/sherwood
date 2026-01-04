use anyhow::Result;
use gray_matter::Matter;
use gray_matter::engine::{TOML, YAML};
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
    toml_matter: Matter<TOML>,
    yaml_matter: Matter<YAML>,
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    pub fn new() -> Self {
        // Configure TOML parser with +++ delimiters
        let mut toml_matter = Matter::<TOML>::new();
        toml_matter.delimiter = "+++".to_string();

        // YAML parser uses default --- delimiters
        let yaml_matter = Matter::<YAML>::new();

        Self {
            toml_matter,
            yaml_matter,
        }
    }

    pub fn parse_markdown_file(file_path: &Path) -> Result<MarkdownFile> {
        let content = std::fs::read_to_string(file_path)?;
        let parser = Self::new();
        parser.parse_content(&content, file_path)
    }

    fn parse_content(&self, content: &str, file_path: &Path) -> Result<MarkdownFile> {
        // Parse frontmatter and extract content
        let (frontmatter, markdown_content) = self.parse_frontmatter(content)?;

        // Extract title from frontmatter, first h1, or filename
        let title = frontmatter
            .title
            .clone()
            .or_else(|| Self::extract_title(&markdown_content))
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

    fn parse_frontmatter(&self, content: &str) -> Result<(Frontmatter, String)> {
        // Try TOML first (+++ delimiters) - maintains existing priority
        if content.starts_with("+++\n")
            && let Ok(result) = self.toml_matter.parse::<Frontmatter>(content)
        {
            return Ok((result.data.unwrap_or_default(), result.content));
        }

        // Try YAML (--- delimiters)
        if content.starts_with("---\n")
            && let Ok(result) = self.yaml_matter.parse::<Frontmatter>(content)
        {
            return Ok((result.data.unwrap_or_default(), result.content));
        }

        // No frontmatter detected
        Ok((Frontmatter::default(), content.to_string()))
    }

    fn extract_title(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix("# ") {
                return Some(stripped.trim().to_string());
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
        // gray_matter extracts content between mismatched delimiters, which is different from original behavior
        assert_eq!(markdown_content, "title = \"Test Title\"\n---\n# Content");
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

# Testing TOML delimiters with gray_matter"#;

        let parser = MarkdownParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Delimiter Test".to_string()));
        assert_eq!(
            markdown_content,
            "# Testing TOML delimiters with gray_matter"
        );
    }

    #[test]
    fn test_gray_matter_yaml_delimiters() {
        let content = r#"---
title: "Delimiter Test"
---

# Testing YAML delimiters with gray_matter"#;

        let parser = MarkdownParser::new();
        let result = parser.parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, Some("Delimiter Test".to_string()));
        assert_eq!(
            markdown_content,
            "# Testing YAML delimiters with gray_matter"
        );
    }
}
