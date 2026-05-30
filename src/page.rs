use std::path::{Path, PathBuf};
use pulldown_cmark::{Parser, Options, html};
use crate::config::SiteConfig;
use crate::frontmatter::{FrontMatter, parse_frontmatter};
use crate::build::BuildError;

#[derive(Debug, Clone)]
pub struct Page {
    pub frontmatter: FrontMatter,
    pub content_html: String,
    pub source_path: PathBuf,
    pub output_path: PathBuf,
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
    let (frontmatter, body) = parse_frontmatter(&source, &path_str)?;
    let content_html = markdown_to_html(&body);
    let output_path = output_path_for(source_path, config);
    Ok(Page { frontmatter, content_html, source_path: source_path.to_owned(), output_path })
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
}
