use super::partials::{FooterData, HeaderData};
use sailfish::TemplateOnce;
use sherwood::templates::common::*;
use sherwood::templates::{FromTemplateData, TemplateData, TemplateDataEnum};

#[derive(TemplateOnce, Debug)]
#[template(path = "docs.stpl")]
pub struct DocsTemplate {
    pub title: String,
    pub content: String,
    pub css_file: Option<String>,
    pub body_attrs: String,
    pub header_data: Option<HeaderData>,
    pub footer_data: Option<FooterData>,
    pub breadcrumb_data: Option<BreadcrumbData>,
    pub sidebar_nav: Option<SidebarNavData>,
    pub table_of_contents: Option<String>,
    pub next_prev_nav: Option<NextPrevNavData>,
}

#[derive(serde::Serialize, Clone)]
#[allow(dead_code)] // Part of public API for external use
pub struct DocsPageData {
    pub title: String,
    pub content: String,
    pub css_file: Option<String>,
    pub body_attrs: String,
    pub header_data: Option<HeaderData>,
    pub footer_data: Option<FooterData>,
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

impl FromTemplateData for DocsTemplate {
    fn from(data: TemplateDataEnum) -> Self {
        Self {
            title: data.get_title().to_string(),
            content: data.get_content().to_string(),
            css_file: data.get_css_file().map(|s| s.to_string()),
            body_attrs: data.get_body_attrs().to_string(),
            header_data: {
                match &data {
                    TemplateDataEnum::Page(page_data) => {
                        if let Some(docs_page) =
                            (page_data as &dyn std::any::Any).downcast_ref::<DocsPageData>()
                        {
                            docs_page.header_data.clone()
                        } else {
                            Some(HeaderData::default())
                        }
                    }
                }
            },
            footer_data: {
                match &data {
                    TemplateDataEnum::Page(page_data) => {
                        if let Some(docs_page) =
                            (page_data as &dyn std::any::Any).downcast_ref::<DocsPageData>()
                        {
                            docs_page.footer_data.clone()
                        } else {
                            Some(FooterData::default())
                        }
                    }
                }
            },
            breadcrumb_data: data.get_breadcrumb_data().cloned(),
            sidebar_nav: data.get_sidebar_nav().cloned(),
            table_of_contents: data.get_table_of_contents().map(|s| s.to_string()),
            next_prev_nav: data.get_next_prev_nav().cloned(),
        }
    }
}
