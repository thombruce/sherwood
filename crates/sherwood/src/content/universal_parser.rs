use crate::content::parser::MarkdownFile;
use crate::plugins::ParsedContent;
use anyhow::Result;
use std::path::Path;

pub struct UniversalContentParser {
    plugin_registry: Option<crate::plugins::PluginRegistry>,
}

impl UniversalContentParser {
    pub fn new(plugin_registry: Option<crate::plugins::PluginRegistry>) -> Self {
        Self { plugin_registry }
    }

    pub fn parse_file(&self, file_path: &Path) -> Result<MarkdownFile> {
        let content = std::fs::read_to_string(file_path)?;

        // Try custom parsers first
        if let Some(registry) = &self.plugin_registry
            && let Ok(parser) = registry.find_parser(file_path)
        {
            let parsed = parser.parse(&content, file_path)?;
            return self.convert_to_markdown_file(parsed, file_path);
        }

        // Fallback to markdown parser
        crate::content::parser::MarkdownParser::parse_markdown_file(file_path)
    }

    fn convert_to_markdown_file(&self, parsed: ParsedContent, path: &Path) -> Result<MarkdownFile> {
        Ok(MarkdownFile {
            path: path.to_path_buf(),
            content: parsed.content,
            frontmatter: parsed.frontmatter,
            title: parsed.title,
        })
    }

    pub fn supported_extensions(&self) -> Vec<String> {
        if let Some(registry) = &self.plugin_registry {
            let mut extensions = registry.supported_extensions();
            extensions.push("md".to_string());
            extensions.push("markdown".to_string());
            extensions
        } else {
            vec!["md".to_string(), "markdown".to_string()]
        }
    }
}
