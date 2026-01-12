use super::common::*;
use super::renderer::TemplateData;
use sailfish::TemplateOnce;
use serde::Serialize;

#[derive(TemplateOnce)]
#[template(path = "sherwood.stpl")]
pub struct SherwoodTemplate {
    pub title: String,
    pub content: String,
    pub css_file: Option<String>,
    pub body_attrs: String,
    pub breadcrumb_data: Option<BreadcrumbData>,
    pub list_data: Option<ListData>,
}

#[derive(Serialize, Clone)]
pub struct PageData {
    pub title: String,
    pub content: String,
    pub css_file: Option<String>,
    pub body_attrs: String,
    pub breadcrumb_data: Option<BreadcrumbData>,
    pub list_data: Option<ListData>,
}

impl TemplateData for PageData {
    fn get_title(&self) -> &str {
        &self.title
    }
    fn get_content(&self) -> &str {
        &self.content
    }
    fn get_css_file(&self) -> Option<&str> {
        self.css_file.as_deref()
    }
    fn get_body_attrs(&self) -> &str {
        &self.body_attrs
    }
    fn get_breadcrumb_data(&self) -> Option<&BreadcrumbData> {
        self.breadcrumb_data.as_ref()
    }
    fn get_list_data(&self) -> Option<&ListData> {
        self.list_data.as_ref()
    }
}
