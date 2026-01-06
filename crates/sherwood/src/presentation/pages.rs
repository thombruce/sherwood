use super::templates::{ListData, TemplateManager};
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
    ) -> Result<String> {
        let css_file = Some("/css/main.css".to_string());
        let body_attrs = String::new();

        self.template_manager
            .render_page(&file.title, content, css_file.as_deref(), &body_attrs)
    }

    fn get_template_name<'a>(
        &self,
        frontmatter: &'a crate::content::parser::Frontmatter,
    ) -> &'a str {
        if let Some(template) = &frontmatter.page_template {
            // Check if the template exists
            if self.template_exists(template) {
                return template;
            } else {
                eprintln!(
                    "Warning: Template '{}' not found, using default template",
                    template
                );
            }
        }

        // Default template
        "default.stpl"
    }

    fn template_exists(&self, template_name: &str) -> bool {
        // First check if it's in the available templates list
        let available_templates = self.template_manager.get_available_templates();
        available_templates.contains(&template_name.to_string())
    }

    pub fn process_markdown_file(&self, file: &MarkdownFile, html_content: &str) -> Result<String> {
        // Get the appropriate template name based on frontmatter
        let template_name = self.get_template_name(&file.frontmatter);

        // For now, we still use the default template rendering logic
        // In the future, this could be extended to dynamically render different templates
        if template_name == "default.stpl" {
            self.generate_html_document_with_template(file, html_content)
        } else {
            // Log that we're using the default template for now
            eprintln!(
                "Note: Template '{}' specified but dynamic template rendering not yet implemented. Using default template.",
                template_name
            );
            self.generate_html_document_with_template(file, html_content)
        }
    }

    pub fn process_markdown_file_with_list(
        &self,
        file: &MarkdownFile,
        html_content: &str,
        list_data: Option<ListData>,
    ) -> Result<String> {
        let title = file.frontmatter.title.as_deref().unwrap_or(&file.title);
        let css_file = Some("/css/main.css".to_string());
        let body_attrs = String::new();

        self.template_manager.render_page_with_list(
            title,
            html_content,
            css_file.as_deref(),
            &body_attrs,
            list_data,
        )
    }
}
