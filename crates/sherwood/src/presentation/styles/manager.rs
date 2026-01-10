use crate::config::CssSection;
use crate::core::utils::ensure_directory_exists;
use anyhow::Result;
use std::path::{Path, PathBuf};

use super::embedded_files::{process_embedded_css_entry_point, process_user_css_entry_point};
use super::processor::CssProcessor;
use super::validation::resolve_and_validate_entry_point;

#[derive(Debug)]
pub struct StyleManager {
    styles_dir: PathBuf,
    css_processor: CssProcessor,
    #[allow(dead_code)]
    is_development: bool,
}

impl StyleManager {
    pub fn new(styles_dir: &Path) -> Self {
        Self::new_with_config(styles_dir, None, false)
    }

    pub fn new_development(styles_dir: &Path) -> Self {
        Self::new_with_config(styles_dir, None, true)
    }

    pub fn new_with_config(
        styles_dir: &Path,
        css_config: Option<&CssSection>,
        is_development: bool,
    ) -> Self {
        let css_processor = if let Some(config) = css_config {
            CssProcessor::from_config(config, is_development)
        } else {
            let processor = CssProcessor::new();
            if is_development {
                processor.with_minify(false).with_source_maps(true)
            } else {
                processor
            }
        };

        Self {
            styles_dir: styles_dir.to_path_buf(),
            css_processor,
            is_development,
        }
    }

    pub fn with_processor(
        styles_dir: &Path,
        css_processor: CssProcessor,
        is_development: bool,
    ) -> Self {
        Self {
            styles_dir: styles_dir.to_path_buf(),
            css_processor,
            is_development,
        }
    }

    pub fn generate_css_file(
        &self,
        output_dir: &Path,
        css_config: Option<&CssSection>,
    ) -> Result<PathBuf> {
        let css_dir = output_dir.join("css");
        ensure_directory_exists(&css_dir)?;

        let entry_point = resolve_and_validate_entry_point(css_config);

        // Check if user explicitly configured an entry_point
        let has_explicit_entry_point = css_config
            .and_then(|config| config.entry_point.as_ref())
            .is_some();

        // Try user's styles directory first, fallback to embedded styles
        if self.styles_dir.exists() {
            process_user_css_entry_point(
                &self.css_processor,
                &self.styles_dir,
                &css_dir,
                &entry_point,
            )?;
        } else if has_explicit_entry_point {
            // User configured entry_point but has no styles directory - error
            return Err(anyhow::anyhow!(
                "CSS entry point '{}' is configured but no styles directory exists at '{}'.\n\
                 Fix: either create the styles directory with the entry point file, or remove the 'entry_point' configuration to use embedded styles.",
                entry_point,
                self.styles_dir.display()
            ));
        } else {
            // No entry_point configured and no styles directory - use embedded styles
            process_embedded_css_entry_point(&self.css_processor, &css_dir, &entry_point)?;
        }

        // Always output to main.css
        Ok(css_dir.join("main.css"))
    }
}
