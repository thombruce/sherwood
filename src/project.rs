use std::fs;
use std::path::Path;
use include_dir::{include_dir, Dir};

// Embed templates directory at compile time
static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");
static THEMES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates/themes");

pub fn create_new_project(path: &Path, theme: &str, no_theme: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Create content directory
    let content_dir = path.join("content");
    fs::create_dir_all(&content_dir)?;
    
    // Copy index.md from templates
    copy_template_file(&TEMPLATES, "content/index.md", &content_dir.join("index.md"))?;
    
    // Copy and process sherwood.toml template
    copy_config_template(path, theme, no_theme)?;
    
    // Copy theme files if requested
    if !no_theme {
        copy_theme_files(path, theme)?;
    }
    
    // Print success message
    print_success_message(path, theme, no_theme);
    
    Ok(())
}

fn copy_template_file(templates_dir: &Dir, template_path: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(file) = templates_dir.get_file(template_path) {
        fs::write(output_path, file.contents_utf8().unwrap())?;
    } else {
        return Err(format!("Template file not found: {}", template_path).into());
    }
    Ok(())
}

fn copy_config_template(path: &Path, theme: &str, no_theme: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = path.join("sherwood.toml");
    
    if let Some(file) = TEMPLATES.get_file("config/sherwood.toml") {
        let config_content = file.contents_utf8().unwrap();
        
        let processed_content = if no_theme {
            // Remove theme line when --no-theme is used
            config_content.lines()
                .filter(|line| !line.trim_start().starts_with("theme ="))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            // Replace default theme with selected theme
            config_content.replace("theme = \"default\"", &format!("theme = \"{}\"", theme))
        };
        
        fs::write(config_path, processed_content)?;
    } else {
        return Err("Config template file not found".into());
    }
    
    Ok(())
}

fn copy_theme_files(path: &Path, theme: &str) -> Result<(), Box<dyn std::error::Error>> {
    let themes_dir = path.join("themes");
    fs::create_dir_all(&themes_dir)?;
    
    if let Some(theme_dir) = THEMES.get_dir(theme) {
        let target_theme_dir = themes_dir.join(theme);
        fs::create_dir_all(&target_theme_dir)?;
        
        // Copy all files from theme directory
        for entry in theme_dir.entries() {
            if let Some(file) = entry.as_file() {
                let relative_path = entry.path().strip_prefix(theme).unwrap();
                let target_path = target_theme_dir.join(relative_path);
                
                // Create parent directories if needed
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                fs::write(&target_path, file.contents_utf8().unwrap())?;
            }
        }
    } else {
        return Err(format!("Theme '{}' not found", theme).into());
    }
    
    Ok(())
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