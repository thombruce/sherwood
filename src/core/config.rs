use std::path::PathBuf;

/// Build configuration: where content is read from and where the site is
/// written.
///
/// Marked `#[non_exhaustive]` so new fields can be added without breaking
/// downstream crates. Library users outside this crate cannot use struct
/// literal syntax — construct via [`SiteConfig::new`] or [`SiteConfig::default`]
/// and the `with_*` builder methods instead:
///
/// ```
/// use sherwood::SiteConfig;
/// let config = SiteConfig::new()
///     .with_content_dir("src")
///     .with_output_dir("out");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SiteConfig {
    pub content_dir: PathBuf,
    pub output_dir: PathBuf,
}

impl SiteConfig {
    /// A config with the default directories (`content/` → `_site/`).
    /// Equivalent to [`SiteConfig::default`]; chain `with_*` methods to
    /// override individual fields.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the content source directory.
    pub fn with_content_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.content_dir = dir.into();
        self
    }

    /// Set the output directory.
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("_site"),
        }
    }
}
