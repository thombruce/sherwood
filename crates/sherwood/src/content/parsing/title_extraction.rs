use crate::content::parsing::ast_utils::extract_text_from_nodes;
use markdown::mdast::Node;
use std::path::Path;

/// Extract title from markdown AST by finding the first H1 heading
/// Strips all formatting and returns plain text
pub fn extract_title_from_ast(root: &Node) -> Option<String> {
    if let Node::Root(root_node) = root {
        for child in &root_node.children {
            if let Node::Heading(heading) = child
                && heading.depth == 1
            {
                let title_text = extract_text_from_nodes(&heading.children);
                if !title_text.trim().is_empty() {
                    return Some(title_text.trim().to_string());
                }
            }
        }
    }
    None
}

/// Extract title from file path by using the filename (without extension)
/// Used as fallback when no frontmatter or H1 title is found
pub fn extract_title_from_path(file_path: &Path) -> String {
    file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

/// Resolve title using priority: frontmatter > H1 > filename
pub fn resolve_title(
    frontmatter_title: Option<String>,
    ast_root: &Node,
    file_path: &Path,
) -> String {
    // Priority 1: Frontmatter title
    if let Some(title) = frontmatter_title {
        return title;
    }

    // Priority 2: First H1 from AST
    if let Some(title) = extract_title_from_ast(ast_root) {
        return title;
    }

    // Priority 3: Filename
    extract_title_from_path(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::{ParseOptions, to_mdast};
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn parse_markdown_ast(content: &str) -> Node {
        let options = ParseOptions {
            constructs: markdown::Constructs {
                frontmatter: true,
                ..Default::default()
            },
            ..ParseOptions::default()
        };
        to_mdast(content, &options).unwrap()
    }

    #[test]
    fn test_extract_title_from_ast_simple() {
        let content = r#"# Simple Title

This content has a simple H1 title."#;

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(title, Some("Simple Title".to_string()));
    }

    #[test]
    fn test_extract_title_from_ast_with_emphasis() {
        let content = r#"# Title with *emphasis* and **bold**

This content has a complex H1 title."#;

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(title, Some("Title with emphasis and bold".to_string()));
    }

    #[test]
    fn test_extract_title_from_ast_with_inline_code() {
        let content = r#"# Title with `code` and more text

This content has inline code in the title."#;

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(title, Some("Title with code and more text".to_string()));
    }

    #[test]
    fn test_extract_title_from_ast_with_link() {
        let content = r#"# Title with [a link](https://example.com) text

This content has a link in the title."#;

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(title, Some("Title with a link text".to_string()));
    }

    #[test]
    fn test_extract_title_from_ast_complex_formatting() {
        let content = r#"# Title with *italic*, **bold**, `code`, and [links](https://example.com)

This content has all types of inline formatting."#;

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(
            title,
            Some("Title with italic, bold, code, and links".to_string())
        );
    }

    #[test]
    fn test_extract_title_from_ast_ignores_h2_and_below() {
        let content = r#"## H2 Title
### H3 Title

This content has no H1 title."#;

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_title_from_ast_first_h1_only() {
        let content = r#"# First Title
# Second Title

This content has multiple H1 titles."#;

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(title, Some("First Title".to_string()));
    }

    #[test]
    fn test_extract_title_from_ast_empty_heading() {
        let content = r"#

This content has an empty H1 heading.";

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_title_from_ast_whitespace_only() {
        let content = "#
   
This content has a whitespace-only H1 heading.";

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_title_from_path_simple() {
        let path = PathBuf::from("my-article.md");
        let title = extract_title_from_path(&path);
        assert_eq!(title, "my-article");
    }

    #[test]
    fn test_extract_title_from_path_nested() {
        let path = PathBuf::from("blog/posts/2024/my-post.md");
        let title = extract_title_from_path(&path);
        assert_eq!(title, "my-post");
    }

    #[test]
    fn test_extract_title_from_path_no_extension() {
        let path = PathBuf::from("README");
        let title = extract_title_from_path(&path);
        assert_eq!(title, "README");
    }

    #[test]
    fn test_extract_title_from_path_unicode() {
        let path = PathBuf::from("тест-статья.md");
        let title = extract_title_from_path(&path);
        assert_eq!(title, "тест-статья");
    }

    #[test]
    fn test_extract_title_from_path_empty() {
        let path = PathBuf::from(".md");
        let title = extract_title_from_path(&path);
        assert_eq!(title, ".md"); // File stem is ".md" in this case
    }

    #[test]
    fn test_resolve_title_frontmatter_priority() {
        let content = r#"# H1 Title

Content here."#;

        let root = parse_markdown_ast(content);
        let path = PathBuf::from("test.md");

        let title = resolve_title(Some("Frontmatter Title".to_string()), &root, &path);

        assert_eq!(title, "Frontmatter Title");
    }

    #[test]
    fn test_resolve_title_h1_fallback() {
        let content = r#"# H1 Title

Content here."#;

        let root = parse_markdown_ast(content);
        let path = PathBuf::from("test.md");

        let title = resolve_title(None, &root, &path);

        assert_eq!(title, "H1 Title");
    }

    #[test]
    fn test_resolve_title_filename_fallback() {
        let content = r#"## No H1 Here

Content here."#;

        let root = parse_markdown_ast(content);
        let path = PathBuf::from("my-article.md");

        let title = resolve_title(None, &root, &path);

        assert_eq!(title, "my-article");
    }

    #[test]
    fn test_resolve_title_empty_h1_fallback_to_filename() {
        let content = r"#

Content here.";

        let root = parse_markdown_ast(content);
        let path = PathBuf::from("fallback.md");

        let title = resolve_title(None, &root, &path);

        assert_eq!(title, "fallback");
    }

    #[test]
    fn test_resolve_title_with_frontmatter_and_content() {
        let content = r#"+++
title = "Frontmatter Title"
+++

# H1 Title

Content here."#;

        let root = parse_markdown_ast(content);
        let path = PathBuf::from("test.md");

        let title = resolve_title(Some("Frontmatter Title".to_string()), &root, &path);

        assert_eq!(title, "Frontmatter Title");
    }

    #[test]
    fn test_ast_vs_string_parsing_compatibility() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;

        // Test cases that should work the same for both methods
        let test_cases = vec![
            ("simple", "# Simple Title\nContent here.", "Simple Title"),
            (
                "with-space",
                "# Title with space\nContent here.",
                "Title with space",
            ),
            (
                "with-punctuation",
                "# Title, with punctuation!\nContent here.",
                "Title, with punctuation!",
            ),
        ];

        for (filename, content, expected_title) in test_cases {
            let file_path = temp_dir.path().join(format!("{}.md", filename));
            std::fs::write(&file_path, content)?;

            let root = parse_markdown_ast(content);
            let title = resolve_title(None, &root, &file_path);

            assert_eq!(title, expected_title, "Failed for case: {}", filename);
        }

        Ok(())
    }

    #[test]
    fn test_extract_title_with_images_in_heading() {
        let content = r#"# Title with ![alt text](image.jpg) image

Content here."#;

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(title, Some("Title with alt text image".to_string()));
    }

    #[test]
    fn test_extract_title_nested_formatting() {
        let content = r#"# Title with **bold and *italic*** text

Complex nested formatting."#;

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        assert_eq!(title, Some("Title with bold and italic text".to_string()));
    }

    #[test]
    fn test_extract_title_with_delete_strikethrough() {
        let content = r#"# Title with ~~strikethrough~~ text

Content here."#;

        let root = parse_markdown_ast(content);
        let title = extract_title_from_ast(&root);
        // Note: The markdown crate doesn't process strikethrough inside headings
        // This is expected behavior - strikethrough extensions typically apply to paragraph content
        assert_eq!(title, Some("Title with ~~strikethrough~~ text".to_string()));
    }
}
