use gray_matter::{Matter, engine::{Engine, YAML, TOML}};
use serde::Deserialize;
use crate::build::BuildError;

#[derive(Debug, Clone, Deserialize)]
pub struct FrontMatter {
    pub title: String,
}

pub fn parse_frontmatter(source: &str, path: &str) -> Result<(FrontMatter, String), BuildError> {
    let first_line = source.lines().next().unwrap_or("").trim();

    match first_line {
        "---" => finalize(Matter::<YAML>::new(), source, path),
        "+++" => {
            let mut matter = Matter::<TOML>::new();
            matter.delimiter = "+++".to_owned();
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
) -> Result<(FrontMatter, String), BuildError> {
    let result = matter
        .parse::<FrontMatter>(source)
        .map_err(|e| BuildError::FrontmatterParse {
            path: path.to_string(),
            message: e.to_string(),
        })?;
    let fm = result.data.ok_or_else(|| BuildError::FrontmatterParse {
        path: path.to_string(),
        message: "No frontmatter data found".to_string(),
    })?;
    Ok((fm, result.content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_frontmatter_parses_title_and_body() {
        let source = "---\ntitle: Hello World\n---\n\nBody content.";
        let (fm, body) = parse_frontmatter(source, "test.md").unwrap();
        assert_eq!(fm.title, "Hello World");
        assert!(body.contains("Body content."));
    }

    #[test]
    fn toml_frontmatter_parses_title_and_body() {
        let source = "+++\ntitle = \"Hello TOML\"\n+++\n\nBody content.";
        let (fm, body) = parse_frontmatter(source, "test.md").unwrap();
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
}
