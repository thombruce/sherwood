use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use crate::content::parser::Frontmatter;

pub trait ContentParser: Send + Sync {
    fn name(&self) -> &'static str;
    fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent>;
}

#[derive(Debug, Clone)]
pub struct ParsedContent {
    pub title: String,
    pub frontmatter: Frontmatter,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

pub struct PluginRegistry {
    parsers: HashMap<String, Box<dyn ContentParser>>,
    extensions: HashMap<String, String>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            extensions: HashMap::new(),
        }
    }

    pub fn register_parser(mut self, name: &str, parser: Box<dyn ContentParser>) -> Self {
        self.parsers.insert(name.to_string(), parser);
        self
    }

    pub fn map_extensions(mut self, mappings: &[(&str, &str)]) -> Self {
        for (extension, parser_name) in mappings {
            if !self.parsers.contains_key(*parser_name) {
                panic!(
                    "Parser '{}' not registered for extension '{}'",
                    parser_name, extension
                );
            }

            if self.extensions.contains_key(*extension) {
                panic!(
                    "Extension '{}' already mapped to parser '{}'",
                    extension, self.extensions[*extension]
                );
            }

            self.extensions
                .insert(extension.to_string(), parser_name.to_string());
        }
        self
    }

    pub fn register(self, name: &str, parser: Box<dyn ContentParser>, extension: &str) -> Self {
        self.register_parser(name, parser)
            .map_extensions(&[(extension, name)])
    }

    pub fn find_parser(&self, path: &Path) -> Result<&dyn ContentParser> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow::anyhow!("File has no extension: {}", path.display()))?;

        match self.extensions.get(extension) {
            Some(parser_name) => {
                let parser = self
                    .parsers
                    .get(parser_name)
                    .ok_or_else(|| anyhow::anyhow!("Parser '{}' not found", parser_name))?;
                Ok(parser.as_ref())
            }
            None => Err(anyhow::anyhow!(
                "No parser registered for .{} files",
                extension
            )),
        }
    }

    pub fn supported_extensions(&self) -> Vec<String> {
        self.extensions.keys().cloned().collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
