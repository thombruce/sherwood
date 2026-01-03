use crate::config::{CssSection, CssTargets};
use crate::core::utils::ensure_directory_exists;
use anyhow::Result;
use include_dir::{Dir, include_dir};
use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::targets::{Browsers, Targets};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

// Embed styles directory at compile time
static STYLES: Dir = include_dir!("$CARGO_MANIFEST_DIR/styles");

#[derive(Debug, Clone)]
pub struct CssProcessor {
    minify: bool,
    targets: Targets,
    enable_css_modules: bool,
    source_maps: bool,
    remove_unused: bool,
    nesting: bool,
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

        // Minify if enabled
        if self.minify {
            let minify_options = MinifyOptions {
                targets: self.targets,
                #[allow(clippy::if_same_then_else)]
                unused_symbols: if self.remove_unused {
                    HashSet::new() // Remove all unused symbols
                } else {
                    HashSet::new() // Default empty set
                },
            };
            stylesheet
                .minify(minify_options)
                .map_err(|e| anyhow::anyhow!("Failed to minify CSS from {}: {}", filename, e))?;
        }

        // Print to CSS
        let result = stylesheet
            .to_css(PrinterOptions {
                minify: self.minify,
                source_map: None, // Will handle source maps separately
                targets: self.targets,
                ..PrinterOptions::default()
            })
            .map_err(|e| anyhow::anyhow!("Failed to serialize CSS from {}: {}", filename, e))?;

        // TODO: Implement proper source map generation when Lightning CSS API supports it
        // For now, source maps are not generated due to API limitations
        if self.source_maps {
            eprintln!(
                "⚠️  Source maps requested but not yet implemented in Lightning CSS integration"
            );
        }

        Ok(result.code)
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

        // Minify if enabled
        if self.minify {
            let minify_options = MinifyOptions {
                targets: self.targets,
                #[allow(clippy::if_same_then_else)]
                unused_symbols: if self.remove_unused {
                    HashSet::new() // Remove all unused symbols
                } else {
                    HashSet::new() // Default empty set
                },
            };
            stylesheet
                .minify(minify_options)
                .map_err(|e| anyhow::anyhow!("Failed to minify bundled CSS: {}", e))?;
        }

        // Print to CSS
        let result = stylesheet
            .to_css(PrinterOptions {
                minify: self.minify,
                source_map: None, // Will handle source maps separately
                targets: self.targets,
                ..PrinterOptions::default()
            })
            .map_err(|e| anyhow::anyhow!("Failed to serialize bundled CSS: {}", e))?;

        let file_name = entry_point
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid CSS file name"))?;
        let output_path = output_dir.join(file_name);

        ensure_directory_exists(output_dir)?;

        // Write the bundled CSS
        fs::write(&output_path, &result.code)?;

        // TODO: Implement proper source map generation when Lightning CSS API supports it
        // For now, source maps are not generated due to API limitations
        if self.source_maps {
            eprintln!(
                "⚠️  Source maps requested but not yet implemented in Lightning CSS integration"
            );
        }

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

fn parse_css_targets(css_targets: &CssTargets) -> Targets {
    let mut browsers = Browsers::default();

    // Parse individual browser versions
    if let Some(chrome) = &css_targets.chrome
        && let Ok(version) = parse_browser_version(chrome)
    {
        browsers.chrome = Some(version);
    }

    if let Some(firefox) = &css_targets.firefox
        && let Ok(version) = parse_browser_version(firefox)
    {
        browsers.firefox = Some(version);
    }

    if let Some(safari) = &css_targets.safari
        && let Ok(version) = parse_browser_version(safari)
    {
        browsers.safari = Some(version);
    }

    if let Some(edge) = &css_targets.edge
        && let Ok(version) = parse_browser_version(edge)
    {
        browsers.edge = Some(version);
    }

    // TODO: Parse browserslist string if provided
    // For now, fall back to defaults if browserslist is provided
    if css_targets.browserslist.is_some() {
        return get_default_browser_targets();
    }

    Targets {
        browsers: Some(browsers),
        ..Targets::default()
    }
}

fn parse_browser_version(version_str: &str) -> Result<u32, std::num::ParseIntError> {
    // Parse version like "103" or "103.0" to Lightning CSS format (version << 16)
    let parts: Vec<&str> = version_str.split('.').collect();
    let major: u32 = parts[0].parse()?;

    // Lightning CSS uses version in format: (major << 16) | (minor << 8) | patch
    let minor = if parts.len() > 1 {
        parts[1].parse().unwrap_or(0)
    } else {
        0
    };
    let patch = if parts.len() > 2 {
        parts[2].parse().unwrap_or(0)
    } else {
        0
    };

    Ok((major << 16) | (minor << 8) | patch)
}

fn get_default_browser_targets() -> Targets {
    // Target modern browsers for better CSS support
    let browsers = Browsers {
        chrome: Some(103 << 16),  // Chrome 103+
        firefox: Some(115 << 16), // Firefox 115+
        safari: Some(15 << 16),   // Safari 15+
        edge: Some(127 << 16),    // Edge 127+
        ..Browsers::default()
    };

    Targets {
        browsers: Some(browsers),
        ..Targets::default()
    }
}

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

    pub fn generate_css_file(&self, output_dir: &Path) -> Result<PathBuf> {
        let css_dir = output_dir.join("css");
        ensure_directory_exists(&css_dir)?;

        // Try user's styles directory first, fallback to embedded styles
        if self.styles_dir.exists() {
            self.process_user_css_files(&css_dir)?;
        } else {
            self.process_embedded_css_files(&css_dir)?;
        }

        // The main stylesheet will be main.css
        Ok(css_dir.join("main.css"))
    }

    fn process_user_css_files(&self, css_dir: &Path) -> Result<()> {
        // Check if there's a main.css file that should be bundled
        let main_css_path = self.styles_dir.join("main.css");

        if main_css_path.exists() {
            // Process main.css with bundling to handle @import statements
            self.css_processor
                .bundle_css_files(&main_css_path, css_dir)?;
            
            // Note: Individual CSS files are not generated when main.css exists
            // because they are bundled into main.css via @import statements.
            // This prevents duplication and follows proper CSS bundling behavior.
        } else {
            // Process individual CSS files when no main.css exists
            for entry in fs::read_dir(&self.styles_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file()
                    && let Some(extension) = path.extension()
                    && extension == "css"
                {
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    let dest_path = css_dir.join(&*file_name);
                    self.css_processor.process_css_file(&path, &dest_path)?;
                }
            }
        }

        Ok(())
    }

    fn process_embedded_css_files(&self, css_dir: &Path) -> Result<()> {
        // Process main.css if it exists (it contains @import statements)
        if let Some(_main_css_file) = STYLES.get_file("main.css") {
            let main_css_path = css_dir.join("main.css");

            // Use bundler by creating a temporary directory with embedded files
            let temp_dir = std::env::temp_dir().join("sherwood-css-").join(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_nanos()
                    .to_string(),
            );

            // Extract all embedded CSS files to temporary directory
            ensure_directory_exists(&temp_dir)?;
            for file in STYLES.files() {
                let file_path = file.path();
                if let Some(file_name) = file_path.file_name()
                    && let Some(extension) = Path::new(file_name)
                        .extension()
                        .and_then(|ext| ext.to_str())
                    && extension == "css"
                {
                    let dest_path = temp_dir.join(file_name);
                    if let Some(content) = file.contents_utf8() {
                        fs::write(&dest_path, content)?;
                    }
                }
            }

            // Use Lightning CSS bundler for proper @import resolution
            let fs_provider = FileProvider::new();
            let mut bundler = Bundler::new(
                &fs_provider,
                None, // No source map generation yet
                ParserOptions {
                    filename: "main.css".to_string(),
                    ..ParserOptions::default()
                },
            );

            // Change to the temp directory so bundler can resolve relative imports
            let original_dir = std::env::current_dir()?;
            std::env::set_current_dir(&temp_dir)?;

            let mut stylesheet = bundler
                .bundle(Path::new("main.css"))
                .map_err(|e| anyhow::anyhow!("Failed to bundle embedded CSS: {}", e))?;

            // Restore original working directory
            std::env::set_current_dir(original_dir)?;

            // Apply minification and other processing
            if self.css_processor.minify {
                let minify_options = MinifyOptions {
                    targets: self.css_processor.targets,
                    #[allow(clippy::if_same_then_else)]
                    unused_symbols: if self.css_processor.remove_unused {
                        HashSet::new() // Remove all unused symbols
                    } else {
                        HashSet::new() // Default empty set
                    },
                };
                stylesheet
                    .minify(minify_options)
                    .map_err(|e| anyhow::anyhow!("Failed to minify bundled CSS: {}", e))?;
            }

            // Print to CSS
            let result = stylesheet
                .to_css(PrinterOptions {
                    minify: self.css_processor.minify,
                    source_map: None, // Will handle source maps separately
                    targets: self.css_processor.targets,
                    ..PrinterOptions::default()
                })
                .map_err(|e| anyhow::anyhow!("Failed to serialize bundled CSS: {}", e))?;

            fs::write(&main_css_path, &result.code)?;

            // TODO: Implement proper source map generation when Lightning CSS API supports it
            if self.css_processor.source_maps {
                eprintln!(
                    "⚠️  Source maps requested but not yet implemented in Lightning CSS integration"
                );
            }

            println!(
                "Bundled embedded CSS: main.css -> {}",
                main_css_path.display()
            );

            // Clean up temporary directory
            let _ = fs::remove_dir_all(&temp_dir);

            // Note: Individual CSS files are not generated when main.css exists
            // because they are bundled into main.css via @import statements.
            // This prevents duplication and follows proper CSS bundling behavior.
        } else {
            // Fallback: just copy all embedded CSS files individually
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
                        let processed_css = self
                            .css_processor
                            .process_css_string(content, &file_name_str)?;
                        self.css_processor
                            .write_processed_css(&processed_css, &dest_path)?;
                    }
                }
            }
        }

        Ok(())
    }
}
