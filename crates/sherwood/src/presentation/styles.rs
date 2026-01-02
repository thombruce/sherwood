use crate::core::utils::ensure_directory_exists;
use anyhow::Result;
use include_dir::{Dir, include_dir};
use std::fs;
use std::path::{Path, PathBuf};

// Embed styles directory at compile time
static STYLES: Dir = include_dir!("$CARGO_MANIFEST_DIR/styles");

#[derive(Debug)]
pub struct StyleManager {
    styles_dir: PathBuf,
}

impl StyleManager {
    pub fn new(styles_dir: &Path) -> Self {
        Self {
            styles_dir: styles_dir.to_path_buf(),
        }
    }

    pub fn generate_css_file(&self, output_dir: &Path) -> Result<PathBuf> {
        let css_dir = output_dir.join("css");
        ensure_directory_exists(&css_dir)?;

        // Copy all CSS files from styles directory
        self.copy_all_css_files(&css_dir)?;

        // The main stylesheet will be main.css
        Ok(css_dir.join("main.css"))
    }

    fn copy_all_css_files(&self, css_dir: &Path) -> Result<()> {
        // 1. Try user's styles directory first
        if self.styles_dir.exists() {
            self.copy_filesystem_css_files(css_dir)?;
        } else {
            // 2. Fallback to embedded styles
            self.copy_embedded_css_files(css_dir)?;
        }

        Ok(())
    }

    fn copy_filesystem_css_files(&self, css_dir: &Path) -> Result<()> {
        for entry in fs::read_dir(&self.styles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(extension) = path.extension()
                && extension == "css"
            {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let dest_path = css_dir.join(&*file_name);
                let content = fs::read_to_string(&path)?;
                fs::write(&dest_path, content)?;
                println!("Copied CSS: {} -> {}", file_name, dest_path.display());
            }
        }
        Ok(())
    }

    fn copy_embedded_css_files(&self, css_dir: &Path) -> Result<()> {
        for file in STYLES.files() {
            let file_path = file.path();
            if let Some(file_name) = file_path.file_name()
                && let Some(extension) = Path::new(file_name)
                    .extension()
                    .and_then(|ext| ext.to_str())
                && extension == "css"
            {
                let file_name_str = file_name.to_string_lossy().to_string();
                let dest_path = css_dir.join(&file_name_str);
                if let Some(content) = file.contents_utf8() {
                    fs::write(&dest_path, content)?;
                    println!(
                        "Copied embedded CSS: {} -> {}",
                        file_name_str,
                        dest_path.display()
                    );
                }
            }
        }
        Ok(())
    }
}
