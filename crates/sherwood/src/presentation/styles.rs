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

        // Apply minification and other processing
        Self::apply_minification(&mut stylesheet, self)?;
        
        // Serialize to CSS
        Self::serialize_stylesheet(&stylesheet, self, filename)
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

        // Apply minification and other processing
        Self::apply_minification(&mut stylesheet, self)?;
        
        // Serialize to CSS
        let filename = entry_point.to_string_lossy();
        let result = Self::serialize_stylesheet(&stylesheet, self, &filename)?;

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

// Common CSS processing functions
impl CssProcessor {
    /// Apply minification to a stylesheet if enabled
    fn apply_minification(stylesheet: &mut StyleSheet, processor: &Self) -> Result<()> {
        if processor.minify {
            let minify_options = MinifyOptions {
                targets: processor.targets,
                #[allow(clippy::if_same_then_else)]
                unused_symbols: if processor.remove_unused {
                    HashSet::new() // Remove all unused symbols
                } else {
                    HashSet::new() // Default empty set
                },
            };
            stylesheet
                .minify(minify_options)
                .map_err(|e| anyhow::anyhow!("Failed to minify CSS: {}", e))?;
        }
        Ok(())
    }

    /// Serialize a stylesheet to CSS string
    fn serialize_stylesheet(stylesheet: &StyleSheet, processor: &Self, filename: &str) -> Result<String> {
        let result = stylesheet
            .to_css(PrinterOptions {
                minify: processor.minify,
                source_map: None, // Will handle source maps separately
                targets: processor.targets,
                ..PrinterOptions::default()
            })
            .map_err(|e| anyhow::anyhow!("Failed to serialize CSS from {}: {}", filename, e))?;

        // TODO: Implement proper source map generation when Lightning CSS API supports it
        // For now, source maps are not generated due to API limitations
        if processor.source_maps {
            eprintln!(
                "⚠️  Source maps requested but not yet implemented in Lightning CSS integration"
            );
        }

        Ok(result.code)
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

// Entry point validation types and functions
#[derive(Debug)]
pub enum EntryPointValidationError {
    Empty,
    ContainsPathSeparators,
    MissingExtension,
    InvalidExtension,
}

impl std::fmt::Display for EntryPointValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryPointValidationError::Empty => write!(f, "cannot be empty"),
            EntryPointValidationError::ContainsPathSeparators => write!(f, "must be a simple filename, not a path"),
            EntryPointValidationError::MissingExtension => write!(f, "must have a file extension"),
            EntryPointValidationError::InvalidExtension => write!(f, "must end with .css"),
        }
    }
}

fn validate_css_entry_point(entry_point: &str) -> Result<String, EntryPointValidationError> {
    if entry_point.is_empty() {
        return Err(EntryPointValidationError::Empty);
    }
    
    if entry_point.contains('/') || entry_point.contains('\\') {
        return Err(EntryPointValidationError::ContainsPathSeparators);
    }
    
    if !entry_point.contains('.') {
        return Err(EntryPointValidationError::MissingExtension);
    }
    
    if !entry_point.ends_with(".css") {
        return Err(EntryPointValidationError::InvalidExtension);
    }
    
    Ok(entry_point.to_string())
}

fn resolve_and_validate_entry_point(css_config: Option<&CssSection>) -> String {
    if let Some(css_config) = css_config
        && let Some(entry_point) = &css_config.entry_point
    {
        match validate_css_entry_point(entry_point) {
            Ok(validated) => validated,
            Err(error) => {
                eprintln!(
                    "⚠️  Warning: Invalid CSS entry point '{}': {}. Using default 'main.css'.",
                    entry_point, error
                );
                "main.css".to_string()
            }
        }
    } else {
        "main.css".to_string()
    }
}

// Common CSS processing functions for StyleManager
impl StyleManager {
    /// Apply minification to a stylesheet if enabled
    fn apply_minification_to_stylesheet(stylesheet: &mut StyleSheet, processor: &CssProcessor) -> Result<()> {
        if processor.minify {
            let minify_options = MinifyOptions {
                targets: processor.targets,
                #[allow(clippy::if_same_then_else)]
                unused_symbols: if processor.remove_unused {
                    HashSet::new() // Remove all unused symbols
                } else {
                    HashSet::new() // Default empty set
                },
            };
            stylesheet
                .minify(minify_options)
                .map_err(|e| anyhow::anyhow!("Failed to minify CSS: {}", e))?;
        }
        Ok(())
    }

    /// Serialize a stylesheet to CSS string
    fn serialize_stylesheet_to_string(stylesheet: &StyleSheet, processor: &CssProcessor, filename: &str) -> Result<String> {
        let result = stylesheet
            .to_css(PrinterOptions {
                minify: processor.minify,
                source_map: None, // Will handle source maps separately
                targets: processor.targets,
                ..PrinterOptions::default()
            })
            .map_err(|e| anyhow::anyhow!("Failed to serialize CSS from {}: {}", filename, e))?;

        // TODO: Implement proper source map generation when Lightning CSS API supports it
        // For now, source maps are not generated due to API limitations
        if processor.source_maps {
            eprintln!(
                "⚠️  Source maps requested but not yet implemented in Lightning CSS integration"
            );
        }

        Ok(result.code)
    }
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



    fn list_available_css_files(&self) -> Result<String> {
        if !self.styles_dir.exists() {
            return Ok("(no styles directory)".to_string());
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&self.styles_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && path.extension().is_some_and(|ext| ext == "css")
                && let Some(name) = path.file_name()
            {
                files.push(name.to_string_lossy().to_string());
            }
        }

        if files.is_empty() {
            Ok("(no CSS files found)".to_string())
        } else {
            Ok(files.join(", "))
        }
    }

    fn list_embedded_css_files(&self) -> String {
        let mut files = Vec::new();
        for file in STYLES.files() {
            if let Some(file_name) = file.path().file_name()
                && file.path().extension().is_some_and(|ext| ext == "css")
            {
                files.push(file_name.to_string_lossy().to_string());
            }
        }
        if files.is_empty() {
            "(no embedded CSS files found)".to_string()
        } else {
            files.join(", ")
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
            self.process_user_css_entry_point(&css_dir, &entry_point)?;
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
            self.process_embedded_css_entry_point(&css_dir, &entry_point)?;
        }

        // Always output to main.css
        Ok(css_dir.join("main.css"))
    }

    fn process_user_css_entry_point(
        &self,
        css_dir: &Path,
        entry_point: &str,
    ) -> Result<()> {
        let entry_path = self.styles_dir.join(entry_point);

        if entry_path.exists() {
            // Bundle the specified entry point
            self.css_processor.bundle_css_files(&entry_path, css_dir)?;
            println!("Bundled CSS: {} -> dist/css/main.css", entry_point);
        } else {
            // Entry point not found - provide helpful error message
            return Err(anyhow::anyhow!(
                "CSS entry point '{}' not found in styles directory '{}'.\n\
                 Available files: {}\n\
                 Fix: either create the file or remove the 'entry_point' configuration to use defaults.",
                entry_point,
                self.styles_dir.display(),
                self.list_available_css_files()?
            ));
        }

        Ok(())
    }

    fn process_embedded_css_entry_point(&self, css_dir: &Path, entry_point: &str) -> Result<()> {
        // Check if embedded entry point exists
        if STYLES.get_file(entry_point).is_some() {
            let main_css_path = css_dir.join("main.css");

            // Use bundler by creating a temporary directory with embedded files
            let temp_dir = std::env::temp_dir().join("sherwood-css-").join(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_nanos()
                    .to_string(),
            );

            // Extract embedded CSS files to temporary directory
            self.extract_embedded_css_to_temp(&temp_dir)?;

            // Use Lightning CSS bundler for proper @import resolution
            let fs_provider = FileProvider::new();
            let mut bundler = Bundler::new(
                &fs_provider,
                None, // No source map generation yet
                ParserOptions {
                    filename: entry_point.to_string(),
                    ..ParserOptions::default()
                },
            );

            // Change to the temp directory so bundler can resolve relative imports
            let original_dir = std::env::current_dir()?;
            std::env::set_current_dir(&temp_dir)?;

            let mut stylesheet = bundler
                .bundle(Path::new(entry_point))
                .map_err(|e| anyhow::anyhow!("Failed to bundle embedded CSS: {}", e))?;

            // Restore original working directory
            std::env::set_current_dir(original_dir)?;

            // Apply minification and other processing using common functions
            StyleManager::apply_minification_to_stylesheet(&mut stylesheet, &self.css_processor)?;
            let result = StyleManager::serialize_stylesheet_to_string(&stylesheet, &self.css_processor, entry_point)?;

            fs::write(&main_css_path, &result)?;

            println!(
                "Bundled embedded CSS: {} -> {}",
                entry_point,
                main_css_path.display()
            );

            // Clean up temporary directory
            let _ = fs::remove_dir_all(&temp_dir);
        } else {
            return Err(anyhow::anyhow!(
                "Embedded CSS entry point '{}' not found.\n\
                 Available embedded files: {}\n\
                 Fix: either add the file to styles/ directory or remove 'entry_point' configuration.",
                entry_point,
                self.list_embedded_css_files()
            ));
        }

        Ok(())
    }

    fn extract_embedded_css_to_temp(&self, temp_dir: &Path) -> Result<()> {
        ensure_directory_exists(temp_dir)?;

        // Extract all embedded CSS files to temporary directory
        for file in STYLES.files() {
            let file_path = file.path();
            if let Some(file_name) = file_path.file_name()
                && let Some(extension) = Path::new(file_name).extension()
                && extension == "css"
            {
                let dest_path = temp_dir.join(file_name);
                if let Some(content) = file.contents_utf8() {
                    fs::write(&dest_path, content)?;
                }
            }
        }

        Ok(())
    }
}
