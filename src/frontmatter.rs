use gray_matter::{Matter, Pod, engine::{Engine, YAML, TOML}};
use crate::build::BuildError;

const EXCERPT_DELIMITER: &str = "<!-- more -->";

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
    path: &str,
) -> Result<(FrontMatter, String, Option<String>), BuildError> {
    let first_line = source.lines().next().unwrap_or("").trim();

    match first_line {
        "---" => {
            let mut matter = Matter::<YAML>::new();
            matter.excerpt_delimiter = Some(EXCERPT_DELIMITER.to_owned());
            finalize(matter, source, path)
        }
        "+++" => {
            let mut matter = Matter::<TOML>::new();
            matter.delimiter = "+++".to_owned();
            matter.excerpt_delimiter = Some(EXCERPT_DELIMITER.to_owned());
            finalize(matter, source, path)
        }
        _ => Err(BuildError::FrontmatterParse {
            path: path.to_string(),
            message: "No frontmatter found (expected --- for YAML or +++ for TOML)".to_string(),
        }),
    }
}

fn finalize<E: Engine>(
    matter: Matter<E>,
    source: &str,
    path: &str,
) -> Result<(FrontMatter, String, Option<String>), BuildError> {
    let result = matter
        .parse::<Pod>(source)
        .map_err(|e| BuildError::FrontmatterParse {
            path: path.to_string(),
            message: e.to_string(),
        })?;
    let data = result.data.ok_or_else(|| BuildError::FrontmatterParse {
        path: path.to_string(),
        message: "No frontmatter data found".to_string(),
    })?;
    let map = match &data {
        Pod::Hash(map) => map,
        _ => {
            return Err(BuildError::FrontmatterParse {
                path: path.to_string(),
                message: "Frontmatter must be a map of fields".to_string(),
            });
        }
    };
    let title = map
        .get("title")
        .and_then(|p| p.as_string().ok())
        .ok_or_else(|| BuildError::FrontmatterParse {
            path: path.to_string(),
            message: "missing required field `title`".to_string(),
        })?;
    Ok((FrontMatter { title, data }, result.content, result.excerpt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_frontmatter_parses_title_and_body() {
        let source = "---\ntitle: Hello World\n---\n\nBody content.";
        let (fm, body, _) = parse_frontmatter(source, "test.md").unwrap();
        assert_eq!(fm.title, "Hello World");
        assert!(body.contains("Body content."));
    }

    #[test]
    fn toml_frontmatter_parses_title_and_body() {
        let source = "+++\ntitle = \"Hello TOML\"\n+++\n\nBody content.";
        let (fm, body, _) = parse_frontmatter(source, "test.md").unwrap();
        assert_eq!(fm.title, "Hello TOML");
        assert!(body.contains("Body content."));
    }

    #[test]
    fn no_frontmatter_returns_error() {
        let source = "# Just a heading\n\nNo frontmatter here.";
        let result = parse_frontmatter(source, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn empty_file_returns_error() {
        let result = parse_frontmatter("", "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn yaml_frontmatter_missing_title_returns_error() {
        let source = "---\nfoo: bar\n---\n\nContent.";
        let result = parse_frontmatter(source, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn toml_frontmatter_missing_title_returns_error() {
        let source = "+++\nfoo = \"bar\"\n+++\n\nContent.";
        let result = parse_frontmatter(source, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn malformed_yaml_returns_error() {
        let source = "---\ntitle: [unclosed\n---\n\nBody.";
        let result = parse_frontmatter(source, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn malformed_toml_returns_error() {
        let source = "+++\ntitle =\n+++\n\nBody.";
        let result = parse_frontmatter(source, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn yaml_extra_fields_accessible_via_get() {
        let source = "---\ntitle: Post\ndate: 2026-05-31\nauthor: Thom\n---\n\nBody.";
        let (fm, _, _) = parse_frontmatter(source, "test.md").unwrap();
        assert_eq!(fm.get_string("author").as_deref(), Some("Thom"));
        assert_eq!(fm.get_string("date").as_deref(), Some("2026-05-31"));
    }

    #[test]
    fn toml_extra_fields_accessible_via_get() {
        let source = "+++\ntitle = \"Post\"\nauthor = \"Thom\"\ndraft = true\n+++\n\nBody.";
        let (fm, _, _) = parse_frontmatter(source, "test.md").unwrap();
        assert_eq!(fm.get_string("author").as_deref(), Some("Thom"));
        match fm.get("draft") {
            Some(Pod::Boolean(b)) => assert!(b),
            other => panic!("expected Pod::Boolean, got {:?}", other),
        }
    }

    #[test]
    fn get_returns_none_for_missing_key() {
        let source = "---\ntitle: Page\n---\n\nBody.";
        let (fm, _, _) = parse_frontmatter(source, "test.md").unwrap();
        assert!(fm.get("nonexistent").is_none());
        assert!(fm.get_string("nonexistent").is_none());
    }

    #[test]
    fn toml_datetime_coerced_to_string() {
        let source = "+++\ntitle = \"Post\"\ndate = 2026-05-31\n+++\n\nBody.";
        let (fm, _, _) = parse_frontmatter(source, "test.md").unwrap();
        assert_eq!(fm.get_string("date").as_deref(), Some("2026-05-31"));
    }

    #[test]
    fn yaml_array_field_accessible() {
        let source = "---\ntitle: Post\ntags:\n  - rust\n  - ssg\n---\n\nBody.";
        let (fm, _, _) = parse_frontmatter(source, "test.md").unwrap();
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
        let (_, _, excerpt) = parse_frontmatter(source, "test.md").unwrap();
        let ex = excerpt.expect("excerpt should be set");
        assert!(ex.contains("Intro line."));
        assert!(!ex.contains("Rest of post."));
    }

    #[test]
    fn no_excerpt_when_delimiter_absent() {
        let source = "---\ntitle: Post\n---\n\nJust a body.";
        let (_, _, excerpt) = parse_frontmatter(source, "test.md").unwrap();
        assert!(excerpt.is_none());
    }
}
