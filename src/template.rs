use crate::template_resolver::TemplateResolver;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    pub title: String,
    pub content: String,
    pub frontmatter: serde_json::Value,
    pub path: String,
    pub url: String,
    pub site: SiteContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteContext {
    pub title: Option<String>,
    pub theme: Option<String>,
}

pub struct TemplateManager {
    tera: Tera,
    resolver: TemplateResolver,
    templates_dir: PathBuf,
}

impl TemplateManager {
    pub fn new(templates_dir: &Path) -> Result<Self> {
        let templates_dir = templates_dir.to_path_buf();
        
        // Ensure templates directory exists
        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir)?;
        }

        // Initialize Tera with the templates directory
        let mut tera = Tera::new(&format!("{}/**/*.html.tera", templates_dir.display()))?;
        
        // Configure Tera
        tera.autoescape_on(vec![".html"]);
        
        let resolver = TemplateResolver::new(&templates_dir)?;

        Ok(Self {
            tera,
            resolver,
            templates_dir,
        })
    }

    pub fn render_template(
        &self,
        content_path: &Path,
        context: TemplateContext,
    ) -> Result<String> {
        let template_name = self.resolver.find_best_template(content_path)?;
        
        match template_name {
            Some(name) => {
                let mut tera_context = Context::new();
                tera_context.insert("page", &context);
                
                // Try with .tera extension first, then without
                let template_with_tera = format!("{}.tera", name);
                let rendered = match self.tera.render(&template_with_tera, &tera_context) {
                    Ok(html) => html,
                    Err(_) => {
                        // Try without .tera extension
                        self.tera.render(&name, &tera_context)?
                    }
                };
                
                Ok(rendered)
            }
            None => {
                // Fallback to basic HTML if no template found
                Ok(self.generate_fallback_html(&context))
            }
        }
    }

    pub fn reload_templates(&mut self) -> Result<()> {
        // Rebuild the template engine
        self.tera = Tera::new(&format!("{}/**/*.html.tera", self.templates_dir.display()))?;
        self.tera.autoescape_on(vec![".html"]);
        
        // Reload the resolver
        self.resolver = TemplateResolver::new(&self.templates_dir)?;
        
        Ok(())
    }

    fn generate_fallback_html(&self, context: &TemplateContext) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
</head>
<body>
    <main>
        {content}
    </main>
</body>
</html>"#,
            title = context.title,
            content = context.content
        )
    }

    pub fn list_available_templates(&self) -> Result<Vec<String>> {
        self.resolver.list_templates()
    }
}