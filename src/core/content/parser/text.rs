use super::{ContentParser, Parsed, ParserError};
use crate::core::content::frontmatter::FrontMatter;
use gray_matter::Pod;
use std::path::Path;

/// The built-in plain-text parser. Handles `.txt`: the first line is the
/// page title, the remaining lines are the body, HTML-escaped and wrapped in a
/// `<pre>` block. Carries no frontmatter convention — the title *is* the
/// metadata.
pub struct TextParser;

impl ContentParser for TextParser {
    fn extensions(&self) -> &[&str] {
        &["txt"]
    }

    fn parse(&self, source: &str, _path: &Path) -> Result<Parsed, ParserError> {
        let mut lines = source.lines();
        let title = lines
            .next()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .ok_or_else(|| ParserError::Message("empty file: no title line".to_string()))?
            .to_string();
        let body = lines.collect::<Vec<_>>().join("\n");
        let content_html = format!("<pre>{}</pre>", escape_html(&body));
        Ok(Parsed {
            frontmatter: FrontMatter {
                title,
                data: Pod::Null,
            },
            content_html,
            excerpt_html: None,
        })
    }
}

/// Escape the HTML metacharacters that would otherwise break out of the
/// `<pre>` block. Quotes are left as-is — they are inert in element content.
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Parsed {
        TextParser
            .parse(source, Path::new("notes.txt"))
            .expect("parse should succeed")
    }

    #[test]
    fn first_line_is_title_rest_is_body() {
        let parsed = parse("My Notes\nline one\nline two");
        assert_eq!(parsed.frontmatter.title, "My Notes");
        assert_eq!(parsed.content_html, "<pre>line one\nline two</pre>");
        assert!(parsed.excerpt_html.is_none());
    }

    #[test]
    fn body_is_html_escaped() {
        let parsed = parse("Title\n<script>alert(1) && 2</script>");
        assert!(parsed.content_html.contains("&lt;script&gt;"));
        assert!(parsed.content_html.contains("&amp;&amp;"));
        assert!(!parsed.content_html.contains("<script>"));
    }

    #[test]
    fn title_only_yields_empty_body() {
        let parsed = parse("Just a title\n");
        assert_eq!(parsed.frontmatter.title, "Just a title");
        assert_eq!(parsed.content_html, "<pre></pre>");
    }

    #[test]
    fn empty_source_is_an_error() {
        let err = TextParser.parse("", Path::new("x.txt")).unwrap_err();
        assert!(matches!(err, ParserError::Message(_)));
    }

    #[test]
    fn blank_first_line_is_an_error() {
        let err = TextParser.parse("\nbody", Path::new("x.txt")).unwrap_err();
        assert!(matches!(err, ParserError::Message(_)));
    }
}
