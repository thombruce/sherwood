use anyhow::Result;
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
}

#[derive(Debug, Clone)]
pub struct MarkdownFile {
    pub path: std::path::PathBuf,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub title: String,
}

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn parse_markdown_file(file_path: &Path) -> Result<MarkdownFile> {
        let content = std::fs::read_to_string(file_path)?;

        // Parse frontmatter and extract content
        let (frontmatter, markdown_content) = Self::parse_frontmatter(&content)?;

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

    fn parse_frontmatter(content: &str) -> Result<(Frontmatter, String)> {
        // Try TOML first (+++ delimiters)
        if content.starts_with("+++\n") {
            return Self::parse_toml_frontmatter(content);
        }

        // Fall back to YAML (--- delimiters)
        if content.starts_with("---\n") {
            return Self::parse_yaml_frontmatter(content);
        }

        // No frontmatter found
        Ok((Frontmatter::default(), content.to_string()))
    }

    fn parse_toml_frontmatter(content: &str) -> Result<(Frontmatter, String)> {
        let parts: Vec<&str> = content.splitn(3, "+++\n").collect();
        if parts.len() >= 3 {
            let frontmatter_str = parts[1];
            let markdown_content = parts[2];

            match toml::from_str::<Frontmatter>(frontmatter_str) {
                Ok(frontmatter) => return Ok((frontmatter, markdown_content.to_string())),
                Err(e) => {
                    eprintln!(
                        "Warning: Invalid frontmatter TOML: {}. Using default frontmatter.",
                        e
                    );
                    return Ok((Frontmatter::default(), content.to_string()));
                }
            }
        }

        Ok((Frontmatter::default(), content.to_string()))
    }

    fn parse_yaml_frontmatter(content: &str) -> Result<(Frontmatter, String)> {
        let parts: Vec<&str> = content.splitn(3, "---\n").collect();
        if parts.len() >= 3 {
            let frontmatter_str = parts[1];
            let markdown_content = parts[2];

            match serde_yaml::from_str::<Frontmatter>(frontmatter_str) {
                Ok(frontmatter) => return Ok((frontmatter, markdown_content.to_string())),
                Err(e) => {
                    eprintln!(
                        "Warning: Invalid frontmatter YAML: {}. Using default frontmatter.",
                        e
                    );
                    return Ok((Frontmatter::default(), content.to_string()));
                }
            }
        }

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

        let result = MarkdownParser::parse_frontmatter(content);
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

        let result = MarkdownParser::parse_frontmatter(content);
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

        let result = MarkdownParser::parse_frontmatter(content);
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

        let result = MarkdownParser::parse_frontmatter(content);
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

        let result = MarkdownParser::parse_frontmatter(content);
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

        let result = MarkdownParser::parse_frontmatter(content);
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

        let result = MarkdownParser::parse_frontmatter(content);
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

        let result = MarkdownParser::parse_frontmatter(content);
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

        let result = MarkdownParser::parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, markdown_content) = result.unwrap();
        assert_eq!(frontmatter.title, None); // Should not parse as frontmatter
        assert_eq!(markdown_content, content);
    }

    #[test]
    fn test_empty_frontmatter_toml() {
        let content = r#"+++
+++

# Content"#;

        let result = MarkdownParser::parse_frontmatter(content);
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

        let result = MarkdownParser::parse_frontmatter(content);
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

        let result = MarkdownParser::parse_frontmatter(content);
        assert!(result.is_ok());

        let (frontmatter, _) = result.unwrap();
        assert_eq!(frontmatter.sort_by, Some("title".to_string()));
        assert_eq!(frontmatter.sort_order, Some("asc".to_string()));
    }
}
