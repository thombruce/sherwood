use super::common::*;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::{registry::TemplateRegistry, sherwood::PageData, sherwood::SherwoodTemplate};
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

#[derive(Clone)]
pub enum TemplateDataEnum {
    Page(PageData),
}

impl TemplateData for TemplateDataEnum {
    fn get_title(&self) -> &str {
        match self {
            TemplateDataEnum::Page(data) => data.get_title(),
        }
    }

    fn get_content(&self) -> &str {
        match self {
            TemplateDataEnum::Page(data) => data.get_content(),
        }
    }

    fn get_css_file(&self) -> Option<&str> {
        match self {
            TemplateDataEnum::Page(data) => data.get_css_file(),
        }
    }

    fn get_body_attrs(&self) -> &str {
        match self {
            TemplateDataEnum::Page(data) => data.get_body_attrs(),
        }
    }

    fn get_breadcrumb_data(&self) -> Option<&BreadcrumbData> {
        match self {
            TemplateDataEnum::Page(data) => data.get_breadcrumb_data(),
        }
    }

    fn get_list_data(&self) -> Option<&ListData> {
        match self {
            TemplateDataEnum::Page(data) => data.get_list_data(),
        }
    }

    fn get_sidebar_nav(&self) -> Option<&SidebarNavData> {
        match self {
            TemplateDataEnum::Page(data) => data.get_sidebar_nav(),
        }
    }

    fn get_table_of_contents(&self) -> Option<&str> {
        match self {
            TemplateDataEnum::Page(data) => data.get_table_of_contents(),
        }
    }

    fn get_next_prev_nav(&self) -> Option<&NextPrevNavData> {
        match self {
            TemplateDataEnum::Page(data) => data.get_next_prev_nav(),
        }
    }
}

#[derive(Debug)]
pub struct TemplateManager {
    templates_dir: PathBuf,
    available_templates: Vec<String>,
    registry: Option<std::sync::Arc<TemplateRegistry>>,
}

impl Clone for TemplateManager {
    fn clone(&self) -> Self {
        Self {
            templates_dir: self.templates_dir.clone(),
            available_templates: self.available_templates.clone(),
            registry: self.registry.clone(),
        }
    }
}

impl TemplateManager {
    pub fn new(templates_dir: &Path) -> Result<Self> {
        Self::new_with_registry(templates_dir, None)
    }

    pub fn new_with_registry(
        templates_dir: &Path,
        registry: Option<TemplateRegistry>,
    ) -> Result<Self> {
        let templates_dir = templates_dir.to_path_buf();
        let registry = registry.map(std::sync::Arc::new);

        let available_templates =
            Self::discover_templates(&templates_dir, registry.as_ref().map(|r| r.as_ref()))?;

        Ok(Self {
            templates_dir,
            available_templates,
            registry,
        })
    }

    /// Discover all available template files in the templates directory
    fn discover_templates(
        templates_dir: &Path,
        registry: Option<&TemplateRegistry>,
    ) -> Result<Vec<String>> {
        let mut templates = Vec::new();

        // Add registered templates from registry
        if let Some(registry) = registry {
            templates.extend(registry.registered_templates());
        }

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
                    0
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
        // First try custom templates from registry
        if let Some(registry) = &self.registry
            && let Some(renderer) = registry.get_renderer(template_name)
        {
            return renderer.render(&data);
        }

        // Fallback to built-in templates
        match template_name {
            "sherwood.stpl" => self.render_sherwood_template(data),
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
}
