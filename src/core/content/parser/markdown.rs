use super::{ContentParser, Parsed, ParserError};
use crate::core::content::frontmatter::split_frontmatter;
use pulldown_cmark::{Options, Parser, html};
use std::path::Path;

/// Everything before this delimiter (if present) becomes the page's excerpt.
const EXCERPT_DELIMITER: &str = "<!-- more -->";

/// The built-in markdown parser. Handles `.md` / `.markdown`, splits YAML or
/// TOML frontmatter via [`split_frontmatter`], renders the body with
/// `pulldown-cmark`, and extracts an optional `<!-- more -->` excerpt.
pub struct MarkdownParser;

impl ContentParser for MarkdownParser {
    fn extensions(&self) -> &[&str] {
        &["md", "markdown"]
    }

    fn parse(&self, source: &str, _path: &Path) -> Result<Parsed, ParserError> {
        let (frontmatter, body) = split_frontmatter(source)?;
        let excerpt_html = body
            .split_once(EXCERPT_DELIMITER)
            .map(|(before, _)| markdown_to_html(before));
        let content_html = markdown_to_html(&body);
        Ok(Parsed {
            frontmatter,
            content_html,
            excerpt_html,
        })
    }
}

/// Render a markdown string to an HTML fragment with all `pulldown-cmark`
/// extensions enabled.
pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Parsed {
        MarkdownParser
            .parse(source, Path::new("test.md"))
            .expect("parse should succeed")
    }

    #[test]
    fn renders_heading_and_body() {
        let parsed = parse("---\ntitle: Post\n---\n\n# Hello\n\nBody.");
        assert!(parsed.content_html.contains("<h1>Hello</h1>"));
        assert!(parsed.content_html.contains("<p>Body.</p>"));
        assert_eq!(parsed.frontmatter.title, "Post");
    }

    #[test]
    fn excerpt_extracted_when_delimiter_present() {
        let parsed =
            parse("---\ntitle: Post\n---\n\nIntro line.\n\n<!-- more -->\n\nRest of post.");
        let excerpt = parsed.excerpt_html.expect("excerpt should be set");
        assert!(excerpt.contains("Intro line."));
        assert!(!excerpt.contains("Rest of post."));
    }

    #[test]
    fn no_excerpt_when_delimiter_absent() {
        let parsed = parse("---\ntitle: Post\n---\n\nJust a body.");
        assert!(parsed.excerpt_html.is_none());
    }

    #[test]
    fn markdown_bold_converts_to_strong() {
        assert!(markdown_to_html("**bold**").contains("<strong>bold</strong>"));
    }

    #[test]
    fn missing_frontmatter_is_a_parser_error() {
        let err = MarkdownParser
            .parse("# No frontmatter", Path::new("x.md"))
            .unwrap_err();
        assert!(matches!(err, ParserError::Frontmatter(_)));
    }
}
