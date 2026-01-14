use serde::Serialize;
use std::path::PathBuf;

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
        source: sailfish::runtime::RenderError,
    },
}

#[derive(Serialize, Clone, Debug)]
pub struct ListItemData {
    pub title: String,
    pub url: String,
    pub date: Option<String>,
    pub excerpt: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct BreadcrumbItem {
    pub title: String,
    pub url: String,
    pub is_current: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct BreadcrumbData {
    pub items: Vec<BreadcrumbItem>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ListData {
    pub items: Vec<ListItemData>,
    pub sort_config: crate::content::renderer::SortConfig,
    pub total_count: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct SidebarNavData {
    pub current_path: String,
    pub items: Vec<SidebarNavItem>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SidebarNavItem {
    pub title: String,
    pub url: String,
    pub is_current: bool,
    pub is_section: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct NextPrevNavData {
    pub previous: Option<ListItemData>,
    pub next: Option<ListItemData>,
}

#[derive(Debug)]
pub struct TemplateInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: usize,
}
