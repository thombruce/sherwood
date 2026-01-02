use super::templates::TemplateManager;
use crate::content::parser::MarkdownFile;
use anyhow::Result;

pub struct PageGenerator {
    pub template_manager: TemplateManager,
}

impl PageGenerator {
    pub fn new(template_manager: TemplateManager) -> Self {
        Self { template_manager }
    }

    pub fn generate_html_document_with_template(
        &self,
        file: &MarkdownFile,
        content: &str,
        theme_name: &str,
        theme_variant: &str,
    ) -> Result<String> {
        let css_file = Some(format!("/css/{theme_name}.css", theme_name = theme_name));
        let body_attrs = if theme_variant != "default" {
            format!(r#" data-theme="{}""#, theme_variant)
        } else {
            String::new()
        };

        self.template_manager
            .render_page(&file.title, content, css_file.as_deref(), &body_attrs)
    }

    pub fn process_markdown_file(
        &self,
        file: &MarkdownFile,
        html_content: &str,
        theme_name: &str,
        theme_variant: &str,
    ) -> Result<String> {
        self.generate_html_document_with_template(file, html_content, theme_name, theme_variant)
    }
}
