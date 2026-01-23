use crate::config::SiteSection;
use crate::content::parsing::MarkdownFile;
use crate::partials::BreadcrumbGenerator;
use crate::templates::{BreadcrumbData, ListData, NextPrevNavData, PageData, SidebarNavData};

/// Builder pattern for constructing page data structures
/// Eliminates duplication in page building logic and provides a fluent API
pub struct PageBuilder<'a> {
    file: &'a MarkdownFile,
    content: &'a str,
    breadcrumb_generator: Option<&'a BreadcrumbGenerator>,
    css_file: Option<String>,
    body_attrs: String,
    site_config: &'a SiteSection,
    list_data: Option<ListData>,
    sidebar_nav: Option<SidebarNavData>,
    table_of_contents: Option<String>,
    next_prev_nav: Option<NextPrevNavData>,
}

impl<'a> PageBuilder<'a> {
    /// Create a new PageBuilder with default values
    pub fn new(
        file: &'a MarkdownFile,
        content: &'a str,
        breadcrumb_generator: Option<&'a BreadcrumbGenerator>,
        site_config: &'a SiteSection,
    ) -> Self {
        Self {
            file,
            content,
            breadcrumb_generator,
            css_file: Some("/css/main.css".to_string()),
            body_attrs: String::new(),
            site_config,
            list_data: None,
            sidebar_nav: None,
            table_of_contents: None,
            next_prev_nav: None,
        }
    }

    /// Set custom CSS file
    pub fn with_css_file(mut self, css_file: Option<String>) -> Self {
        self.css_file = css_file;
        self
    }

    /// Set custom body attributes
    pub fn with_body_attrs(mut self, body_attrs: String) -> Self {
        self.body_attrs = body_attrs;
        self
    }

    /// Set list data for content listings
    pub fn with_list_data(mut self, list_data: Option<ListData>) -> Self {
        self.list_data = list_data;
        self
    }

    /// Set sidebar navigation data
    pub fn with_sidebar_nav(mut self, sidebar_nav: Option<SidebarNavData>) -> Self {
        self.sidebar_nav = sidebar_nav;
        self
    }

    /// Set table of contents data
    pub fn with_table_of_contents(mut self, table_of_contents: Option<String>) -> Self {
        self.table_of_contents = table_of_contents;
        self
    }

    /// Set next/previous navigation data
    pub fn with_next_prev_nav(mut self, next_prev_nav: Option<NextPrevNavData>) -> Self {
        self.next_prev_nav = next_prev_nav;
        self
    }

    /// Generate breadcrumb data if generator is available
    fn generate_breadcrumb_data(&self) -> Option<BreadcrumbData> {
        if let Some(generator) = self.breadcrumb_generator {
            generator.generate_breadcrumb(self.file).ok()?
        } else {
            None
        }
    }

    /// Build PageData for default templates
    pub fn build_page(self) -> PageData {
        let title = self
            .file
            .frontmatter
            .title
            .as_deref()
            .unwrap_or(&self.file.title);
        let breadcrumb_data = self.generate_breadcrumb_data();

        PageData {
            title: title.to_string(),
            content: self.content.to_string(),
            css_file: self.css_file,
            body_attrs: self.body_attrs,
            site_title: self.site_config.title.clone(),
            footer_text: self.site_config.footer_text.clone(),
            breadcrumb_data,
            list_data: self.list_data,
            sidebar_nav: self.sidebar_nav,
            table_of_contents: self.table_of_contents,
            next_prev_nav: self.next_prev_nav,
        }
    }
}
