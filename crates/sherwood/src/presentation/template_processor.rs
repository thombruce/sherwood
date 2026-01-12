use crate::content::parsing::{Frontmatter, MarkdownFile};
use crate::content_generation::{ContentGenerator, DefaultContentGenerator};
use crate::templates::{ListData, TemplateDataEnum, TemplateManager};
use anyhow::Result;

/// Template type enumeration for routing to appropriate templates
#[derive(Debug, Clone)]
pub enum TemplateType {
    Default,
    Docs,
    Custom(String),
}

impl TemplateType {
    /// Resolve the template type based on frontmatter configuration
    pub fn resolve(frontmatter: &Frontmatter) -> Self {
        if let Some(template) = &frontmatter.page_template {
            match template.as_str() {
                "sherwood.stpl" => Self::Default,
                "docs.stpl" => Self::Docs,
                custom => Self::Custom(custom.to_string()),
            }
        } else {
            Self::Default
        }
    }

    /// Get the template file name for this template type
    pub fn template_name(&self) -> &str {
        match self {
            Self::Default => "sherwood.stpl",
            Self::Docs => "docs.stpl",
            Self::Custom(name) => name,
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
    /// Create a new TemplateProcessor with default content generator
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

    /// Create a new TemplateProcessor with custom content generator
    pub fn with_content_generator(
        template_manager: TemplateManager,
        breadcrumb_generator: Option<crate::partials::BreadcrumbGenerator>,
        content_generator: Box<dyn ContentGenerator>,
    ) -> Self {
        Self {
            template_manager,
            breadcrumb_generator,
            content_generator,
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
            TemplateType::Docs => {
                // Generate docs-specific content
                let sidebar_nav = self.content_generator.generate_sidebar_nav(file);

                // Generate table of contents from original file content
                let original_content =
                    std::fs::read_to_string(&file.path).unwrap_or_else(|_| file.content.clone());
                let table_of_contents = self
                    .content_generator
                    .generate_table_of_contents(&original_content);

                // Generate next/previous navigation
                let next_prev_nav = self.content_generator.generate_next_prev_nav(file);

                Ok(TemplateDataEnum::Docs(
                    PageBuilder::new(file, html_content, breadcrumb_gen)
                        .with_sidebar_nav(sidebar_nav)
                        .with_table_of_contents(table_of_contents)
                        .with_next_prev_nav(next_prev_nav)
                        .build_docs(),
                ))
            }
            TemplateType::Custom(template_name) => {
                eprintln!(
                    "Warning: Unknown template '{}', using default template",
                    template_name
                );

                Ok(TemplateDataEnum::Page(
                    PageBuilder::new(file, html_content, breadcrumb_gen)
                        .with_list_data(list_data)
                        .build_page(),
                ))
            }
        }
    }
}
