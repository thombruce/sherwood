//! Pluggable content parsers.
//!
//! A [`ContentParser`] turns the raw source of one content file into a
//! [`Parsed`] payload (frontmatter + rendered HTML). Parsers are keyed by file
//! extension in a [`ParserRegistry`]; the build pipeline looks up the parser
//! for each file's extension and skips files with no registered parser.
//!
//! Markdown ships built in ([`MarkdownParser`]). Downstream crates implement
//! [`ContentParser`] for other formats and `register` them — reusing
//! [`crate::split_frontmatter`] for frontmatter handling if their format uses
//! the same `---` / `+++` convention.

mod markdown;

pub use markdown::{MarkdownParser, markdown_to_html};

use crate::frontmatter::{FrontMatter, FrontmatterError};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

/// The product of parsing one content file: its frontmatter plus rendered
/// HTML. Path-derived fields (output path, URL, section-index flag) are the
/// build pipeline's concern, not the parser's — a parser never sees or sets
/// them.
#[derive(Debug, Clone)]
pub struct Parsed {
    pub frontmatter: FrontMatter,
    pub content_html: String,
    /// Optional pre-rendered excerpt HTML, when the format supports one (e.g.
    /// markdown's `<!-- more -->` delimiter). `None` otherwise.
    pub excerpt_html: Option<String>,
}

/// Turns the raw source of a single content file into a [`Parsed`] payload.
///
/// Implementors must be `Send + Sync`: the dev server shares the registry
/// across threads when rebuilding on file changes.
pub trait ContentParser: Send + Sync {
    /// File extensions this parser claims, lowercase and without the leading
    /// dot — e.g. `["md", "markdown"]`.
    fn extensions(&self) -> &[&str];

    /// Parse `source` (the full file contents) into a [`Parsed`] payload.
    /// `path` is provided for diagnostics only.
    fn parse(&self, source: &str, path: &Path) -> Result<Parsed, ParserError>;
}

/// Errors a [`ContentParser`] may return. Open enough that third-party parsers
/// can surface their own failures via [`ParserError::Message`] or
/// [`ParserError::Other`].
#[derive(Debug, Error)]
pub enum ParserError {
    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

/// Maps file extensions to the parser that handles them.
///
/// [`ParserRegistry::default`] (and [`with_markdown`](Self::with_markdown))
/// registers the built-in [`MarkdownParser`]. Start from [`empty`](Self::empty)
/// for a registry with no formats at all.
#[derive(Clone)]
pub struct ParserRegistry {
    by_ext: HashMap<String, Arc<dyn ContentParser>>,
}

impl Default for ParserRegistry {
    /// Registers the built-in markdown parser. Use [`ParserRegistry::empty`]
    /// for a registry with no formats.
    fn default() -> Self {
        Self::with_markdown()
    }
}

impl ParserRegistry {
    /// A registry with no parsers registered.
    pub fn empty() -> Self {
        Self {
            by_ext: HashMap::new(),
        }
    }

    /// A registry with the built-in markdown parser registered.
    pub fn with_markdown() -> Self {
        let mut registry = Self::empty();
        registry.register(Arc::new(MarkdownParser));
        registry
    }

    /// Register a parser for every extension it claims. A later registration
    /// for an already-claimed extension wins.
    pub fn register(&mut self, parser: Arc<dyn ContentParser>) -> &mut Self {
        for ext in parser.extensions() {
            self.by_ext.insert(ext.to_string(), parser.clone());
        }
        self
    }

    /// The parser registered for `ext` (no leading dot), if any.
    pub fn get(&self, ext: &str) -> Option<&Arc<dyn ContentParser>> {
        self.by_ext.get(ext)
    }
}

impl std::fmt::Debug for ParserRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut exts: Vec<&str> = self.by_ext.keys().map(String::as_str).collect();
        exts.sort_unstable();
        f.debug_struct("ParserRegistry")
            .field("extensions", &exts)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeParser;
    impl ContentParser for FakeParser {
        fn extensions(&self) -> &[&str] {
            &["txt", "text"]
        }
        fn parse(&self, source: &str, _path: &Path) -> Result<Parsed, ParserError> {
            Ok(Parsed {
                frontmatter: FrontMatter {
                    title: "Fake".to_string(),
                    data: gray_matter::Pod::Null,
                },
                content_html: source.to_string(),
                excerpt_html: None,
            })
        }
    }

    #[test]
    fn default_registry_handles_markdown_extensions() {
        let registry = ParserRegistry::default();
        assert!(registry.get("md").is_some());
        assert!(registry.get("markdown").is_some());
        assert!(registry.get("txt").is_none());
    }

    #[test]
    fn empty_registry_has_no_parsers() {
        let registry = ParserRegistry::empty();
        assert!(registry.get("md").is_none());
    }

    #[test]
    fn register_claims_every_extension() {
        let mut registry = ParserRegistry::empty();
        registry.register(Arc::new(FakeParser));
        assert!(registry.get("txt").is_some());
        assert!(registry.get("text").is_some());
    }

    #[test]
    fn later_registration_wins_for_shared_extension() {
        let mut registry = ParserRegistry::with_markdown();
        // FakeParser does not claim "md", so markdown stays; sanity check the
        // override path with a parser that re-claims "md".
        struct MdShadow;
        impl ContentParser for MdShadow {
            fn extensions(&self) -> &[&str] {
                &["md"]
            }
            fn parse(&self, _s: &str, _p: &Path) -> Result<Parsed, ParserError> {
                Err(ParserError::Message("shadow".to_string()))
            }
        }
        registry.register(Arc::new(MdShadow));
        let err = registry
            .get("md")
            .unwrap()
            .parse("", Path::new("x.md"))
            .unwrap_err();
        assert!(matches!(err, ParserError::Message(m) if m == "shadow"));
    }
}
