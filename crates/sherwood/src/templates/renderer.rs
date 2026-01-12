use super::common::*;
use anyhow::Result;
use include_dir::{Dir, include_dir};
use std::fs;
use std::path::{Path, PathBuf};

// Embed templates directory at compile time
static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

use super::{
    docs::DocsPageData, docs::DocsTemplate, sherwood::PageData, sherwood::SherwoodTemplate,
};
use sailfish::TemplateOnce;

pub trait TemplateData {
    // Core required fields
    fn get_title(&self) -> &str;
    fn get_content(&self) -> &str;
    fn get_css_file(&self) -> Option<&str>;
    fn get_body_attrs(&self) -> &str;

    // Optional sections - sherwood to None
    fn get_breadcrumb_data(&self) -> Option<&BreadcrumbData> {
        None
    }
    fn get_list_data(&self) -> Option<&ListData> {
        None
    }
    fn get_sidebar_nav(&self) -> Option<&SidebarNavData> {
        None
    }
    fn get_table_of_contents(&self) -> Option<&str> {
        None
    }
    fn get_next_prev_nav(&self) -> Option<&NextPrevNavData> {
        None
    }
}

pub enum TemplateDataEnum {
    Page(PageData),
    Docs(DocsPageData),
}

impl TemplateData for TemplateDataEnum {
    fn get_title(&self) -> &str {
        match self {
            TemplateDataEnum::Page(data) => data.get_title(),
            TemplateDataEnum::Docs(data) => data.get_title(),
        }
    }

    fn get_content(&self) -> &str {
        match self {
            TemplateDataEnum::Page(data) => data.get_content(),
            TemplateDataEnum::Docs(data) => data.get_content(),
        }
    }

    fn get_css_file(&self) -> Option<&str> {
        match self {
            TemplateDataEnum::Page(data) => data.get_css_file(),
            TemplateDataEnum::Docs(data) => data.get_css_file(),
        }
    }

    fn get_body_attrs(&self) -> &str {
        match self {
            TemplateDataEnum::Page(data) => data.get_body_attrs(),
            TemplateDataEnum::Docs(data) => data.get_body_attrs(),
        }
    }

    fn get_breadcrumb_data(&self) -> Option<&BreadcrumbData> {
        match self {
            TemplateDataEnum::Page(data) => data.get_breadcrumb_data(),
            TemplateDataEnum::Docs(data) => data.get_breadcrumb_data(),
        }
    }

    fn get_list_data(&self) -> Option<&ListData> {
        match self {
            TemplateDataEnum::Page(data) => data.get_list_data(),
            TemplateDataEnum::Docs(data) => data.get_list_data(),
        }
    }

    fn get_sidebar_nav(&self) -> Option<&SidebarNavData> {
        match self {
            TemplateDataEnum::Page(data) => data.get_sidebar_nav(),
            TemplateDataEnum::Docs(data) => data.get_sidebar_nav(),
        }
    }

    fn get_table_of_contents(&self) -> Option<&str> {
        match self {
            TemplateDataEnum::Page(data) => data.get_table_of_contents(),
            TemplateDataEnum::Docs(data) => data.get_table_of_contents(),
        }
    }

    fn get_next_prev_nav(&self) -> Option<&NextPrevNavData> {
        match self {
            TemplateDataEnum::Page(data) => data.get_next_prev_nav(),
            TemplateDataEnum::Docs(data) => data.get_next_prev_nav(),
        }
    }
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

    /// Single unified render function that handles all template types
    pub fn render_template(&self, template_name: &str, data: TemplateDataEnum) -> Result<String> {
        match template_name {
            "sherwood.stpl" => self.render_sherwood_template(data),
            "docs.stpl" => self.render_docs_template(data),
            _ => Err(TemplateError::TemplateNotFound {
                template_name: template_name.to_string(),
            }
            .into()),
        }
    }

    pub fn get_template_path(&self, template_name: &str) -> PathBuf {
        self.templates_dir.join(template_name)
    }

    pub fn template_exists(&self, template_name: &str) -> bool {
        self.templates_dir.join(template_name).exists()
    }

    fn render_sherwood_template(&self, data: TemplateDataEnum) -> Result<String> {
        let template = SherwoodTemplate {
            title: data.get_title().to_string(),
            content: data.get_content().to_string(),
            css_file: data.get_css_file().map(|s| s.to_string()),
            body_attrs: data.get_body_attrs().to_string(),
            breadcrumb_data: data.get_breadcrumb_data().cloned(),
            list_data: data.get_list_data().cloned(),
        };

        template.render_once().map_err(|e| {
            TemplateError::CompilationError {
                template_name: "sherwood.stpl".to_string(),
                source: e,
            }
            .into()
        })
    }

    fn render_docs_template(&self, data: TemplateDataEnum) -> Result<String> {
        let template = DocsTemplate {
            title: data.get_title().to_string(),
            content: data.get_content().to_string(),
            css_file: data.get_css_file().map(|s| s.to_string()),
            body_attrs: data.get_body_attrs().to_string(),
            breadcrumb_data: data.get_breadcrumb_data().cloned(),
            sidebar_nav: data.get_sidebar_nav().cloned(),
            table_of_contents: data.get_table_of_contents().map(|s| s.to_string()),
            next_prev_nav: data.get_next_prev_nav().cloned(),
        };

        template.render_once().map_err(|e| {
            TemplateError::CompilationError {
                template_name: "docs.stpl".to_string(),
                source: e,
            }
            .into()
        })
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
