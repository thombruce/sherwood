use anyhow::Result;
use include_dir::{Dir, include_dir};
use sailfish::TemplateOnce;
use std::fs;
use std::path::{Path, PathBuf};

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
#[template(path = "blog_post.stpl")]
struct BlogPostTemplate {
    title: String,
    url: String,
    date: Option<String>,
    excerpt: Option<String>,
}

pub struct TemplateManager {
    templates_dir: PathBuf,
}

impl TemplateManager {
    pub fn new(templates_dir: &Path) -> Self {
        Self {
            templates_dir: templates_dir.to_path_buf(),
        }
    }

    pub fn render_page(
        &self,
        title: &str,
        content: &str,
        css_file: Option<&str>,
        body_attrs: &str,
    ) -> Result<String> {
        let template = PageTemplate {
            title: title.to_string(),
            content: content.to_string(),
            css_file: css_file.map(|s| s.to_string()),
            body_attrs: body_attrs.to_string(),
        };

        Ok(template.render_once()?)
    }

    pub fn render_blog_post(
        &self,
        title: &str,
        url: &str,
        date: Option<&str>,
        excerpt: Option<&str>,
    ) -> Result<String> {
        let template = BlogPostTemplate {
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
