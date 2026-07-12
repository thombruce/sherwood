use crate::core::config::SiteConfig;
use crate::core::content::page::{Page, PageError, load_page};
use crate::core::content::parser::ParserRegistry;
use crate::core::nav::{self, PageContext, is_root_index};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    #[error("{} and {} both write {}", first.display(), second.display(), output.display())]
    DuplicateOutput {
        first: PathBuf,
        second: PathBuf,
        output: PathBuf,
    },
}

pub fn build_site<F, P>(
    config: &SiteConfig,
    registry: &ParserRegistry,
    mut renderer: F,
    mut progress: P,
) -> Result<(), BuildError>
where
    F: FnMut(&Page, &PageContext) -> Result<String, BuildError>,
    P: FnMut(&Page),
{
    std::fs::create_dir_all(&config.output_dir)?;

    let mut pages: Vec<Page> = Vec::new();
    // output path -> source path, so two sources mapping to the same output
    // file (e.g. content/about.md and content/about/index.md) fail loudly
    // instead of one silently overwriting the other.
    let mut claimed: HashMap<PathBuf, PathBuf> = HashMap::new();
    for entry in WalkDir::new(&config.content_dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        match load_page(entry.path(), config, registry)? {
            Some(page) => {
                claim_output(&mut claimed, &page.output_path, &page.source_path)?;
                pages.push(page);
            }
            // No parser claims the extension: a static asset (image, CSS, …)
            // living in the content tree. Copy it verbatim to the mirrored
            // output path.
            None => {
                let relative = entry
                    .path()
                    .strip_prefix(&config.content_dir)
                    .unwrap_or(entry.path());
                let dest = config.output_dir.join(relative);
                claim_output(&mut claimed, &dest, entry.path())?;
                copy_asset(entry.path(), &dest)?;
            }
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

fn copy_asset(source: &Path, dest: &Path) -> Result<(), BuildError> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(source, dest)?;
    Ok(())
}

fn claim_output(
    claimed: &mut HashMap<PathBuf, PathBuf>,
    output: &Path,
    source: &Path,
) -> Result<(), BuildError> {
    if let Some(first) = claimed.insert(output.to_owned(), source.to_owned()) {
        return Err(BuildError::DuplicateOutput {
            first,
            second: source.to_owned(),
            output: output.to_owned(),
        });
    }
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
            ..SiteConfig::default()
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
            &ParserRegistry::default(),
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
            &ParserRegistry::default(),
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
        build_site(
            &config,
            &ParserRegistry::default(),
            |_page, _ctx| Ok(String::new()),
            |_| {},
        )
        .unwrap();
        assert!(config.output_dir.join("blog/post/index.html").exists());
    }

    #[test]
    fn build_empty_content_dir_succeeds() {
        let (_tmp, config) = setup(&[]);
        assert!(
            build_site(
                &config,
                &ParserRegistry::default(),
                |_page, _ctx| Ok(String::new()),
                |_| {}
            )
            .is_ok()
        );
    }

    #[test]
    fn build_missing_content_dir_returns_error() {
        let tmp = TempDir::new().unwrap();
        let config = SiteConfig {
            content_dir: tmp.path().join("nonexistent"),
            output_dir: tmp.path().join("_site"),
            ..SiteConfig::default()
        };
        assert!(
            build_site(
                &config,
                &ParserRegistry::default(),
                |_page, _ctx| Ok(String::new()),
                |_| {}
            )
            .is_err()
        );
    }

    #[test]
    fn build_renderer_error_propagates() {
        let (_tmp, config) = setup(&[("index.md", "---\ntitle: Home\n---\n")]);
        let result = build_site(
            &config,
            &ParserRegistry::default(),
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
        build_site(
            &config,
            &ParserRegistry::default(),
            |_p, _ctx| Ok(String::new()),
            |_p| count += 1,
        )
        .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn build_copies_unparsed_files_verbatim() {
        let (_tmp, config) = setup(&[
            ("index.md", "---\ntitle: Home\n---\n"),
            ("logo.png", "png bytes"),
            ("blog/photo.jpg", "jpg bytes"),
        ]);
        build_site(
            &config,
            &ParserRegistry::default(),
            |_p, _ctx| Ok(String::new()),
            |_| {},
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(config.output_dir.join("logo.png")).unwrap(),
            "png bytes"
        );
        assert_eq!(
            fs::read_to_string(config.output_dir.join("blog/photo.jpg")).unwrap(),
            "jpg bytes"
        );
    }

    #[test]
    fn build_duplicate_page_outputs_error() {
        // about.md and about/index.md both map to _site/about/index.html.
        let (_tmp, config) = setup(&[
            ("about.md", "---\ntitle: About\n---\n"),
            ("about/index.md", "---\ntitle: Also About\n---\n"),
        ]);
        let err = build_site(
            &config,
            &ParserRegistry::default(),
            |_p, _ctx| Ok(String::new()),
            |_| {},
        )
        .unwrap_err();
        assert!(matches!(err, BuildError::DuplicateOutput { .. }), "{err}");
        let msg = err.to_string();
        assert!(msg.contains("about.md"), "{msg}");
        assert!(msg.contains("index.html"), "{msg}");
    }

    #[test]
    fn build_asset_colliding_with_page_output_errors() {
        // A static about/index.html would be overwritten by the page rendered
        // from about.md.
        let (_tmp, config) = setup(&[
            ("about.md", "---\ntitle: About\n---\n"),
            ("about/index.html", "<h1>static</h1>"),
        ]);
        let err = build_site(
            &config,
            &ParserRegistry::default(),
            |_p, _ctx| Ok(String::new()),
            |_| {},
        )
        .unwrap_err();
        assert!(matches!(err, BuildError::DuplicateOutput { .. }), "{err}");
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
            &ParserRegistry::default(),
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
