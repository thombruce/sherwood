use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SiteConfig {
    pub content_dir: PathBuf,
    pub output_dir: PathBuf,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("_site"),
        }
    }
}
