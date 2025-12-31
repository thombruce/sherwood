use crate::config::{NavigationItem, SiteConfig};
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
    pub navigation: Vec<NavigationItem>,
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
    site_config: SiteConfig,
}

impl TemplateManager {
    /// Normalize template name to ensure it has the proper .html.tera extension
    fn normalize_template_name(template_name: &str) -> String {
        let normalized = if template_name.ends_with(".html.tera") {
            // Already has the correct extension
            template_name.to_string()
        } else if template_name.ends_with(".tera") {
            // Has .tera but missing .html - insert .html before .tera
            template_name.trim_end_matches(".tera").to_string() + ".html.tera"
        } else if template_name.ends_with(".html") {
            // Has .html but missing .tera
            template_name.to_string() + ".tera"
        } else {
            // No extension - add both
            template_name.to_string() + ".html.tera"
        };
        
        normalized
    }

    pub fn new(templates_dir: &Path, site_config: SiteConfig) -> Result<Self> {
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
            site_config,
        })
    }

    pub fn render_template(&self, content_path: &Path, context: TemplateContext) -> Result<String> {
        let template_name = self.resolver.find_best_template(content_path)?;

        // Add navigation to context
        let navigation = self
            .site_config
            .site
            .navigation
            .clone()
            .map(|nav| nav.items)
            .unwrap_or_default();

        let template_context = TemplateContext {
            navigation,
            ..context
        };

        match template_name {
            Some(name) => {
                let mut tera_context = Context::new();
                tera_context.insert("page", &template_context);

                // Normalize template name to ensure proper .html.tera extension
                let normalized_name = Self::normalize_template_name(&name);
                
                // Try rendering with the normalized name
                match self.tera.render(&normalized_name, &tera_context) {
                    Ok(html) => Ok(html),
                    Err(e) => {
                        // If normalized name fails, try the original name for backward compatibility
                        if normalized_name != name {
                            match self.tera.render(&name, &tera_context) {
                                Ok(html) => Ok(html),
                                Err(_) => {
                                    // Final fallback to basic HTML
                                    eprintln!(
                                        "Warning: Template '{}' (tried '{}') failed to render: {}. Using fallback HTML.",
                                        name, normalized_name, e
                                    );
                                    Ok(self.generate_fallback_html(&template_context))
                                }
                            }
                        } else {
                            eprintln!(
                                "Warning: Template '{}' failed to render: {}. Using fallback HTML.",
                                name, e
                            );
                            Ok(self.generate_fallback_html(&template_context))
                        }
                    }
                }
            }
            None => {
                // Fallback to basic HTML if no template found
                Ok(self.generate_fallback_html(&template_context))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_template_name() {
        let test_cases = vec![
            ("blog", "blog.html.tera"),
            ("blog.html", "blog.html.tera"), 
            ("blog.tera", "blog.html.tera"),
            ("blog.html.tera", "blog.html.tera"),
            ("docs/page", "docs/page.html.tera"),
            ("docs/page.html", "docs/page.html.tera"),
            ("docs/page.tera", "docs/page.html.tera"),
            ("docs/page.html.tera", "docs/page.html.tera"),
        ];

        for (input, expected) in test_cases {
            assert_eq!(TemplateManager::normalize_template_name(input), expected);
        }
    }
}
