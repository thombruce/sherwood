use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub path: PathBuf,
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
        let theme_path = self.themes_dir.join(theme_name);

        if !theme_path.exists() {
            return Err(anyhow::anyhow!(
                "Theme '{}' not found in {}",
                theme_name,
                self.themes_dir.display()
            ));
        }

        Ok(Theme {
            name: theme_name.to_string(),
            path: theme_path,
        })
    }

    pub fn get_available_themes(&self) -> Result<Vec<String>> {
        let mut themes = Vec::new();

        if !self.themes_dir.exists() {
            return Ok(themes);
        }

        for entry in fs::read_dir(&self.themes_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                themes.push(name.to_string());
            }
        }

        themes.sort();
        Ok(themes)
    }

    pub fn generate_css_file(&self, theme: &Theme, output_dir: &Path) -> Result<PathBuf> {
        let css_dir = output_dir.join("css");
        fs::create_dir_all(&css_dir)?;

        // Copy all CSS files from theme directory except the main theme file
        self.copy_all_css_files(theme, &css_dir)?;

        // Copy the main theme CSS file with the correct name
        let theme_css_path = css_dir.join(format!("{}.css", theme.name));
        let source_css_path = theme.path.join("default.css");
        if source_css_path.exists() {
            fs::copy(&source_css_path, &theme_css_path)?;
            println!(
                "Generated main CSS: {} -> {}",
                source_css_path.display(),
                theme_css_path.display()
            );
        }

        Ok(css_dir.join(format!("{}.css", theme.name)))
    }

    fn copy_all_css_files(&self, theme: &Theme, css_dir: &Path) -> Result<()> {
        // Copy all CSS files from theme directory except the main theme file
        for entry in fs::read_dir(&theme.path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(extension) = path.extension()
                && extension == "css"
                && let Some(file_name) = path.file_name()
                && file_name != "default.css"
            // Skip main theme file
            {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let dest_path = css_dir.join(&*file_name);
                fs::copy(&path, &dest_path)?;
                println!("Copied CSS: {} -> {}", path.display(), dest_path.display());
            }
        }

        Ok(())
    }

    pub fn get_default_theme(&self) -> String {
        "default".to_string()
    }

    pub fn get_default_variant(&self, _theme: &Theme) -> String {
        "default".to_string()
    }
}
