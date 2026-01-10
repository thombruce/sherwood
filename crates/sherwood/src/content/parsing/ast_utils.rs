use markdown::mdast::Node;

/// Extract plain text content from AST nodes recursively
/// Strips all formatting (bold, italic, code, links, etc.) and returns plain text
pub fn extract_text_from_nodes(nodes: &[Node]) -> String {
    nodes
        .iter()
        .map(|node| match node {
            Node::Text(text) => text.value.clone(),
            Node::Emphasis(emphasis) => extract_text_from_nodes(&emphasis.children),
            Node::Strong(strong) => extract_text_from_nodes(&strong.children),
            Node::InlineCode(code) => code.value.clone(),
            Node::Delete(delete) => extract_text_from_nodes(&delete.children),
            Node::Link(link) => extract_text_from_nodes(&link.children),
            Node::Image(image) => {
                // Use alt text for images in headings
                image.alt.clone()
            }
            Node::InlineMath(math) => math.value.clone(),
            // MDX nodes
            Node::MdxTextExpression(_) | Node::MdxJsxTextElement(_) => {
                // For MDX content, we'll extract text if possible or skip
                String::new()
            }
            _ => String::new(),
        })
        .collect::<Vec<String>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::mdast::{Emphasis, Image, InlineCode, Link, Strong, Text};
    use markdown::{ParseOptions, to_mdast};

    #[test]
    fn test_extract_text_simple() {
        let nodes = vec![
            Node::Text(Text {
                value: "Hello ".to_string(),
                position: None,
            }),
            Node::Text(Text {
                value: "world".to_string(),
                position: None,
            }),
        ];

        let result = extract_text_from_nodes(&nodes);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_extract_text_with_emphasis() {
        let nodes = vec![
            Node::Text(Text {
                value: "Hello ".to_string(),
                position: None,
            }),
            Node::Emphasis(Emphasis {
                children: vec![Node::Text(Text {
                    value: "emphasized".to_string(),
                    position: None,
                })],
                position: None,
            }),
            Node::Text(Text {
                value: " world".to_string(),
                position: None,
            }),
        ];

        let result = extract_text_from_nodes(&nodes);
        assert_eq!(result, "Hello emphasized world");
    }

    #[test]
    fn test_extract_text_with_strong() {
        let nodes = vec![
            Node::Text(Text {
                value: "Hello ".to_string(),
                position: None,
            }),
            Node::Strong(Strong {
                children: vec![Node::Text(Text {
                    value: "bold".to_string(),
                    position: None,
                })],
                position: None,
            }),
            Node::Text(Text {
                value: " world".to_string(),
                position: None,
            }),
        ];

        let result = extract_text_from_nodes(&nodes);
        assert_eq!(result, "Hello bold world");
    }

    #[test]
    fn test_extract_text_with_inline_code() {
        let nodes = vec![
            Node::Text(Text {
                value: "Use ".to_string(),
                position: None,
            }),
            Node::InlineCode(InlineCode {
                value: "printf()".to_string(),
                position: None,
            }),
            Node::Text(Text {
                value: " function".to_string(),
                position: None,
            }),
        ];

        let result = extract_text_from_nodes(&nodes);
        assert_eq!(result, "Use printf() function");
    }

    #[test]
    fn test_extract_text_with_link() {
        let nodes = vec![
            Node::Text(Text {
                value: "Visit ".to_string(),
                position: None,
            }),
            Node::Link(Link {
                children: vec![Node::Text(Text {
                    value: "this link".to_string(),
                    position: None,
                })],
                url: "https://example.com".to_string(),
                title: None,
                position: None,
            }),
            Node::Text(Text {
                value: " for more".to_string(),
                position: None,
            }),
        ];

        let result = extract_text_from_nodes(&nodes);
        assert_eq!(result, "Visit this link for more");
    }

    #[test]
    fn test_extract_text_with_image() {
        let nodes = vec![
            Node::Text(Text {
                value: "See ".to_string(),
                position: None,
            }),
            Node::Image(Image {
                alt: "Alt text".to_string(),
                url: "/image.jpg".to_string(),
                title: None,
                position: None,
            }),
            Node::Text(Text {
                value: " here".to_string(),
                position: None,
            }),
        ];

        let result = extract_text_from_nodes(&nodes);
        assert_eq!(result, "See Alt text here");
    }

    #[test]
    fn test_extract_text_complex_formatting() {
        // Parse a real markdown string with complex formatting
        let content = "This has **bold**, *italic*, `code`, and [links](https://example.com)";
        let options = ParseOptions::default();
        let root = to_mdast(content, &options).unwrap();

        if let Node::Root(root_node) = root {
            // Find the paragraph and extract its text
            for child in &root_node.children {
                if let Node::Paragraph(para) = child {
                    let text = extract_text_from_nodes(&para.children);
                    assert_eq!(text, "This has bold, italic, code, and links");
                    return;
                }
            }
        }

        panic!("Should have found paragraph in AST");
    }

    #[test]
    fn test_extract_text_nested_formatting() {
        // Test deeply nested formatting
        let nodes = vec![
            Node::Text(Text {
                value: "Start ".to_string(),
                position: None,
            }),
            Node::Strong(Strong {
                children: vec![
                    Node::Text(Text {
                        value: "bold with ".to_string(),
                        position: None,
                    }),
                    Node::Emphasis(Emphasis {
                        children: vec![Node::Text(Text {
                            value: "italic".to_string(),
                            position: None,
                        })],
                        position: None,
                    }),
                    Node::Text(Text {
                        value: " inside".to_string(),
                        position: None,
                    }),
                ],
                position: None,
            }),
            Node::Text(Text {
                value: " end".to_string(),
                position: None,
            }),
        ];

        let result = extract_text_from_nodes(&nodes);
        assert_eq!(result, "Start bold with italic inside end");
    }

    #[test]
    fn test_extract_text_empty_nodes() {
        let nodes = vec![];
        let result = extract_text_from_nodes(&nodes);
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_text_ignored_nodes() {
        // Test nodes that should be ignored (like unsupported MDX nodes)
        let nodes = vec![
            Node::Text(Text {
                value: "Before ".to_string(),
                position: None,
            }),
            Node::MdxTextExpression(markdown::mdast::MdxTextExpression {
                value: "ignored".to_string(),
                position: None,
                stops: vec![],
            }),
            Node::Text(Text {
                value: " after".to_string(),
                position: None,
            }),
        ];

        let result = extract_text_from_nodes(&nodes);
        assert_eq!(result, "Before  after");
    }
}
