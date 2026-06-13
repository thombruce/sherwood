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
///     .with_output_dir("out")
///     .with_base_path("/sherwood");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SiteConfig {
    pub content_dir: PathBuf,
    pub output_dir: PathBuf,
    /// URL prefix for a site served from a non-root path, e.g. `/sherwood` for
    /// `https://host/sherwood/`. Normalized to either `""` (served at the
    /// domain root — the default) or a leading-slash, no-trailing-slash string
    /// like `"/sherwood"`. Affects generated URLs only, never output paths.
    pub base_path: String,
}

impl SiteConfig {
    /// A config with the default directories (`content/` → `_site/`) and no
    /// base path. Equivalent to [`SiteConfig::default`]; chain `with_*` methods
    /// to override individual fields.
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

    /// Set the URL base path for serving the site from a subdirectory. The
    /// value is normalized: surrounding slashes are trimmed and a single
    /// leading slash is added, so `"sherwood"`, `"/sherwood/"`, and
    /// `"/sherwood"` all become `"/sherwood"`. Empty or `"/"` clears it
    /// (root-served).
    pub fn with_base_path(mut self, path: impl AsRef<str>) -> Self {
        self.base_path = normalize_base_path(path.as_ref());
        self
    }
}

/// Normalize a raw base path into `""` (root) or `"/segment[/segment...]"`.
fn normalize_base_path(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("/{trimmed}")
    }
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("_site"),
            base_path: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_path_normalizes() {
        let cases = [
            ("", ""),
            ("/", ""),
            ("sherwood", "/sherwood"),
            ("/sherwood", "/sherwood"),
            ("/sherwood/", "/sherwood"),
            ("sherwood/", "/sherwood"),
            ("  /docs/  ", "/docs"),
            ("a/b", "/a/b"),
        ];
        for (input, want) in cases {
            assert_eq!(
                SiteConfig::new().with_base_path(input).base_path,
                want,
                "input {input:?}"
            );
        }
    }

    #[test]
    fn default_base_path_is_empty() {
        assert_eq!(SiteConfig::default().base_path, "");
    }
}
