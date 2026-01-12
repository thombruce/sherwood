use super::common::*;
use super::registry::FromTemplateData;
use super::renderer::{TemplateData, TemplateDataEnum};
use sailfish::TemplateOnce;
use serde::Serialize;

#[derive(TemplateOnce, Debug)]
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

impl FromTemplateData for SherwoodTemplate {
    fn from(data: TemplateDataEnum) -> Self {
        Self {
            title: data.get_title().to_string(),
            content: data.get_content().to_string(),
            css_file: data.get_css_file().map(|s| s.to_string()),
            body_attrs: data.get_body_attrs().to_string(),
            breadcrumb_data: data.get_breadcrumb_data().cloned(),
            list_data: data.get_list_data().cloned(),
        }
    }
}
