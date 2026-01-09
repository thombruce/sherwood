use crate::content::parser::MarkdownFile;
use crate::partials::BreadcrumbGenerator;
use crate::templates::{
    DocsPageData, ListData, NextPrevNavData, PageData, SidebarNavData, TemplateDataEnum,
    TemplateManager,
};
use anyhow::Result;
use markdown::mdast::Node;
use markdown::{ParseOptions, to_mdast};

pub struct PageGenerator {
    pub template_manager: TemplateManager,
    pub breadcrumb_generator: Option<BreadcrumbGenerator>,
}

impl PageGenerator {
    pub fn new(template_manager: TemplateManager) -> Self {
        Self {
            template_manager,
            breadcrumb_generator: None,
        }
    }

    pub fn new_with_breadcrumb(
        template_manager: TemplateManager,
        breadcrumb_generator: Option<BreadcrumbGenerator>,
    ) -> Self {
        Self {
            template_manager,
            breadcrumb_generator,
        }
    }

    pub fn generate_html_document_with_template(
        &self,
        file: &MarkdownFile,
        content: &str,
    ) -> Result<String> {
        let css_file = Some("/css/main.css".to_string());
        let body_attrs = String::new();

        // Generate breadcrumb if generator is available
        let breadcrumb_data = if let Some(ref generator) = self.breadcrumb_generator {
            generator.generate_breadcrumb(file)?
        } else {
            None
        };

        let page_data = PageData {
            title: file.title.clone(),
            content: content.to_string(),
            css_file,
            body_attrs,
            breadcrumb_data,
            list_data: None,
        };

        self.template_manager
            .render_template("default.stpl", TemplateDataEnum::Page(page_data))
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

        match template_name {
            "default.stpl" => self.generate_html_document_with_template(file, html_content),
            "docs.stpl" => self.generate_docs_page(file, html_content),
            _ => {
                eprintln!(
                    "Warning: Unknown template '{}', using default template",
                    template_name
                );
                self.generate_html_document_with_template(file, html_content)
            }
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

        // Generate breadcrumb if generator is available
        let breadcrumb_data = if let Some(ref generator) = self.breadcrumb_generator {
            generator.generate_breadcrumb(file)?
        } else {
            None
        };

        let page_data = PageData {
            title: title.to_string(),
            content: html_content.to_string(),
            css_file,
            body_attrs,
            breadcrumb_data,
            list_data,
        };

        self.template_manager
            .render_template("default.stpl", TemplateDataEnum::Page(page_data))
    }

    fn generate_docs_page(&self, file: &MarkdownFile, html_content: &str) -> Result<String> {
        let title = file.frontmatter.title.as_deref().unwrap_or(&file.title);
        let css_file = Some("/css/main.css".to_string());
        let body_attrs = String::new();

        // Generate breadcrumb if generator is available
        let breadcrumb_data = if let Some(ref generator) = self.breadcrumb_generator {
            generator.generate_breadcrumb(file)?
        } else {
            None
        };

        // Generate sidebar navigation (basic implementation for now)
        let sidebar_nav = self.generate_sidebar_nav(file);

        // Generate table of contents from headings - use original file content
        let original_content =
            std::fs::read_to_string(&file.path).unwrap_or_else(|_| file.content.clone());
        let table_of_contents = self.generate_toc_from_content(&original_content);

        // Generate next/previous navigation
        let next_prev_nav = self.generate_next_prev_nav(file);

        let page_data = DocsPageData {
            title: title.to_string(),
            content: html_content.to_string(),
            css_file,
            body_attrs,
            breadcrumb_data,
            sidebar_nav,
            table_of_contents,
            next_prev_nav,
        };

        self.template_manager
            .render_template("docs.stpl", TemplateDataEnum::Docs(page_data))
    }

    fn generate_sidebar_nav(&self, file: &MarkdownFile) -> Option<SidebarNavData> {
        // Basic implementation - will be enhanced later
        let current_path = file
            .path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("docs");

        Some(SidebarNavData {
            current_path: current_path.to_string(),
            items: vec![
                crate::templates::SidebarNavItem {
                    title: "Documentation".to_string(),
                    url: "docs".to_string(),
                    is_current: current_path == "docs",
                    is_section: true,
                },
                crate::templates::SidebarNavItem {
                    title: "Getting Started".to_string(),
                    url: "docs/getting-started".to_string(),
                    is_current: file.path.ends_with("getting-started.md"),
                    is_section: false,
                },
                crate::templates::SidebarNavItem {
                    title: "Frontmatter".to_string(),
                    url: "docs/frontmatter".to_string(),
                    is_current: file.path.ends_with("frontmatter.md"),
                    is_section: false,
                },
                crate::templates::SidebarNavItem {
                    title: "CLI Commands".to_string(),
                    url: "docs/cli-commands".to_string(),
                    is_current: file.path.ends_with("cli-commands.md"),
                    is_section: false,
                },
                crate::templates::SidebarNavItem {
                    title: "Deployment".to_string(),
                    url: "docs/deployment".to_string(),
                    is_current: file.path.ends_with("deployment.md"),
                    is_section: false,
                },
            ],
        })
    }

    fn generate_toc_from_content(&self, content: &str) -> Option<String> {
        let root = to_mdast(content, &ParseOptions::default()).ok()?;

        let mut toc_html = String::from("<ul class=\"toc-list\">");
        let mut has_items = false;

        if let Node::Root(root_node) = root {
            for child in &root_node.children {
                if let Node::Heading(heading) = child
                    && heading.depth >= 2
                    && heading.depth <= 3
                {
                    has_items = true;
                    let text = self.extract_text_from_node(child);
                    let anchor = self.create_anchor(&text);
                    let class = if heading.depth == 2 {
                        "toc-h2"
                    } else {
                        "toc-h3"
                    };
                    toc_html.push_str(&format!(
                        "<li class=\"{}\"><a href=\"#{}\">{}</a></li>",
                        class, anchor, text
                    ));
                }
            }
        }

        toc_html.push_str("</ul>");

        if has_items { Some(toc_html) } else { None }
    }

    fn extract_text_from_node(&self, node: &Node) -> String {
        match node {
            Node::Root(root) => Self::extract_text_from_nodes(&root.children),
            Node::Blockquote(quote) => Self::extract_text_from_nodes(&quote.children),
            Node::List(list) => Self::extract_text_from_nodes(&list.children),
            Node::ListItem(item) => Self::extract_text_from_nodes(&item.children),
            Node::Definition(_def) => String::new(),
            Node::Paragraph(para) => Self::extract_text_from_nodes(&para.children),
            Node::Heading(heading) => Self::extract_text_from_nodes(&heading.children),
            Node::Table(_table) => String::new(),
            Node::TableRow(_row) => String::new(),
            Node::TableCell(_cell) => String::new(),
            Node::Html(_html) => String::new(),
            Node::Code(code) => code.value.clone(),
            Node::Math(_math) => String::new(),
            Node::Yaml(_yaml) => String::new(),
            Node::Toml(_toml) => String::new(),
            Node::Text(text) => text.value.clone(),
            Node::Emphasis(emphasis) => Self::extract_text_from_nodes(&emphasis.children),
            Node::Strong(strong) => Self::extract_text_from_nodes(&strong.children),
            Node::Delete(delete) => Self::extract_text_from_nodes(&delete.children),
            Node::InlineCode(code) => code.value.clone(),
            Node::Break(_break) => String::new(),
            Node::Link(link) => Self::extract_text_from_nodes(&link.children),
            Node::Image(image) => image.alt.clone(),
            Node::FootnoteReference(_footnote) => String::new(),
            Node::FootnoteDefinition(_def) => String::new(),
            Node::InlineMath(math) => math.value.clone(),
            Node::MdxTextExpression(_) | Node::MdxJsxTextElement(_) => String::new(),
            _ => String::new(),
        }
    }

    fn extract_text_from_nodes(nodes: &[Node]) -> String {
        nodes
            .iter()
            .map(Self::extract_text_from_node_static)
            .collect::<Vec<String>>()
            .join("")
    }

    fn extract_text_from_node_static(node: &Node) -> String {
        match node {
            Node::Text(text) => text.value.clone(),
            Node::Emphasis(emphasis) => Self::extract_text_from_nodes(&emphasis.children),
            Node::Strong(strong) => Self::extract_text_from_nodes(&strong.children),
            Node::Delete(delete) => Self::extract_text_from_nodes(&delete.children),
            Node::InlineCode(code) => code.value.clone(),
            Node::Link(link) => Self::extract_text_from_nodes(&link.children),
            Node::Image(image) => image.alt.clone(),
            Node::InlineMath(math) => math.value.clone(),
            _ => String::new(),
        }
    }

    fn create_anchor(&self, text: &str) -> String {
        // Simple kebab-case conversion
        text.to_lowercase()
            .chars()
            .map(|c| match c {
                'a'..='z' | '0'..='9' => c,
                ' ' | '_' | '-' => '-',
                _ => '-',
            })
            .collect::<String>()
            .replace("--", "-")
            .trim_matches('-')
            .to_string()
    }

    fn generate_next_prev_nav(&self, _file: &MarkdownFile) -> Option<NextPrevNavData> {
        // TODO:
        // Basic implementation - will be enhanced later
        // For now, return None so the section doesn't render
        None
    }
}
