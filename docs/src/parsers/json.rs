use anyhow::Result;
use sherwood::content::parser::Frontmatter;
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
        let frontmatter = Frontmatter {
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
        };

        let mut metadata = HashMap::new();
        if let Some(description) = parsed.get("description").and_then(|v| v.as_str()) {
            metadata.insert("description".to_string(), description.to_string());
        }
        if let Some(author) = parsed.get("author").and_then(|v| v.as_str()) {
            metadata.insert("author".to_string(), author.to_string());
        }

        Ok(ParsedContent {
            title,
            frontmatter,
            content: parsed
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            metadata,
        })
    }
}
