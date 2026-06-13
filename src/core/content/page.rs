use crate::core::config::SiteConfig;
use crate::core::content::frontmatter::FrontMatter;
use crate::core::content::parser::{ParserError, ParserRegistry};
use crate::core::nav::href_for;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Failure modes when loading a single content source into a [`Page`]. Each
/// variant carries the offending source path so build errors point at the
/// exact file.
#[derive(Debug, Error)]
pub enum PageError {
    #[error("reading {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("parsing {}: {source}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: ParserError,
    },
}

#[derive(Debug, Clone)]
pub struct Page {
    pub frontmatter: FrontMatter,
    pub content_html: String,
    /// Pre-rendered excerpt HTML, when the source contains the `<!-- more -->`
    /// delimiter. Everything before the delimiter is extracted, converted to
    /// HTML, and stored here. `None` if the delimiter is absent.
    pub excerpt_html: Option<String>,
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    /// Absolute URL of the page, e.g. `/blog/first-post/`. Cross-platform
    /// (uses `/` separators on every OS).
    pub url: String,
    /// `true` when the source file is named `index.md` (a section landing
    /// page, including the root index). Regular pages are wrapped in a
    /// `<stem>/index.html` directory for pretty URLs and have this flag set
    /// to `false`.
    pub is_section_index: bool,
}

/// Load one content file into a [`Page`], dispatching to the parser registered
/// for its extension. Returns `Ok(None)` when no parser claims the extension,
/// so the build can skip non-content files (images, CSS, …) living in the
/// content tree.
pub fn load_page(
    source_path: &Path,
    config: &SiteConfig,
    registry: &ParserRegistry,
) -> Result<Option<Page>, PageError> {
    let ext = source_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let Some(parser) = registry.get(ext) else {
        return Ok(None);
    };

    let source = std::fs::read_to_string(source_path).map_err(|e| PageError::Read {
        path: source_path.to_owned(),
        source: e,
    })?;
    let parsed = parser
        .parse(&source, source_path)
        .map_err(|e| PageError::Parse {
            path: source_path.to_owned(),
            source: e,
        })?;

    let is_section_index = source_path.file_stem().and_then(|s| s.to_str()) == Some("index");
    let output_path = output_path_for(source_path, config);
    let url = href_for(&output_path, config);
    Ok(Some(Page {
        frontmatter: parsed.frontmatter,
        content_html: parsed.content_html,
        excerpt_html: parsed.excerpt_html,
        source_path: source_path.to_owned(),
        output_path,
        url,
        is_section_index,
    }))
}

pub(crate) fn output_path_for(source: &Path, config: &SiteConfig) -> PathBuf {
    let relative = source.strip_prefix(&config.content_dir).unwrap_or(source);
    let stem = relative.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let parent = relative.parent().unwrap_or(Path::new(""));
    if stem == "index" {
        config.output_dir.join(parent).join("index.html")
    } else {
        config.output_dir.join(parent).join(stem).join("index.html")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::SiteConfig;
    use std::fs;
    use tempfile::TempDir;

    fn default_config() -> SiteConfig {
        SiteConfig {
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("_site"),
            ..SiteConfig::default()
        }
    }

    #[test]
    fn output_path_flat_file_wraps_in_dir() {
        let config = default_config();
        let path = output_path_for(Path::new("content/about.md"), &config);
        assert_eq!(path, PathBuf::from("_site/about/index.html"));
    }

    #[test]
    fn output_path_nested_file_wraps_in_dir() {
        let config = default_config();
        let path = output_path_for(Path::new("content/blog/post.md"), &config);
        assert_eq!(path, PathBuf::from("_site/blog/post/index.html"));
    }

    #[test]
    fn output_path_root_index_stays_flat() {
        let config = default_config();
        let path = output_path_for(Path::new("content/index.md"), &config);
        assert_eq!(path, PathBuf::from("_site/index.html"));
    }

    #[test]
    fn output_path_section_index_stays_flat() {
        let config = default_config();
        let path = output_path_for(Path::new("content/blog/index.md"), &config);
        assert_eq!(path, PathBuf::from("_site/blog/index.html"));
    }

    #[test]
    fn output_path_outside_content_dir_falls_back() {
        let config = default_config();
        let path = output_path_for(Path::new("other/page.md"), &config);
        assert_eq!(path, PathBuf::from("_site/other/page/index.html"));
    }

    #[test]
    fn load_page_reads_yaml_frontmatter() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("about.md");
        fs::write(&file, "---\ntitle: About\n---\n\n# About").unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().to_owned(),
            output_dir: tmp.path().join("_site"),
            ..SiteConfig::default()
        };
        let page = load_page(&file, &config, &ParserRegistry::default())
            .unwrap()
            .unwrap();
        assert_eq!(page.frontmatter.title, "About");
        assert!(page.content_html.contains("<h1>About</h1>"));
    }

    #[test]
    fn load_page_reads_toml_frontmatter() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("post.md");
        fs::write(&file, "+++\ntitle = \"My Post\"\n+++\n\nHello.").unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().to_owned(),
            output_dir: tmp.path().join("_site"),
            ..SiteConfig::default()
        };
        let page = load_page(&file, &config, &ParserRegistry::default())
            .unwrap()
            .unwrap();
        assert_eq!(page.frontmatter.title, "My Post");
    }

    #[test]
    fn load_page_missing_title_returns_error() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("bad.md");
        fs::write(&file, "---\nfoo: bar\n---\n\nContent.").unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().to_owned(),
            output_dir: tmp.path().join("_site"),
            ..SiteConfig::default()
        };
        assert!(load_page(&file, &config, &ParserRegistry::default()).is_err());
    }

    #[test]
    fn load_page_sets_pretty_url() {
        let tmp = TempDir::new().unwrap();
        let blog = tmp.path().join("blog");
        fs::create_dir_all(&blog).unwrap();
        let file = blog.join("post.md");
        fs::write(&file, "---\ntitle: Post\n---\n\nBody.").unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().to_owned(),
            output_dir: tmp.path().join("_site"),
            ..SiteConfig::default()
        };
        let page = load_page(&file, &config, &ParserRegistry::default())
            .unwrap()
            .unwrap();
        assert_eq!(page.url, "/blog/post/");
        assert!(!page.is_section_index);
    }

    #[test]
    fn load_page_section_index_url_and_flag() {
        let tmp = TempDir::new().unwrap();
        let blog = tmp.path().join("blog");
        fs::create_dir_all(&blog).unwrap();
        let file = blog.join("index.md");
        fs::write(&file, "---\ntitle: Blog\n---\n\nBody.").unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().to_owned(),
            output_dir: tmp.path().join("_site"),
            ..SiteConfig::default()
        };
        let page = load_page(&file, &config, &ParserRegistry::default())
            .unwrap()
            .unwrap();
        assert_eq!(page.url, "/blog/");
        assert!(page.is_section_index);
    }

    #[test]
    fn load_page_root_index_url() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("index.md");
        fs::write(&file, "---\ntitle: Home\n---\n\nBody.").unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().to_owned(),
            output_dir: tmp.path().join("_site"),
            ..SiteConfig::default()
        };
        let page = load_page(&file, &config, &ParserRegistry::default())
            .unwrap()
            .unwrap();
        assert_eq!(page.url, "/");
        assert!(page.is_section_index);
    }

    #[test]
    fn load_page_extracts_excerpt_when_delimiter_present() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("post.md");
        fs::write(
            &file,
            "---\ntitle: Post\n---\n\nIntro paragraph.\n\n<!-- more -->\n\nRest of body.",
        )
        .unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().to_owned(),
            output_dir: tmp.path().join("_site"),
            ..SiteConfig::default()
        };
        let page = load_page(&file, &config, &ParserRegistry::default())
            .unwrap()
            .unwrap();
        let excerpt = page.excerpt_html.expect("excerpt should be set");
        assert!(excerpt.contains("Intro paragraph."));
        assert!(!excerpt.contains("Rest of body."));
    }

    #[test]
    fn load_page_no_excerpt_when_delimiter_absent() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("post.md");
        fs::write(&file, "---\ntitle: Post\n---\n\nJust a body, no delimiter.").unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().to_owned(),
            output_dir: tmp.path().join("_site"),
            ..SiteConfig::default()
        };
        let page = load_page(&file, &config, &ParserRegistry::default())
            .unwrap()
            .unwrap();
        assert!(page.excerpt_html.is_none());
    }
}
