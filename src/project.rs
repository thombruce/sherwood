use crate::config::SiteConfig;
use crate::templates::copy_embedded_templates;
use crate::utils::{ensure_directory_exists, ensure_parent_exists};
use include_dir::{Dir, include_dir};
use std::fs;
use std::path::Path;
use toml;

// Embed content and themes directories at compile time
static CONTENT_TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/content");
static THEMES: Dir = include_dir!("$CARGO_MANIFEST_DIR/themes");

// Constants for template names
const DEFAULT_PAGE_TEMPLATE: &str = "default.stpl";

// Default sherwood.toml template function
fn get_default_sherwood_toml() -> String {
    format!(
        r#"[site]
theme = "default"
# Available themes: default, kanagawa

[templates]
page_template = "{}"
"#,
        DEFAULT_PAGE_TEMPLATE
    )
}

pub fn create_new_project(
    path: &Path,
    theme: &str,
    no_theme: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate inputs
    validate_inputs(path, theme, no_theme)?;

    // Create content directory
    let content_dir = path.join("content");
    ensure_directory_exists(&content_dir)?;

    // Copy index.md from content templates
    copy_template_file(
        &CONTENT_TEMPLATES,
        "index.md",
        &content_dir.join("index.md"),
    )?;

    // Copy and process sherwood.toml template
    copy_config_template(path, theme, no_theme)?;

    // Copy theme files if requested
    if !no_theme {
        copy_theme_files(path, theme)?;
    }

    // Copy template files
    copy_embedded_templates(path)?;

    // Print success message
    print_success_message(path, theme, no_theme);

    Ok(())
}

fn copy_template_file(
    templates_dir: &Dir,
    template_path: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(file) = templates_dir.get_file(template_path) {
        fs::write(
            output_path,
            file.contents_utf8()
                .ok_or_else(|| format!("Template file {} contains invalid UTF-8", template_path))?,
        )?;
    } else {
        return Err(format!("Template file not found: {}", template_path).into());
    }
    Ok(())
}

fn copy_config_template(
    path: &Path,
    theme: &str,
    no_theme: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = path.join("sherwood.toml");

    // Parse default TOML structure
    let mut config: SiteConfig = toml::from_str(&get_default_sherwood_toml())
        .map_err(|e| format!("Failed to parse config template: {}", e))?;

    if no_theme {
        // Remove theme when --no-theme is used
        config.site.theme = None;
    } else {
        // Set the selected theme
        config.site.theme = Some(theme.to_string());
    }

    // Serialize back to TOML
    let processed_content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(config_path, processed_content)?;

    Ok(())
}

fn copy_theme_files(path: &Path, theme: &str) -> Result<(), Box<dyn std::error::Error>> {
    let themes_dir = path.join("themes");
    ensure_directory_exists(&themes_dir)?;

    if let Some(theme_dir) = THEMES.get_dir(theme) {
        let target_theme_dir = themes_dir.join(theme);
        ensure_directory_exists(&target_theme_dir)?;

        // Copy all files from theme directory
        for entry in theme_dir.entries() {
            if let Some(file) = entry.as_file() {
                let relative_path = entry.path().strip_prefix(theme).unwrap();
                let target_path = target_theme_dir.join(relative_path);

                // Create parent directories if needed
                ensure_parent_exists(&target_path)?;

                fs::write(
                    &target_path,
                    file.contents_utf8().ok_or_else(|| {
                        format!(
                            "Theme file {} contains invalid UTF-8",
                            entry.path().display()
                        )
                    })?,
                )?;
            }
        }
    } else {
        return Err(format!("Theme '{}' not found", theme).into());
    }

    Ok(())
}

fn validate_inputs(
    path: &Path,
    theme: &str,
    no_theme: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate path
    if path.exists() {
        // Check if it's a directory with existing content
        if path.is_dir() && path.read_dir()?.next().is_some() {
            return Err(format!(
                "Directory '{}' is not empty - refusing to overwrite existing files",
                path.display()
            )
            .into());
        } else if !path.is_dir() {
            return Err(format!("Path '{}' exists but is not a directory", path.display()).into());
        }
    }

    // Check if parent directory exists and is writable
    if let Some(parent) = path.parent() {
        if parent.as_os_str().is_empty() {
            // Path is just a filename, current directory should be used
        } else if !parent.exists() {
            return Err(format!("Parent directory '{}' does not exist", parent.display()).into());
        }

        // Test writability by creating a temporary file
        let test_dir = if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        };
        let test_file = test_dir.join(".sherwood_write_test");
        match fs::write(&test_file, "test") {
            Ok(_) => {
                let _ = fs::remove_file(&test_file);
            }
            Err(e) => {
                return Err(format!(
                    "Cannot write to parent directory '{}': {}",
                    test_dir.display(),
                    e
                )
                .into());
            }
        }
    }

    // Validate theme if not using --no-theme
    if !no_theme {
        if theme.trim().is_empty() {
            return Err("Theme name cannot be empty".into());
        }

        // Validate theme name characters (alphanumeric, hyphens, underscores)
        if !theme
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(format!("Invalid theme name '{}': only letters, numbers, hyphens, and underscores are allowed", theme).into());
        }

        // Check if theme exists in embedded themes
        if THEMES.get_dir(theme).is_none() {
            return Err(format!(
                "Theme '{}' not found. Available themes: {}",
                theme,
                get_available_themes().join(", ")
            )
            .into());
        }
    }

    Ok(())
}

fn get_available_themes() -> Vec<String> {
    THEMES
        .dirs()
        .map(|dir| {
            dir.path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect()
}

fn print_success_message(path: &Path, theme: &str, no_theme: bool) {
    println!("✅ New Sherwood project created successfully!");
    println!("📁 Location: {}", path.display());
    println!("📝 Edit content/index.md to customize your site");

    if !no_theme {
        println!("🎨 Theme: {} (configured in sherwood.toml)", theme);
    }

    println!("🚀 Run `sherwood dev` to start development");
}
