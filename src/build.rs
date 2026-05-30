use std::path::Path;
use thiserror::Error;
use walkdir::WalkDir;
use crate::config::SiteConfig;
use crate::page::load_page;

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
    F: Fn(&crate::page::Page) -> Result<String, BuildError>,
{
    std::fs::create_dir_all(&config.output_dir)?;

    for entry in WalkDir::new(&config.content_dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let page = load_page(path, config)?;
        let html = renderer(&page)?;
        write_page(&page.output_path, &html)?;

        println!("{} -> {}", path.display(), page.output_path.display());
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
