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
