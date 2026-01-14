use super::common::*;
use super::registry::FromTemplateData;
use super::renderer::{TemplateData, TemplateDataEnum};
use sailfish::TemplateOnce;
use serde::Serialize;

#[derive(TemplateOnce, Debug)]
#[template(path = "sherwood.stpl")]
#[allow(unused)] // Some fields unused by sherwood template but available for docs templates
pub struct SherwoodTemplate {
    pub title: String,
    pub content: String,
    pub css_file: Option<String>,
    pub body_attrs: String,
    pub breadcrumb_data: Option<BreadcrumbData>,
    pub list_data: Option<ListData>,
    // Docs-specific fields - available but unused by sherwood template
    // These fields are populated but not rendered in sherwood.stpl
    // Using underscore prefix to indicate intentionally unused in template
    // pub sidebar_nav: Option<SidebarNavData>,
    // pub table_of_contents: Option<String>,
    // pub next_prev_nav: Option<NextPrevNavData>,
}

#[derive(Serialize, Clone)]
pub struct PageData {
    pub title: String,
    pub content: String,
    pub css_file: Option<String>,
    pub body_attrs: String,
    pub breadcrumb_data: Option<BreadcrumbData>,
    pub list_data: Option<ListData>,
    // Docs-specific fields - now available to all templates
    pub sidebar_nav: Option<SidebarNavData>,
    pub table_of_contents: Option<String>,
    pub next_prev_nav: Option<NextPrevNavData>,
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
    fn get_sidebar_nav(&self) -> Option<&SidebarNavData> {
        self.sidebar_nav.as_ref()
    }
    fn get_table_of_contents(&self) -> Option<&str> {
        self.table_of_contents.as_deref()
    }
    fn get_next_prev_nav(&self) -> Option<&NextPrevNavData> {
        self.next_prev_nav.as_ref()
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
