use anyhow::Result;
use include_dir::{Dir, include_dir};
use sailfish::{TemplateOnce, runtime::RenderError};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {template_name}")]
    TemplateNotFound { template_name: String },

    #[error("Invalid template format: {template_name} - {details}")]
    InvalidTemplate {
        template_name: String,
        details: String,
    },

    #[error("Template validation failed: {template_name} - {reason}")]
    ValidationFailed {
        template_name: String,
        reason: String,
    },

    #[error("No templates directory found at: {path}")]
    TemplatesDirectoryNotFound { path: String },

    #[error("Template compilation error: {template_name} - {source}")]
    CompilationError {
        template_name: String,
        #[source]
        source: RenderError,
    },
}

// Embed templates directory at compile time
static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

#[derive(Serialize, Clone)]
pub struct ListItemData {
    pub title: String,
    pub url: String,
    pub date: Option<String>,
    pub excerpt: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct BreadcrumbItem {
    pub title: String,
    pub url: String,
    pub is_current: bool,
}

#[derive(Serialize, Clone)]
pub struct BreadcrumbData {
    pub items: Vec<BreadcrumbItem>,
}

#[derive(Serialize)]
pub struct ListData {
    pub items: Vec<ListItemData>,
    pub sort_config: crate::content::renderer::SortConfig,
    pub total_count: usize,
}

#[derive(TemplateOnce)]
#[template(path = "default.stpl")]
struct PageTemplate {
    title: String,
    content: String,
    css_file: Option<String>,
    body_attrs: String,
    list_data: Option<ListData>,
    breadcrumb_data: Option<BreadcrumbData>,
}

#[derive(Debug)]
pub struct TemplateInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct TemplateManager {
    templates_dir: PathBuf,
    available_templates: Vec<String>,
}

impl TemplateManager {
    pub fn new(templates_dir: &Path) -> Result<Self> {
        let templates_dir = templates_dir.to_path_buf();

        // Templates directory is optional - we have embedded fallbacks

        let available_templates = Self::discover_templates(&templates_dir)?;

        Ok(Self {
            templates_dir,
            available_templates,
        })
    }

    /// Discover all available template files in the templates directory
    fn discover_templates(templates_dir: &Path) -> Result<Vec<String>> {
        let mut templates = Vec::new();

        // First, add embedded templates
        templates.extend(get_available_templates());

        // Then scan the templates directory for additional templates
        if templates_dir.exists() && templates_dir.is_dir() {
            for entry in fs::read_dir(templates_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file()
                    && let Some(extension) = path.extension()
                    && extension == "stpl"
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                {
                    templates.push(name.to_string());
                }
            }
        }

        // Remove duplicates and sort
        templates.sort();
        templates.dedup();

        Ok(templates)
    }

    /// Get information about all available templates
    pub fn get_template_info(&self) -> Vec<TemplateInfo> {
        self.available_templates
            .iter()
            .map(|name| {
                let path = self.templates_dir.join(name);
                let size = if path.exists() {
                    fs::metadata(&path).map(|m| m.len() as usize).unwrap_or(0)
                } else {
                    // Check embedded templates
                    TEMPLATES
                        .get_file(name)
                        .map(|f| f.contents().len())
                        .unwrap_or(0)
                };

                TemplateInfo {
                    name: name.clone(),
                    path: path.clone(),
                    size,
                }
            })
            .collect()
    }

    /// Get list of available template names
    pub fn get_available_templates(&self) -> Vec<String> {
        self.available_templates.clone()
    }

    /// Print template information (for debugging)
    pub fn debug_print_templates(&self) {
        println!(
            "📋 Available templates in {}:",
            self.templates_dir.display()
        );
        let info = self.get_template_info();

        for template_info in info {
            println!(" {} ({} bytes)", template_info.name, template_info.size);
        }

        if self.available_templates.is_empty() {
            println!("  No templates found");
        }
    }

    pub fn render_page(
        &self,
        title: &str,
        content: &str,
        css_file: Option<&str>,
        body_attrs: &str,
    ) -> Result<String> {
        let template = PageTemplate {
            title: title.to_string(),
            content: content.to_string(),
            css_file: css_file.map(|s| s.to_string()),
            body_attrs: body_attrs.to_string(),
            list_data: None,
            breadcrumb_data: None,
        };

        Ok(template.render_once()?)
    }

    pub fn render_page_with_list(
        &self,
        title: &str,
        content: &str,
        css_file: Option<&str>,
        body_attrs: &str,
        list_data: Option<ListData>,
    ) -> Result<String> {
        let template = PageTemplate {
            title: title.to_string(),
            content: content.to_string(),
            css_file: css_file.map(|s| s.to_string()),
            body_attrs: body_attrs.to_string(),
            list_data,
            breadcrumb_data: None,
        };

        template
            .render_once()
            .map_err(|e| anyhow::anyhow!("Template render error: {}", e))
    }

    pub fn render_page_with_breadcrumb(
        &self,
        title: &str,
        content: &str,
        css_file: Option<&str>,
        body_attrs: &str,
        breadcrumb_data: Option<BreadcrumbData>,
    ) -> Result<String> {
        let template = PageTemplate {
            title: title.to_string(),
            content: content.to_string(),
            css_file: css_file.map(|s| s.to_string()),
            body_attrs: body_attrs.to_string(),
            list_data: None,
            breadcrumb_data,
        };

        template
            .render_once()
            .map_err(|e| anyhow::anyhow!("Template render error: {}", e))
    }

    pub fn render_page_with_list_and_breadcrumb(
        &self,
        title: &str,
        content: &str,
        css_file: Option<&str>,
        body_attrs: &str,
        list_data: Option<ListData>,
        breadcrumb_data: Option<BreadcrumbData>,
    ) -> Result<String> {
        let template = PageTemplate {
            title: title.to_string(),
            content: content.to_string(),
            css_file: css_file.map(|s| s.to_string()),
            body_attrs: body_attrs.to_string(),
            list_data,
            breadcrumb_data,
        };

        template
            .render_once()
            .map_err(|e| anyhow::anyhow!("Template render error: {}", e))
    }

    pub fn get_template_path(&self, template_name: &str) -> PathBuf {
        self.templates_dir.join(template_name)
    }

    pub fn template_exists(&self, template_name: &str) -> bool {
        self.templates_dir.join(template_name).exists()
    }
}

pub fn copy_embedded_templates(output_dir: &Path) -> Result<()> {
    let templates_output_dir = output_dir.join("templates");
    fs::create_dir_all(&templates_output_dir)?;

    for entry in TEMPLATES.entries() {
        if let Some(file) = entry.as_file() {
            let template_name = entry
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid template name"))?;

            let output_path = templates_output_dir.join(template_name);
            fs::write(
                &output_path,
                file.contents_utf8().ok_or_else(|| {
                    anyhow::anyhow!("Template {} contains invalid UTF-8", template_name)
                })?,
            )?;
        }
    }

    Ok(())
}

pub fn get_available_templates() -> Vec<String> {
    TEMPLATES
        .files()
        .map(|file| {
            file.path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect()
}
