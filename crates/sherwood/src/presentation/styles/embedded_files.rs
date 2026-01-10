use super::processor::CssProcessor;
use crate::core::utils::ensure_directory_exists;
use crate::presentation::css_processing::{apply_minification, serialize_stylesheet};
use anyhow::Result;
use include_dir::{Dir, include_dir};
use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::ParserOptions;
use std::fs;
use std::path::Path;

// Embed styles directory at compile time
pub static STYLES: Dir = include_dir!("$CARGO_MANIFEST_DIR/styles");

pub fn process_user_css_entry_point(
    css_processor: &CssProcessor,
    styles_dir: &Path,
    css_dir: &Path,
    entry_point: &str,
) -> Result<()> {
    let entry_path = styles_dir.join(entry_point);

    if entry_path.exists() {
        // Bundle the specified entry point
        css_processor.bundle_css_files(&entry_path, css_dir)?;
        println!("Bundled CSS: {} -> dist/css/main.css", entry_point);
    } else {
        // Entry point not found - provide helpful error message
        return Err(anyhow::anyhow!(
            "CSS entry point '{}' not found in styles directory '{}'.\n\
             Available files: {}\n\
             Fix: either create the file or remove the 'entry_point' configuration to use defaults.",
            entry_point,
            styles_dir.display(),
            list_available_css_files(styles_dir)?
        ));
    }

    Ok(())
}

pub fn process_embedded_css_entry_point(
    css_processor: &CssProcessor,
    css_dir: &Path,
    entry_point: &str,
) -> Result<()> {
    // Check if embedded entry point exists
    if STYLES.get_file(entry_point).is_some() {
        let main_css_path = css_dir.join("main.css");

        // Use secure temporary directory with automatic cleanup
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path();

        // Extract embedded CSS files to temporary directory
        extract_embedded_css_to_temp(temp_path)?;

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
        std::env::set_current_dir(temp_path)?;

        let mut stylesheet = bundler
            .bundle(Path::new(entry_point))
            .map_err(|e| anyhow::anyhow!("Failed to bundle embedded CSS: {}", e))?;

        // Restore original working directory
        std::env::set_current_dir(original_dir)?;

        // Apply minification and other processing using shared functions
        apply_minification(&mut stylesheet, css_processor)?;
        let result = serialize_stylesheet(&stylesheet, css_processor, entry_point)?;

        fs::write(&main_css_path, &result)?;

        println!(
            "Bundled embedded CSS: {} -> {}",
            entry_point,
            main_css_path.display()
        );

        // temp_dir automatically cleaned up when it goes out of scope
    } else {
        return Err(anyhow::anyhow!(
            "Embedded CSS entry point '{}' not found.\n\
             Available embedded files: {}\n\
             Fix: either add the file to styles/ directory or remove 'entry_point' configuration.",
            entry_point,
            list_embedded_css_files()
        ));
    }

    Ok(())
}

pub fn extract_embedded_css_to_temp(temp_dir: &Path) -> Result<()> {
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

fn list_available_css_files(styles_dir: &Path) -> Result<String> {
    if !styles_dir.exists() {
        return Ok("(no styles directory)".to_string());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(styles_dir)? {
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

fn list_embedded_css_files() -> String {
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
