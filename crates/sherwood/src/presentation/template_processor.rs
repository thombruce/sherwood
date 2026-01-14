use crate::content::parsing::{Frontmatter, MarkdownFile};
use crate::content_generation::{ContentGenerator, DefaultContentGenerator};
use crate::templates::{ListData, TemplateDataEnum, TemplateManager};
use anyhow::Result;

/// Template type enumeration for routing to appropriate templates
#[derive(Debug, Clone)]
pub enum TemplateType {
    Default,
    External(String),
}

impl TemplateType {
    /// Resolve the template type based on frontmatter configuration
    pub fn resolve(frontmatter: &Frontmatter) -> Self {
        if let Some(template) = &frontmatter.page_template {
            match template.as_str() {
                "sherwood.stpl" => Self::Default,
                external => Self::External(external.to_string()),
            }
        } else {
            Self::Default
        }
    }

    /// Get the template file name for this template type
    pub fn template_name(&self) -> &str {
        match self {
            Self::Default => "sherwood.stpl",
            Self::External(name) => name,
        }
    }
}

/// Processor for handling template selection and rendering
/// Unifies the logic for processing markdown files with different templates
pub struct TemplateProcessor {
    template_manager: TemplateManager,
    breadcrumb_generator: Option<crate::partials::BreadcrumbGenerator>,
    content_generator: Box<dyn ContentGenerator>,
}

impl TemplateProcessor {
    /// Create a new TemplateProcessor
    pub fn new(
        template_manager: TemplateManager,
        breadcrumb_generator: Option<crate::partials::BreadcrumbGenerator>,
    ) -> Self {
        Self {
            template_manager,
            breadcrumb_generator,
            content_generator: Box::new(DefaultContentGenerator),
        }
    }

    /// Process a markdown file using the unified system
    /// This replaces all the separate process_* methods
    pub fn process_markdown_file(
        &self,
        file: &MarkdownFile,
        html_content: &str,
        list_data: Option<ListData>,
    ) -> Result<String> {
        // Resolve template type from frontmatter
        let template_type = TemplateType::resolve(&file.frontmatter);

        // Build the appropriate template data
        let template_data =
            self.build_for_template(&template_type, file, html_content, list_data)?;

        // Render using unified template manager
        self.template_manager
            .render_template(template_type.template_name(), template_data)
    }

    /// Build template data for the specified template type
    fn build_for_template(
        &self,
        template_type: &TemplateType,
        file: &MarkdownFile,
        html_content: &str,
        list_data: Option<ListData>,
    ) -> Result<TemplateDataEnum> {
        use crate::presentation::page_builder::PageBuilder;

        let breadcrumb_gen = self.breadcrumb_generator.as_ref();

        match template_type {
            TemplateType::Default => Ok(TemplateDataEnum::Page(
                PageBuilder::new(file, html_content, breadcrumb_gen)
                    .with_list_data(list_data)
                    .build_page(),
            )),
            TemplateType::External(_template_name) => {
                // For external templates, generate comprehensive page data including docs-specific fields
                let mut page_builder =
                    PageBuilder::new(file, html_content, breadcrumb_gen).with_list_data(list_data);

                // Generate docs-specific data using ContentGenerator
                let sidebar_nav = self.content_generator.generate_sidebar_nav(file);
                let original_content =
                    std::fs::read_to_string(&file.path).unwrap_or_else(|_| file.content.clone());
                let table_of_contents = self
                    .content_generator
                    .generate_table_of_contents(&original_content);
                let next_prev_nav = self.content_generator.generate_next_prev_nav(file);

                page_builder = page_builder
                    .with_sidebar_nav(sidebar_nav)
                    .with_table_of_contents(table_of_contents)
                    .with_next_prev_nav(next_prev_nav);

                Ok(TemplateDataEnum::Page(page_builder.build_page()))
            }
        }
    }
}
