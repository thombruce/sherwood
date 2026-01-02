use super::utils::ensure_directory_exists;
use anyhow::Result;
use include_dir::{Dir, include_dir};
use std::fs;
use std::path::{Path, PathBuf};

// Embed themes directory at compile time
static THEMES: Dir = include_dir!("$CARGO_MANIFEST_DIR/themes");

// Constants
const DEFAULT_CSS_FILE: &str = "default.css";

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub path: Option<PathBuf>, // None for embedded themes
    pub is_embedded: bool,
}

#[derive(Debug)]
pub struct ThemeManager {
    themes_dir: PathBuf,
}

impl ThemeManager {
    pub fn new(themes_dir: &Path) -> Self {
        Self {
            themes_dir: themes_dir.to_path_buf(),
        }
    }

    pub fn load_theme(&self, theme_name: &str) -> Result<Theme> {
        // 1. Try user's themes directory first, but only if it contains CSS files
        let theme_path = self.themes_dir.join(theme_name);
        if theme_path.exists() && theme_path.join(DEFAULT_CSS_FILE).exists() {
            return Ok(Theme {
                name: theme_name.to_string(),
                path: Some(theme_path),
                is_embedded: false,
            });
        }

        // 2. Try embedded themes
        if let Some(_embedded_theme) = THEMES.get_dir(theme_name) {
            return Ok(Theme {
                name: theme_name.to_string(),
                path: None,
                is_embedded: true,
            });
        }

        // 3. Final fallback to embedded default theme
        if theme_name != "default"
            && let Some(_embedded_default) = THEMES.get_dir("default")
        {
            println!(
                "Theme '{}' not found, falling back to embedded default theme",
                theme_name
            );
            return Ok(Theme {
                name: "default".to_string(),
                path: None,
                is_embedded: true,
            });
        }

        // 4. If even default theme is missing, fail
        Err(anyhow::anyhow!(
            "Theme '{}' not found and no embedded default theme available",
            theme_name
        ))
    }

    pub fn get_available_themes(&self) -> Result<Vec<String>> {
        let mut themes = Vec::new();

        // Add filesystem themes
        if self.themes_dir.exists() {
            for entry in fs::read_dir(&self.themes_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir()
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                {
                    themes.push(name.to_string());
                }
            }
        }

        // Add embedded themes
        for entry in THEMES.dirs() {
            if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                themes.push(name.to_string());
            }
        }

        // Remove duplicates and sort (filesystem themes keep precedence)
        themes.sort();
        themes.dedup();
        Ok(themes)
    }

    pub fn generate_css_file(&self, theme: &Theme, output_dir: &Path) -> Result<PathBuf> {
        let css_dir = output_dir.join("css");
        ensure_directory_exists(&css_dir)?;

        // Copy all CSS files from theme directory except the main theme file
        self.copy_all_css_files(theme, &css_dir)?;

        // Copy the main theme CSS file with the correct name
        let theme_css_path = css_dir.join(format!("{}.css", theme.name));

        if let Some(css_content) = self.get_theme_css_content(theme, DEFAULT_CSS_FILE)? {
            fs::write(&theme_css_path, css_content)?;
        } else {
            println!("Warning: No default.css found for theme '{}'", theme.name);
        }

        Ok(css_dir.join(format!("{}.css", theme.name)))
    }

    fn copy_all_css_files(&self, theme: &Theme, css_dir: &Path) -> Result<()> {
        let css_files = self.get_theme_css_files(theme)?;

        for (file_name, content) in css_files {
            let dest_path = css_dir.join(&file_name);
            fs::write(&dest_path, content)?;
            println!("Copied CSS: {} -> {}", file_name, dest_path.display());
        }

        Ok(())
    }

    fn get_theme_css_files(&self, theme: &Theme) -> Result<Vec<(String, String)>> {
        let mut css_files = Vec::new();

        if theme.is_embedded {
            self.get_embedded_css_files(theme, &mut css_files)?;
        } else if let Some(theme_path) = &theme.path {
            self.get_filesystem_css_files(theme_path, &mut css_files)?;
        }

        Ok(css_files)
    }

    fn get_embedded_css_files(
        &self,
        theme: &Theme,
        css_files: &mut Vec<(String, String)>,
    ) -> Result<()> {
        if let Some(embedded_theme) = THEMES.get_dir(&theme.name) {
            for file in embedded_theme.files() {
                let file_path = file.path();
                if let Some(file_name) = file_path.file_name()
                    && let Some(extension) = Path::new(file_name)
                        .extension()
                        .and_then(|ext| ext.to_str())
                    && extension == "css"
                    && file_name != DEFAULT_CSS_FILE
                {
                    let file_name_str = file_name.to_string_lossy().to_string();
                    if let Some(content) = file.contents_utf8() {
                        css_files.push((file_name_str, content.to_string()));
                    }
                }
            }
        }
        Ok(())
    }

    fn get_filesystem_css_files(
        &self,
        theme_path: &Path,
        css_files: &mut Vec<(String, String)>,
    ) -> Result<()> {
        for entry in fs::read_dir(theme_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(extension) = path.extension()
                && extension == "css"
                && let Some(file_name) = path.file_name()
                && file_name != DEFAULT_CSS_FILE
            {
                let file_name_str = file_name.to_string_lossy().to_string();
                let content = fs::read_to_string(&path)?;
                css_files.push((file_name_str, content));
            }
        }
        Ok(())
    }

    /// Get CSS file content from filesystem or embedded theme
    fn get_theme_css_content(&self, theme: &Theme, css_file: &str) -> Result<Option<String>> {
        if theme.is_embedded {
            // Try embedded theme first
            if let Some(embedded_theme) = THEMES.get_dir(&theme.name) {
                let css_path_in_theme = format!("{}/{}", theme.name, css_file);
                if let Some(css_file_entry) = embedded_theme.get_file(&css_path_in_theme) {
                    return Ok(Some(
                        css_file_entry
                            .contents_utf8()
                            .ok_or_else(|| anyhow::anyhow!("CSS file contains invalid UTF-8"))?
                            .to_string(),
                    ));
                }
            }
            Ok(None)
        } else if let Some(theme_path) = &theme.path {
            // Try filesystem
            let css_path = theme_path.join(css_file);
            if css_path.exists() {
                return Ok(Some(fs::read_to_string(css_path)?));
            }
            Ok(None)
        } else {
            Ok(None)
        }
    }

    pub fn get_default_theme(&self) -> String {
        "default".to_string()
    }

    pub fn resolve_theme(
        &self,
        frontmatter_theme: Option<String>,
        site_theme: Option<String>,
    ) -> String {
        let theme_name = frontmatter_theme
            .or(site_theme)
            .unwrap_or_else(|| self.get_default_theme());

        // Validate theme name for security
        self.validate_theme_name(&theme_name);
        theme_name
    }

    fn validate_theme_name(&self, theme_name: &str) {
        // Check for path traversal attempts
        if theme_name.contains("..") || theme_name.contains('/') || theme_name.contains('\\') {
            eprintln!(
                "Warning: Theme name '{}' contains invalid characters, using default theme",
                theme_name
            );
            return;
        }

        // Check for empty or whitespace-only names
        if theme_name.trim().is_empty() {
            eprintln!("Warning: Theme name is empty, using default theme");
        }
    }

    pub fn get_default_variant(&self, _theme: &Theme) -> String {
        "default".to_string()
    }
}
