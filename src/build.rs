use crate::config::SiteConfig;
use crate::nav::{self, PageContext, is_root_index};
use crate::page::{Page, PageError, load_page};
use std::path::Path;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),
    #[error(transparent)]
    Page(#[from] PageError),
    #[error("Render error: {0}")]
    Render(String),
}

pub fn build_site<F, P>(
    config: &SiteConfig,
    mut renderer: F,
    mut progress: P,
) -> Result<(), BuildError>
where
    F: FnMut(&Page, &PageContext) -> Result<String, BuildError>,
    P: FnMut(&Page),
{
    std::fs::create_dir_all(&config.output_dir)?;

    let mut pages: Vec<Page> = Vec::new();
    for entry in WalkDir::new(&config.content_dir) {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|s| s.to_str()) == Some("md")
        {
            pages.push(load_page(entry.path(), config)?);
        }
    }

    // Root index first, then remaining pages by output path. This keeps the
    // homepage at the front of the nav rather than buried after alphabetical
    // siblings like "about.html".
    pages.sort_by(|a, b| {
        let ka = (!is_root_index(a, config), a.output_path.clone());
        let kb = (!is_root_index(b, config), b.output_path.clone());
        ka.cmp(&kb)
    });

    for page in &pages {
        let ctx = nav::compute_context(page, &pages, config);
        let html = renderer(page, &ctx)?;
        write_page(&page.output_path, &html)?;
        progress(page);
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
        let config = SiteConfig {
            content_dir,
            output_dir,
        };
        (tmp, config)
    }

    #[test]
    fn build_creates_output_files() {
        let (_tmp, config) = setup(&[
            ("index.md", "---\ntitle: Home\n---\n\n# Home"),
            ("about.md", "---\ntitle: About\n---\n\nAbout page."),
        ]);
        build_site(
            &config,
            |page, _ctx| {
                Ok(format!(
                    "<html><title>{}</title></html>",
                    page.frontmatter.title
                ))
            },
            |_| {},
        )
        .unwrap();
        assert!(config.output_dir.join("index.html").exists());
        assert!(config.output_dir.join("about/index.html").exists());
    }

    #[test]
    fn build_renderer_receives_all_pages_in_nav() {
        let (_tmp, config) = setup(&[
            ("index.md", "---\ntitle: Home\n---\n"),
            ("about.md", "---\ntitle: About\n---\n"),
        ]);
        build_site(
            &config,
            |_page, ctx| Ok(format!("nav:{}", ctx.nav.len())),
            |_| {},
        )
        .unwrap();
        let content = fs::read_to_string(config.output_dir.join("index.html")).unwrap();
        assert!(content.contains("nav:2"));
    }

    #[test]
    fn build_output_mirrors_nested_structure() {
        let (_tmp, config) = setup(&[("blog/post.md", "---\ntitle: Post\n---\n\nHello.")]);
        build_site(&config, |_page, _ctx| Ok(String::new()), |_| {}).unwrap();
        assert!(config.output_dir.join("blog/post/index.html").exists());
    }

    #[test]
    fn build_empty_content_dir_succeeds() {
        let (_tmp, config) = setup(&[]);
        assert!(build_site(&config, |_page, _ctx| Ok(String::new()), |_| {}).is_ok());
    }

    #[test]
    fn build_missing_content_dir_returns_error() {
        let tmp = TempDir::new().unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().join("nonexistent"),
            output_dir: tmp.path().join("_site"),
        };
        assert!(build_site(&config, |_page, _ctx| Ok(String::new()), |_| {}).is_err());
    }

    #[test]
    fn build_renderer_error_propagates() {
        let (_tmp, config) = setup(&[("index.md", "---\ntitle: Home\n---\n")]);
        let result = build_site(
            &config,
            |_p, _ctx| Err(BuildError::Render("boom".to_string())),
            |_| {},
        );
        assert!(matches!(result, Err(BuildError::Render(msg)) if msg == "boom"));
    }

    #[test]
    fn build_progress_called_per_page() {
        let (_tmp, config) = setup(&[
            ("index.md", "---\ntitle: Home\n---\n"),
            ("about.md", "---\ntitle: About\n---\n"),
        ]);
        let mut count = 0;
        build_site(&config, |_p, _ctx| Ok(String::new()), |_p| count += 1).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn build_root_index_sorts_first() {
        let (_tmp, config) = setup(&[
            ("about.md", "---\ntitle: About\n---\n"),
            ("blog/post.md", "---\ntitle: Post\n---\n"),
            ("index.md", "---\ntitle: Home\n---\n"),
        ]);
        let mut titles = Vec::new();
        build_site(
            &config,
            |_p, ctx| {
                if titles.is_empty() {
                    titles = ctx.nav.iter().map(|n| n.title.clone()).collect();
                }
                Ok(String::new())
            },
            |_| {},
        )
        .unwrap();
        assert_eq!(titles[0], "Home");
    }
}
