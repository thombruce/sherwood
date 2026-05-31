use std::path::{Path, PathBuf};
use pulldown_cmark::{Parser, Options, html};
use crate::config::SiteConfig;
use crate::frontmatter::{FrontMatter, parse_frontmatter};
use crate::build::BuildError;
use crate::nav::href_for;

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
    /// Absolute URL of the page, e.g. `/blog/first-post.html`. Cross-platform
    /// (uses `/` separators on every OS).
    pub url: String,
}

pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub fn load_page(source_path: &Path, config: &SiteConfig) -> Result<Page, BuildError> {
    let source = std::fs::read_to_string(source_path)?;
    let path_str = source_path.to_string_lossy();
    let (frontmatter, body, excerpt_md) = parse_frontmatter(&source, &path_str)?;
    let content_html = markdown_to_html(&body);
    let excerpt_html = excerpt_md.map(|md| markdown_to_html(&md));
    let output_path = output_path_for(source_path, config);
    let url = href_for(&output_path, config);
    Ok(Page {
        frontmatter,
        content_html,
        excerpt_html,
        source_path: source_path.to_owned(),
        output_path,
        url,
    })
}

pub(crate) fn output_path_for(source: &Path, config: &SiteConfig) -> PathBuf {
    let relative = source
        .strip_prefix(&config.content_dir)
        .unwrap_or(source);
    config.output_dir.join(relative.with_extension("html"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SiteConfig;
    use std::fs;
    use tempfile::TempDir;

    fn default_config() -> SiteConfig {
        SiteConfig {
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("_site"),
        }
    }

    #[test]
    fn markdown_heading_converts_to_h1() {
        let html = markdown_to_html("# Hello");
        assert!(html.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn markdown_paragraph_converts_to_p() {
        let html = markdown_to_html("Simple paragraph.");
        assert!(html.contains("<p>Simple paragraph.</p>"));
    }

    #[test]
    fn markdown_bold_converts_to_strong() {
        let html = markdown_to_html("**bold**");
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn output_path_flat_file() {
        let config = default_config();
        let path = output_path_for(Path::new("content/about.md"), &config);
        assert_eq!(path, PathBuf::from("_site/about.html"));
    }

    #[test]
    fn output_path_nested_file() {
        let config = default_config();
        let path = output_path_for(Path::new("content/blog/post.md"), &config);
        assert_eq!(path, PathBuf::from("_site/blog/post.html"));
    }

    #[test]
    fn output_path_index_file() {
        let config = default_config();
        let path = output_path_for(Path::new("content/index.md"), &config);
        assert_eq!(path, PathBuf::from("_site/index.html"));
    }

    #[test]
    fn output_path_outside_content_dir_falls_back() {
        let config = default_config();
        let path = output_path_for(Path::new("other/page.md"), &config);
        assert_eq!(path, PathBuf::from("_site/other/page.html"));
    }

    #[test]
    fn load_page_reads_yaml_frontmatter() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("about.md");
        fs::write(&file, "---\ntitle: About\n---\n\n# About").unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().to_owned(),
            output_dir: tmp.path().join("_site"),
        };
        let page = load_page(&file, &config).unwrap();
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
        };
        let page = load_page(&file, &config).unwrap();
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
        };
        assert!(load_page(&file, &config).is_err());
    }

    #[test]
    fn load_page_sets_url_from_output_path() {
        let tmp = TempDir::new().unwrap();
        let blog = tmp.path().join("blog");
        fs::create_dir_all(&blog).unwrap();
        let file = blog.join("post.md");
        fs::write(&file, "---\ntitle: Post\n---\n\nBody.").unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().to_owned(),
            output_dir: tmp.path().join("_site"),
        };
        let page = load_page(&file, &config).unwrap();
        assert_eq!(page.url, "/blog/post.html");
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
        };
        let page = load_page(&file, &config).unwrap();
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
        };
        let page = load_page(&file, &config).unwrap();
        assert!(page.excerpt_html.is_none());
    }
}
