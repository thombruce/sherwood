use crate::config::CssSection;
use crate::core::utils::ensure_directory_exists;
use crate::presentation::css_processing::{apply_minification, serialize_stylesheet};
use anyhow::Result;
use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use lightningcss::targets::Targets;
use std::fs;
use std::path::{Path, PathBuf};

use super::browser_targets::{get_default_browser_targets, parse_css_targets};

#[derive(Debug, Clone)]
pub struct CssProcessor {
    pub minify: bool,
    pub targets: Targets,
    pub enable_css_modules: bool,
    pub source_maps: bool,
    pub remove_unused: bool,
    pub nesting: bool,
}

impl CssProcessor {
    pub fn new() -> Self {
        Self {
            minify: true,
            targets: get_default_browser_targets(),
            enable_css_modules: false,
            source_maps: false,
            remove_unused: false,
            nesting: true,
        }
    }

    pub fn from_config(css_config: &CssSection, is_development: bool) -> Self {
        let mut processor = Self {
            minify: css_config.minify.unwrap_or(!is_development),
            targets: css_config
                .targets
                .as_ref()
                .map(parse_css_targets)
                .unwrap_or_else(get_default_browser_targets),
            enable_css_modules: false, // TODO: Add CSS modules support later
            source_maps: css_config.source_maps.unwrap_or(is_development),
            remove_unused: css_config.remove_unused.unwrap_or(false),
            nesting: css_config.nesting.unwrap_or(true),
        };

        // Always disable minification and enable source maps in development
        if is_development {
            processor.minify = false;
            processor.source_maps = true;
        }

        processor
    }

    pub fn with_minify(mut self, minify: bool) -> Self {
        self.minify = minify;
        self
    }

    pub fn with_targets(mut self, targets: Targets) -> Self {
        self.targets = targets;
        self
    }

    pub fn with_css_modules(mut self, enable: bool) -> Self {
        self.enable_css_modules = enable;
        self
    }

    pub fn with_source_maps(mut self, enable: bool) -> Self {
        self.source_maps = enable;
        self
    }

    pub fn with_remove_unused(mut self, enable: bool) -> Self {
        self.remove_unused = enable;
        self
    }

    pub fn with_nesting(mut self, enable: bool) -> Self {
        self.nesting = enable;
        self
    }

    /// Process CSS content from a string and return the processed CSS string
    pub fn process_css_string(&self, content: &str, filename: &str) -> Result<String> {
        // Parse CSS with Lightning CSS
        let mut stylesheet = StyleSheet::parse(
            content,
            ParserOptions {
                filename: filename.to_string(),
                ..ParserOptions::default()
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to parse CSS content from {}: {}", filename, e))?;

        // Apply minification and other processing using shared functions
        apply_minification(&mut stylesheet, self)?;

        // Serialize to CSS
        serialize_stylesheet(&stylesheet, self, filename)
    }

    /// Write processed CSS content to a file
    pub fn write_processed_css(&self, content: &str, output_path: &Path) -> Result<()> {
        ensure_directory_exists(output_path.parent().unwrap_or_else(|| Path::new("")))?;
        fs::write(output_path, content)?;
        Ok(())
    }

    /// Process CSS from a file and write to output file (legacy method)
    pub fn process_css_file(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let css_content = fs::read_to_string(input_path)?;
        let filename = input_path.to_string_lossy().to_string();

        let processed_content = self.process_css_string(&css_content, &filename)?;
        self.write_processed_css(&processed_content, output_path)?;

        println!(
            "Processed CSS: {} -> {}",
            input_path.display(),
            output_path.display()
        );

        Ok(())
    }

    pub fn bundle_css_files(&self, entry_point: &Path, output_dir: &Path) -> Result<PathBuf> {
        // Use Lightning CSS bundler for proper @import resolution
        let fs_provider = FileProvider::new();
        let mut bundler = Bundler::new(
            &fs_provider,
            None,
            ParserOptions {
                filename: entry_point.to_string_lossy().to_string(),
                ..ParserOptions::default()
            },
        );

        let mut stylesheet = bundler.bundle(entry_point).map_err(|e| {
            anyhow::anyhow!("Failed to bundle CSS file {}: {}", entry_point.display(), e)
        })?;

        // Apply minification and other processing using shared functions
        apply_minification(&mut stylesheet, self)?;

        // Serialize to CSS
        let filename = entry_point.to_string_lossy();
        let result = serialize_stylesheet(&stylesheet, self, &filename)?;

        // Always output to main.css for consistent behavior
        let output_path = output_dir.join("main.css");

        ensure_directory_exists(output_dir)?;

        // Write the bundled CSS
        fs::write(&output_path, &result)?;

        println!(
            "Bundled CSS: {} -> {}",
            entry_point.display(),
            output_path.display()
        );

        Ok(output_path)
    }
}

impl Default for CssProcessor {
    fn default() -> Self {
        Self::new()
    }
}
