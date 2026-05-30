use std::path::Path;
use thiserror::Error;
use walkdir::WalkDir;
use crate::config::SiteConfig;
use crate::nav::{self, PageContext};
use crate::page::{Page, load_page};

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),
    #[error("Frontmatter parse error in {path}: {message}")]
    FrontmatterParse { path: String, message: String },
    #[error("Render error: {0}")]
    Render(String),
}

pub fn build_site<F>(config: &SiteConfig, renderer: F) -> Result<(), BuildError>
where
    F: Fn(&Page, &PageContext) -> Result<String, BuildError>,
{
    std::fs::create_dir_all(&config.output_dir)?;

    // Pass 1: collect all pages
    let mut pages: Vec<Page> = Vec::new();
    for entry in WalkDir::new(&config.content_dir) {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|s| s.to_str()) == Some("md")
        {
            pages.push(load_page(entry.path(), config)?);
        }
    }

    // Sort alphabetically by output path
    pages.sort_by(|a, b| a.output_path.cmp(&b.output_path));

    // Pass 2: render each page with navigation context
    for page in &pages {
        let ctx = nav::compute_context(page, &pages, config);
        let html = renderer(page, &ctx)?;
        write_page(&page.output_path, &html)?;
        println!("{} -> {}", page.source_path.display(), page.output_path.display());
    }

    Ok(())
}

fn write_page(output_path: &Path, html: &str) -> Result<(), BuildError> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_path, html)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup(files: &[(&str, &str)]) -> (TempDir, SiteConfig) {
        let tmp = TempDir::new().unwrap();
        let content_dir = tmp.path().join("content");
        let output_dir = tmp.path().join("_site");
        fs::create_dir_all(&content_dir).unwrap();
        for (path, content) in files {
            let full = content_dir.join(path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(full, content).unwrap();
        }
        let config = SiteConfig { content_dir, output_dir };
        (tmp, config)
    }

    #[test]
    fn build_creates_output_files() {
        let (_tmp, config) = setup(&[
            ("index.md", "---\ntitle: Home\n---\n\n# Home"),
            ("about.md", "---\ntitle: About\n---\n\nAbout page."),
        ]);
        build_site(&config, |page, _ctx| {
            Ok(format!("<html><title>{}</title></html>", page.frontmatter.title))
        })
        .unwrap();
        assert!(config.output_dir.join("index.html").exists());
        assert!(config.output_dir.join("about.html").exists());
    }

    #[test]
    fn build_renderer_receives_all_pages_in_nav() {
        let (_tmp, config) = setup(&[
            ("index.md", "---\ntitle: Home\n---\n"),
            ("about.md", "---\ntitle: About\n---\n"),
        ]);
        build_site(&config, |_page, ctx| Ok(format!("nav:{}", ctx.nav.len()))).unwrap();
        let content = fs::read_to_string(config.output_dir.join("index.html")).unwrap();
        assert!(content.contains("nav:2"));
    }

    #[test]
    fn build_output_mirrors_nested_structure() {
        let (_tmp, config) = setup(&[
            ("blog/post.md", "---\ntitle: Post\n---\n\nHello."),
        ]);
        build_site(&config, |_page, _ctx| Ok(String::new())).unwrap();
        assert!(config.output_dir.join("blog/post.html").exists());
    }

    #[test]
    fn build_empty_content_dir_succeeds() {
        let (_tmp, config) = setup(&[]);
        assert!(build_site(&config, |_page, _ctx| Ok(String::new())).is_ok());
    }

    #[test]
    fn build_missing_content_dir_returns_error() {
        let tmp = TempDir::new().unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().join("nonexistent"),
            output_dir: tmp.path().join("_site"),
        };
        assert!(build_site(&config, |_page, _ctx| Ok(String::new())).is_err());
    }
}
