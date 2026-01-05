use anyhow::Result;
use sherwood::content::markdown_util::MarkdownProcessor;
use sherwood::content::parser::Frontmatter;
use sherwood::plugins::{ContentParser, ParsedContent};
use std::collections::HashMap;
use std::path::Path;

pub struct TextContentParser;

impl TextContentParser {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Box<dyn ContentParser> {
        Box::new(Self)
    }
}

impl Default for TextContentParser {
    fn default() -> Self {
        Self
    }
}

impl ContentParser for TextContentParser {
    fn name(&self) -> &'static str {
        "text"
    }

    fn parse(&self, content: &str, _path: &Path) -> Result<ParsedContent> {
        // No frontmatter parsing - entire file is content
        // Title will be derived from filename by existing logic
        let frontmatter = Frontmatter::default(); // Empty frontmatter

        // Convert entire text content to HTML
        let html_content = MarkdownProcessor::process(content)?;

        Ok(ParsedContent {
            title: String::new(), // Will be overridden by filename fallback
            frontmatter,
            content: html_content, // Convert text to HTML
            metadata: HashMap::new(),
        })
    }
}
