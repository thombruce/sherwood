use super::common::*;
use super::renderer::TemplateData;
use sailfish::TemplateOnce;
use serde::Serialize;

#[derive(TemplateOnce)]
#[template(path = "docs.stpl")]
pub struct DocsTemplate {
    pub title: String,
    pub content: String,
    pub css_file: Option<String>,
    pub body_attrs: String,
    pub breadcrumb_data: Option<BreadcrumbData>,
    pub sidebar_nav: Option<SidebarNavData>,
    pub table_of_contents: Option<String>,
    pub next_prev_nav: Option<NextPrevNavData>,
}

#[derive(Serialize, Clone)]
pub struct DocsPageData {
    pub title: String,
    pub content: String,
    pub css_file: Option<String>,
    pub body_attrs: String,
    pub breadcrumb_data: Option<BreadcrumbData>,
    pub sidebar_nav: Option<SidebarNavData>,
    pub table_of_contents: Option<String>,
    pub next_prev_nav: Option<NextPrevNavData>,
}

impl TemplateData for DocsPageData {
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
