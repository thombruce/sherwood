use crate::config::BreadcrumbSection;
use crate::content::parser::{MarkdownFile, MarkdownParser};
use crate::presentation::templates::{BreadcrumbData, BreadcrumbItem};
use anyhow::Result;
use sailfish::TemplateOnce;
use std::path::{Path, PathBuf};

#[derive(TemplateOnce)]
#[template(path = "breadcrumb.stpl")]
pub struct Breadcrumb {
    pub items: Vec<BreadcrumbItem>,
}

impl Breadcrumb {
    pub fn new(data: BreadcrumbData) -> Self {
        Self { items: data.items }
    }
}

impl From<BreadcrumbData> for Breadcrumb {
    fn from(data: BreadcrumbData) -> Self {
        Self { items: data.items }
    }
}

pub struct BreadcrumbGenerator {
    input_dir: PathBuf,
    config: Option<BreadcrumbSection>,
}

impl BreadcrumbGenerator {
    pub fn new(input_dir: &Path, config: Option<BreadcrumbSection>) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            config,
        }
    }

    pub fn generate_breadcrumb(&self, file: &MarkdownFile) -> Result<Option<BreadcrumbData>> {
        // Check if breadcrumbs are enabled
        if let Some(config) = &self.config
            && config.enabled == Some(false)
        {
            return Ok(None);
        }

        // Get the relative path from input directory
        let relative_path = match file.path.strip_prefix(&self.input_dir) {
            Ok(path) => path,
            Err(_) => return Ok(None),
        };

        // Don't generate breadcrumbs for the root index
        if relative_path == Path::new("index.md") {
            return Ok(None);
        }

        let mut breadcrumb_items = Vec::new();

        // Add Home breadcrumb
        breadcrumb_items.push(BreadcrumbItem {
            title: "Home".to_string(),
            url: String::new(), // Root URL
            is_current: false,
        });

        // Extract path components
        let components: Vec<&str> = relative_path
            .parent()
            .unwrap_or(Path::new(""))
            .components()
            .filter_map(|comp| comp.as_os_str().to_str())
            .collect();

        // Build breadcrumb trail
        let mut current_path = String::new();
        for component in components.iter() {
            current_path = if current_path.is_empty() {
                component.to_string()
            } else {
                format!("{}/{}", current_path, component)
            };

            let title = self.get_title_for_path(&current_path)?;

            breadcrumb_items.push(BreadcrumbItem {
                title,
                url: current_path.clone(),
                is_current: false,
            });
        }

        // Add current page as last breadcrumb
        let current_url = relative_path
            .with_extension("")
            .to_string_lossy()
            .to_string();

        breadcrumb_items.push(BreadcrumbItem {
            title: file.title.clone(),
            url: current_url,
            is_current: true,
        });

        // Apply max_items limit if configured
        let max_items = self
            .config
            .as_ref()
            .and_then(|c| c.max_items)
            .unwrap_or(usize::MAX);
        
        // Only apply truncation if we have enough items for the logic to work
        if breadcrumb_items.len() > max_items && max_items >= 3 {
            // Keep first item (Home), add ellipsis, keep last (max_items - 2) items
            let last_items = breadcrumb_items.split_off(breadcrumb_items.len() - (max_items - 2));
            breadcrumb_items.truncate(1); // Keep only Home

            // Add ellipsis
            breadcrumb_items.push(BreadcrumbItem {
                title: "...".to_string(),
                url: String::new(),
                is_current: false,
            });

            breadcrumb_items.extend(last_items);
        }

        Ok(Some(BreadcrumbData {
            items: breadcrumb_items,
        }))
    }

    fn get_title_for_path(&self, path: &str) -> Result<String> {
        // Try to find an index.md file in the directory
        let index_path = self.input_dir.join(path).join("index.md");

        if index_path.exists() {
            // Parse the index file to extract title
            if let Ok(markdown_file) = MarkdownParser::parse_markdown_file(&index_path)
                && !markdown_file.title.is_empty()
            {
                return Ok(markdown_file.title);
            }
        }

        // Fallback to directory name with title case
        let dir_name = Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path);

        Ok(self.to_title_case(dir_name))
    }

    fn to_title_case(&self, text: &str) -> String {
        text.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}
