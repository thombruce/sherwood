use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub date: Option<String>,
    pub list: Option<bool>,
    pub page_template: Option<String>,
}

#[derive(Debug)]
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
        if content.starts_with("---\n") {
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
