use crate::content::parsing::MarkdownFile;
use crate::content::parsing::ast_utils::extract_text_from_nodes;
use crate::core::markdown_config;
use crate::templates::{NextPrevNavData, SidebarNavData};
use markdown::mdast::Node;
use markdown::to_mdast;

/// Generic trait for generating content-specific page components.
/// This trait is designed to be extensible for future plugins and custom generators.
pub trait ContentGenerator {
    /// Generate sidebar navigation for the given file
    fn generate_sidebar_nav(&self, _file: &MarkdownFile) -> Option<SidebarNavData> {
        None // Default implementation
    }

    /// Generate table of contents from the given content
    fn generate_table_of_contents(&self, _content: &str) -> Option<String> {
        None // Default implementation
    }

    /// Generate next/previous navigation for the given file
    fn generate_next_prev_nav(&self, _file: &MarkdownFile) -> Option<NextPrevNavData> {
        None // Default implementation
    }
}

/// Default implementation of ContentGenerator with standard Sherwood behavior
pub struct DefaultContentGenerator;

impl ContentGenerator for DefaultContentGenerator {
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

    fn generate_table_of_contents(&self, content: &str) -> Option<String> {
        let root = to_mdast(content, &markdown_config::default()).ok()?;

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

    fn generate_next_prev_nav(&self, _file: &MarkdownFile) -> Option<NextPrevNavData> {
        // TODO:
        // Basic implementation - will be enhanced later
        // For now, return None so the section doesn't render
        None
    }
}

impl DefaultContentGenerator {
    /// Extract plain text from a markdown AST node
    fn extract_text_from_node(&self, node: &Node) -> String {
        match node {
            Node::Root(root) => extract_text_from_nodes(&root.children),
            Node::Blockquote(quote) => extract_text_from_nodes(&quote.children),
            Node::List(list) => extract_text_from_nodes(&list.children),
            Node::ListItem(item) => extract_text_from_nodes(&item.children),
            Node::Definition(_def) => String::new(),
            Node::Paragraph(para) => extract_text_from_nodes(&para.children),
            Node::Heading(heading) => extract_text_from_nodes(&heading.children),
            Node::Table(_table) => String::new(),
            Node::TableRow(_row) => String::new(),
            Node::TableCell(_cell) => String::new(),
            Node::Html(_html) => String::new(),
            Node::Code(_code) => String::new(),
            Node::Yaml(_yaml) => String::new(),
            Node::Toml(_toml) => String::new(),
            Node::InlineMath(_math) => String::new(),
            Node::Math(_math) => String::new(),
            Node::MdxJsxFlowElement(_jsx) => String::new(),
            Node::MdxFlowExpression(_expr) => String::new(),
            Node::MdxTextExpression(_expr) => String::new(),
            // Extract text from inline elements
            Node::Emphasis(emphasis) => extract_text_from_nodes(&emphasis.children),
            Node::Strong(strong) => extract_text_from_nodes(&strong.children),
            Node::Delete(delete) => extract_text_from_nodes(&delete.children),
            Node::InlineCode(code) => code.value.clone(),
            Node::Break(_break) => String::new(),
            Node::Link(link) => extract_text_from_nodes(&link.children),
            Node::Image(image) => image.alt.clone(),
            Node::LinkReference(_reference) => String::new(),
            Node::ImageReference(_reference) => String::new(),
            Node::FootnoteReference(_reference) => String::new(),
            Node::FootnoteDefinition(_definition) => String::new(),
            // Handle remaining variants
            Node::Text(text) => text.value.clone(),
            Node::ThematicBreak(_) => String::new(),
            Node::MdxjsEsm(_) => String::new(),
            Node::MdxJsxTextElement(_) => String::new(),
        }
    }

    /// Create an anchor link from text by converting to kebab-case
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
}
