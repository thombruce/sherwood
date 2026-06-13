use gray_matter::{
    Matter, Pod,
    engine::{Engine, TOML, YAML},
};
use thiserror::Error;

const EXCERPT_DELIMITER: &str = "<!-- more -->";

/// Failure modes when parsing a Markdown source's frontmatter block. Carries
/// no file path — that context is the caller's to supply (see
/// [`crate::page::PageError`]).
#[derive(Debug, Error)]
pub enum FrontmatterError {
    /// The source did not begin with a `---` (YAML) or `+++` (TOML) delimiter.
    #[error("No frontmatter found (expected --- for YAML or +++ for TOML)")]
    MissingDelimiters,
    /// Frontmatter was present but could not be parsed into a titled map. The
    /// message embeds a line-numbered snippet of the offending block.
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug, Clone)]
pub struct FrontMatter {
    pub title: String,
    pub data: Pod,
}

impl FrontMatter {
    /// Look up an arbitrary frontmatter field by key. Returns `None` if the
    /// frontmatter is not a map or the key is absent.
    pub fn get(&self, key: &str) -> Option<&Pod> {
        match &self.data {
            Pod::Hash(map) => map.get(key),
            _ => None,
        }
    }

    /// Convenience: look up a field and coerce it to a `String`. Returns
    /// `None` if absent or non-stringy.
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|p| p.as_string().ok())
    }
}

/// Parse the frontmatter, body, and (optional) excerpt out of a Markdown
/// source string. The excerpt is the Markdown text before a `<!-- more -->`
/// delimiter; `None` if the delimiter is absent.
pub fn parse_frontmatter(
    source: &str,
) -> Result<(FrontMatter, String, Option<String>), FrontmatterError> {
    let first_line = source.lines().next().unwrap_or("").trim();

    match first_line {
        "---" => {
            let mut matter = Matter::<YAML>::new();
            matter.excerpt_delimiter = Some(EXCERPT_DELIMITER.to_owned());
            finalize(matter, source)
        }
        "+++" => {
            let mut matter = Matter::<TOML>::new();
            matter.delimiter = "+++".to_owned();
            matter.excerpt_delimiter = Some(EXCERPT_DELIMITER.to_owned());
            finalize(matter, source)
        }
        _ => Err(FrontmatterError::MissingDelimiters),
    }
}

fn finalize<E: Engine>(
    matter: Matter<E>,
    source: &str,
) -> Result<(FrontMatter, String, Option<String>), FrontmatterError> {
    let frontmatter_text = extract_frontmatter_text(source);
    let make_err = |message: String| {
        FrontmatterError::Invalid(format_parse_error(&message, &frontmatter_text))
    };
    let result = matter
        .parse::<Pod>(source)
        .map_err(|e| make_err(e.to_string()))?;
    let data = result
        .data
        .ok_or_else(|| make_err("no frontmatter data found".to_string()))?;
    let map = match &data {
        Pod::Hash(map) => map,
        _ => return Err(make_err("frontmatter must be a map of fields".to_string())),
    };
    let title = map
        .get("title")
        .and_then(|p| p.as_string().ok())
        .ok_or_else(|| make_err("missing required field `title`".to_string()))?;
    Ok((FrontMatter { title, data }, result.content, result.excerpt))
}

/// Slice out just the frontmatter block (delimiter to delimiter) so error
/// messages can show the user exactly what they wrote.
fn extract_frontmatter_text(source: &str) -> String {
    let mut lines = source.lines();
    let first = match lines.next() {
        Some(l) => l.trim(),
        None => return String::new(),
    };
    if first != "---" && first != "+++" {
        return String::new();
    }
    let mut collected = vec![first];
    for line in lines {
        collected.push(line);
        if line.trim() == first {
            return collected.join("\n");
        }
    }
    String::new()
}

fn format_parse_error(message: &str, frontmatter: &str) -> String {
    if frontmatter.is_empty() {
        return message.to_string();
    }
    let indented = frontmatter
        .lines()
        .enumerate()
        .map(|(i, l)| format!("  {:>3} | {}", i + 1, l))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{message}\n\n{indented}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_frontmatter_parses_title_and_body() {
        let source = "---\ntitle: Hello World\n---\n\nBody content.";
        let (fm, body, _) = parse_frontmatter(source).unwrap();
        assert_eq!(fm.title, "Hello World");
        assert!(body.contains("Body content."));
    }

    #[test]
    fn toml_frontmatter_parses_title_and_body() {
        let source = "+++\ntitle = \"Hello TOML\"\n+++\n\nBody content.";
        let (fm, body, _) = parse_frontmatter(source).unwrap();
        assert_eq!(fm.title, "Hello TOML");
        assert!(body.contains("Body content."));
    }

    #[test]
    fn no_frontmatter_returns_error() {
        let source = "# Just a heading\n\nNo frontmatter here.";
        let result = parse_frontmatter(source);
        assert!(result.is_err());
    }

    #[test]
    fn empty_file_returns_error() {
        let result = parse_frontmatter("");
        assert!(result.is_err());
    }

    #[test]
    fn yaml_frontmatter_missing_title_returns_error() {
        let source = "---\nfoo: bar\n---\n\nContent.";
        let result = parse_frontmatter(source);
        assert!(result.is_err());
    }

    #[test]
    fn toml_frontmatter_missing_title_returns_error() {
        let source = "+++\nfoo = \"bar\"\n+++\n\nContent.";
        let result = parse_frontmatter(source);
        assert!(result.is_err());
    }

    #[test]
    fn malformed_yaml_returns_error() {
        let source = "---\ntitle: [unclosed\n---\n\nBody.";
        let result = parse_frontmatter(source);
        assert!(result.is_err());
    }

    #[test]
    fn malformed_toml_returns_error() {
        let source = "+++\ntitle =\n+++\n\nBody.";
        let result = parse_frontmatter(source);
        assert!(result.is_err());
    }

    #[test]
    fn yaml_extra_fields_accessible_via_get() {
        let source = "---\ntitle: Post\ndate: 2026-05-31\nauthor: Thom\n---\n\nBody.";
        let (fm, _, _) = parse_frontmatter(source).unwrap();
        assert_eq!(fm.get_string("author").as_deref(), Some("Thom"));
        assert_eq!(fm.get_string("date").as_deref(), Some("2026-05-31"));
    }

    #[test]
    fn toml_extra_fields_accessible_via_get() {
        let source = "+++\ntitle = \"Post\"\nauthor = \"Thom\"\ndraft = true\n+++\n\nBody.";
        let (fm, _, _) = parse_frontmatter(source).unwrap();
        assert_eq!(fm.get_string("author").as_deref(), Some("Thom"));
        match fm.get("draft") {
            Some(Pod::Boolean(b)) => assert!(b),
            other => panic!("expected Pod::Boolean, got {:?}", other),
        }
    }

    #[test]
    fn get_returns_none_for_missing_key() {
        let source = "---\ntitle: Page\n---\n\nBody.";
        let (fm, _, _) = parse_frontmatter(source).unwrap();
        assert!(fm.get("nonexistent").is_none());
        assert!(fm.get_string("nonexistent").is_none());
    }

    #[test]
    fn toml_datetime_coerced_to_string() {
        let source = "+++\ntitle = \"Post\"\ndate = 2026-05-31\n+++\n\nBody.";
        let (fm, _, _) = parse_frontmatter(source).unwrap();
        assert_eq!(fm.get_string("date").as_deref(), Some("2026-05-31"));
    }

    #[test]
    fn yaml_array_field_accessible() {
        let source = "---\ntitle: Post\ntags:\n  - rust\n  - ssg\n---\n\nBody.";
        let (fm, _, _) = parse_frontmatter(source).unwrap();
        match fm.get("tags") {
            Some(Pod::Array(items)) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].as_string().ok().as_deref(), Some("rust"));
                assert_eq!(items[1].as_string().ok().as_deref(), Some("ssg"));
            }
            other => panic!("expected Pod::Array, got {:?}", other),
        }
    }

    #[test]
    fn excerpt_extracted_when_delimiter_present() {
        let source = "---\ntitle: Post\n---\n\nIntro line.\n\n<!-- more -->\n\nRest of post.";
        let (_, _, excerpt) = parse_frontmatter(source).unwrap();
        let ex = excerpt.expect("excerpt should be set");
        assert!(ex.contains("Intro line."));
        assert!(!ex.contains("Rest of post."));
    }

    #[test]
    fn no_excerpt_when_delimiter_absent() {
        let source = "---\ntitle: Post\n---\n\nJust a body.";
        let (_, _, excerpt) = parse_frontmatter(source).unwrap();
        assert!(excerpt.is_none());
    }

    #[test]
    fn parse_error_includes_frontmatter_snippet() {
        let source = "---\ntitle: [unclosed\n---\n\nBody.";
        let err = parse_frontmatter(source).unwrap_err();
        assert!(matches!(err, FrontmatterError::Invalid(_)));
        let msg = err.to_string();
        // Frontmatter lines shown with line numbers (path is added by the
        // page layer, not here).
        assert!(msg.contains("  1 | ---"));
        assert!(msg.contains("  2 | title: [unclosed"));
        assert!(msg.contains("  3 | ---"));
    }

    #[test]
    fn missing_title_error_includes_what_was_provided() {
        let source = "---\nfoo: bar\nbaz: qux\n---\n\nBody.";
        let err = parse_frontmatter(source).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing required field `title`"));
        assert!(msg.contains("foo: bar"));
        assert!(msg.contains("baz: qux"));
    }

    #[test]
    fn no_frontmatter_error_skips_snippet() {
        let source = "no delimiters here";
        let err = parse_frontmatter(source).unwrap_err();
        assert!(matches!(err, FrontmatterError::MissingDelimiters));
        // Without delimiters there's nothing to extract; just the bare message.
        assert!(!err.to_string().contains(" | "));
    }
}
