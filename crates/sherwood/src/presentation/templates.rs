use anyhow::{Result, anyhow};
use include_dir::{Dir, include_dir};
use sailfish::{TemplateOnce, runtime::RenderError};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {template_name}")]
    TemplateNotFound { template_name: String },

    #[error("Invalid template format: {template_name} - {details}")]
    InvalidTemplate {
        template_name: String,
        details: String,
    },

    #[error("Template validation failed: {template_name} - {reason}")]
    ValidationFailed {
        template_name: String,
        reason: String,
    },

    #[error("No templates directory found at: {path}")]
    TemplatesDirectoryNotFound { path: String },

    #[error("Template compilation error: {template_name} - {source}")]
    CompilationError {
        template_name: String,
        #[source]
        source: RenderError,
    },
}

// Embed templates directory at compile time
static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

#[derive(TemplateOnce)]
#[template(path = "default.stpl")]
struct PageTemplate {
    title: String,
    content: String,
    css_file: Option<String>,
    body_attrs: String,
}

#[derive(TemplateOnce)]
#[template(path = "content_item.stpl")]
struct ContentItemTemplate {
    title: String,
    url: String,
    date: Option<String>,
    excerpt: Option<String>,
}

#[derive(Debug)]
pub struct TemplateInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: usize,
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub struct TemplateManager {
    templates_dir: PathBuf,
    available_templates: Vec<String>,
}

impl TemplateManager {
    pub fn new(templates_dir: &Path) -> Result<Self> {
        let templates_dir = templates_dir.to_path_buf();

        // Templates directory is optional - we have embedded fallbacks

        let available_templates = Self::discover_templates(&templates_dir)?;

        Ok(Self {
            templates_dir,
            available_templates,
        })
    }

    /// Discover all available template files in the templates directory
    fn discover_templates(templates_dir: &Path) -> Result<Vec<String>> {
        let mut templates = Vec::new();

        // First, add embedded templates
        templates.extend(get_available_templates());

        // Then scan the templates directory for additional templates
        if templates_dir.exists() && templates_dir.is_dir() {
            for entry in fs::read_dir(templates_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file()
                    && let Some(extension) = path.extension()
                    && extension == "stpl"
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                {
                    templates.push(name.to_string());
                }
            }
        }

        // Remove duplicates and sort
        templates.sort();
        templates.dedup();

        Ok(templates)
    }

    /// Get information about all available templates
    pub fn get_template_info(&self) -> Vec<TemplateInfo> {
        self.available_templates
            .iter()
            .map(|name| {
                let path = self.templates_dir.join(name);
                let size = if path.exists() {
                    fs::metadata(&path).map(|m| m.len() as usize).unwrap_or(0)
                } else {
                    // Check embedded templates
                    TEMPLATES
                        .get_file(name)
                        .map(|f| f.contents().len())
                        .unwrap_or(0)
                };

                TemplateInfo {
                    name: name.clone(),
                    path: path.clone(),
                    size,
                    is_valid: self.validate_template(name).is_ok(),
                }
            })
            .collect()
    }

    /// Validate a specific template
    pub fn validate_template(&self, template_name: &str) -> Result<()> {
        // Check if template exists
        if !self
            .available_templates
            .contains(&template_name.to_string())
        {
            return Err(TemplateError::TemplateNotFound {
                template_name: template_name.to_string(),
            }
            .into());
        }

        // Try to read the template content
        let template_content = self.get_template_content(template_name)?;

        // Basic validation checks
        self.validate_template_syntax(template_name, &template_content)?;

        // Try to compile the template (this is a more thorough validation)
        self.validate_template_compilation(template_name)?;

        Ok(())
    }

    /// Get template content from filesystem or embedded templates
    fn get_template_content(&self, template_name: &str) -> Result<String> {
        // Try filesystem first
        let fs_path = self.templates_dir.join(template_name);
        if fs_path.exists() {
            return Ok(fs::read_to_string(&fs_path)?);
        }

        // Fall back to embedded templates
        if let Some(embedded_file) = TEMPLATES.get_file(template_name) {
            return Ok(embedded_file
                .contents_utf8()
                .ok_or_else(|| TemplateError::InvalidTemplate {
                    template_name: template_name.to_string(),
                    details: "Template contains invalid UTF-8".to_string(),
                })?
                .to_string());
        }

        Err(TemplateError::TemplateNotFound {
            template_name: template_name.to_string(),
        }
        .into())
    }

    /// Basic template syntax validation
    fn validate_template_syntax(&self, template_name: &str, content: &str) -> Result<()> {
        // Check for balanced template delimiters
        let open_count = content.matches("<%").count();
        let close_count = content.matches("%>").count();

        if open_count != close_count {
            return Err(TemplateError::ValidationFailed {
                template_name: template_name.to_string(),
                reason: format!(
                    "Unbalanced template delimiters: {} opening, {} closing",
                    open_count, close_count
                ),
            }
            .into());
        }

        // Check for obvious syntax errors
        if content.matches("<%").any(|m| m.ends_with("%>")) {
            return Err(TemplateError::ValidationFailed {
                template_name: template_name.to_string(),
                reason: "Empty template blocks found".to_string(),
            }
            .into());
        }

        Ok(())
    }

    /// Validate template by attempting compilation
    fn validate_template_compilation(&self, template_name: &str) -> Result<()> {
        // For the known templates, we can test compilation
        match template_name {
            "default.stpl" => {
                let template = PageTemplate {
                    title: "test".to_string(),
                    content: "test".to_string(),
                    css_file: Some("/test.css".to_string()),
                    body_attrs: "test".to_string(),
                };
                template
                    .render_once()
                    .map_err(|e| TemplateError::CompilationError {
                        template_name: template_name.to_string(),
                        source: e,
                    })?;
            }
            "content_item.stpl" => {
                let template = ContentItemTemplate {
                    title: "test".to_string(),
                    url: "test".to_string(),
                    date: Some("2024-01-01".to_string()),
                    excerpt: Some("test".to_string()),
                };
                template
                    .render_once()
                    .map_err(|e| TemplateError::CompilationError {
                        template_name: template_name.to_string(),
                        source: e,
                    })?;
            }
            _ => {
                // For unknown templates, just check if they exist
                // We could potentially implement generic template validation here
            }
        }

        Ok(())
    }

    /// Validate all available templates
    pub fn validate_all_templates(&self) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        for template_name in &self.available_templates {
            if let Err(e) = self.validate_template(template_name) {
                errors.push(format!("Template '{}': {}", template_name, e));
            }
        }

        if errors.is_empty() {
            println!(
                "✅ All {} templates validated successfully",
                self.available_templates.len()
            );
            Ok(errors)
        } else {
            println!("❌ Found {} template validation errors:", errors.len());
            for error in &errors {
                println!("  - {}", error);
            }
            Ok(errors)
        }
    }

    /// Get list of available template names
    pub fn get_available_templates(&self) -> Vec<String> {
        self.available_templates.clone()
    }

    /// Print template information (for debugging)
    pub fn debug_print_templates(&self) {
        println!(
            "📋 Available templates in {}:",
            self.templates_dir.display()
        );
        let info = self.get_template_info();

        for template_info in info {
            let status = if template_info.is_valid { "✅" } else { "❌" };
            println!(
                "  {} {} ({} bytes)",
                status, template_info.name, template_info.size
            );
        }

        if self.available_templates.is_empty() {
            println!("  No templates found");
        }
    }

    pub fn render_page(
        &self,
        title: &str,
        content: &str,
        css_file: Option<&str>,
        body_attrs: &str,
    ) -> Result<String> {
        // Validate template before rendering
        self.validate_template("default.stpl")?;

        let template = PageTemplate {
            title: title.to_string(),
            content: content.to_string(),
            css_file: css_file.map(|s| s.to_string()),
            body_attrs: body_attrs.to_string(),
        };

        Ok(template.render_once()?)
    }

    pub fn render_content_item(
        &self,
        title: &str,
        url: &str,
        date: Option<&str>,
        excerpt: Option<&str>,
    ) -> Result<String> {
        // Validate template before rendering
        self.validate_template("content_item.stpl")?;

        let template = ContentItemTemplate {
            title: title.to_string(),
            url: url.to_string(),
            date: date.map(|s| s.to_string()),
            excerpt: excerpt.map(|s| s.to_string()),
        };

        Ok(template.render_once()?)
    }

    pub fn get_template_path(&self, template_name: &str) -> PathBuf {
        self.templates_dir.join(template_name)
    }

    pub fn template_exists(&self, template_name: &str) -> bool {
        self.templates_dir.join(template_name).exists()
    }
}

pub fn copy_embedded_templates(output_dir: &Path) -> Result<()> {
    let templates_output_dir = output_dir.join("templates");
    fs::create_dir_all(&templates_output_dir)?;

    for entry in TEMPLATES.entries() {
        if let Some(file) = entry.as_file() {
            let template_name = entry
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid template name"))?;

            let output_path = templates_output_dir.join(template_name);
            fs::write(
                &output_path,
                file.contents_utf8().ok_or_else(|| {
                    anyhow::anyhow!("Template {} contains invalid UTF-8", template_name)
                })?,
            )?;
        }
    }

    Ok(())
}

pub fn get_available_templates() -> Vec<String> {
    TEMPLATES
        .files()
        .map(|file| {
            file.path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect()
}

/// Validate templates in the specified directory
pub fn validate_templates(templates_dir: &Option<PathBuf>, verbose: bool) -> Result<()> {
    let templates_path = match templates_dir {
        Some(path) => path.clone(),
        None => {
            // Default to ../templates relative to current directory
            std::env::current_dir()
                .unwrap_or_default()
                .join("../templates")
        }
    };

    match TemplateManager::new(&templates_path) {
        Ok(template_manager) => {
            if verbose {
                template_manager.debug_print_templates();
            }

            let errors = template_manager.validate_all_templates()?;
            if errors.is_empty() {
                println!("🎉 All templates are valid!");
                Ok(())
            } else {
                Err(anyhow!(
                    "Template validation failed with {} errors",
                    errors.len()
                ))
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("❌ Failed to initialize template manager: {}", e);
            }
            Err(e)
        }
    }
}
