use crate::config::{NavigationItem, SiteConfig};
use crate::template_resolver::TemplateResolver;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
    #[allow(dead_code)] // May be needed for future hot reload functionality
    templates_dir: PathBuf,
    site_config: SiteConfig,
    // Lazy loading cache for tracking which templates have been loaded
    loaded_templates: HashSet<String>,
}

impl TemplateManager {
    /// Normalize template name to ensure it has the proper .html.tera extension
    fn normalize_template_name(template_name: &str) -> String {
        if template_name.ends_with(".html.tera") {
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
        }
    }

    pub fn new(templates_dir: &Path, site_config: SiteConfig) -> Result<Self> {
        let templates_dir = templates_dir.to_path_buf();

        // Ensure templates directory exists
        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir)?;
        }

        // Initialize Tera with empty template set for lazy loading
        let mut tera = Tera::default();
        tera.autoescape_on(vec![".html"]);

        // Initialize resolver with lazy loading
        let resolver = TemplateResolver::new(&templates_dir)?;

        Ok(Self {
            tera,
            resolver,
            templates_dir,
            site_config,
            loaded_templates: HashSet::new(),
        })
    }

    /// Load a specific template on demand if not already loaded
    fn ensure_template_loaded(&mut self, template_name: &str) -> Result<()> {
        let normalized_name = Self::normalize_template_name(template_name);
        
        // Check if already loaded
        if self.loaded_templates.contains(&normalized_name) {
            return Ok(());
        }

        // Find the template file path
        if let Some(template_path) = self.resolver.get_template_path(&normalized_name) {
            // Load the template file content
            let _template_content = fs::read_to_string(template_path)?;
            
            // Add template to Tera
            self.tera.add_template_file(template_path, Some(&normalized_name))?;
            
            // Mark as loaded
            self.loaded_templates.insert(normalized_name);
            
            Ok(())
        } else {
            // Try the original template name for backward compatibility
            if normalized_name != template_name {
                if let Some(template_path) = self.resolver.get_template_path(template_name) {
                    let _template_content = fs::read_to_string(template_path)?;
                    self.tera.add_template_file(template_path, Some(template_name))?;
                    self.loaded_templates.insert(template_name.to_string());
                    return Ok(());
                }
            }
            
            Err(anyhow::anyhow!("Template '{}' not found", template_name))
        }
    }

    pub fn render_template(&mut self, content_path: &Path, context: TemplateContext) -> Result<String> {
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
                // Ensure template is loaded before rendering
                if let Err(e) = self.ensure_template_loaded(&name) {
                    eprintln!(
                        "Warning: Failed to load template '{}': {}. Using fallback HTML.",
                        name, e
                    );
                    return Ok(self.generate_fallback_html(&template_context));
                }

                let mut tera_context = Context::new();
                tera_context.insert("page", &template_context);

                // Normalize template name to ensure proper .html.tera extension
                let normalized_name = Self::normalize_template_name(&name);
                
                // Try rendering with the normalized name first
                match self.tera.render(&normalized_name, &tera_context) {
                    Ok(html) => Ok(html),
                    Err(e) => {
                        // If normalized name fails, try the original name for backward compatibility
                        if normalized_name != name {
                            match self.tera.render(&name, &tera_context) {
                                Ok(html) => Ok(html),
                                Err(render_err) => {
                                    // Final fallback to basic HTML
                                    eprintln!(
                                        "Warning: Template '{}' (tried '{}') failed to render: {}. Using fallback HTML.",
                                        name, normalized_name, render_err
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

    // TODO: Implement hot reloading functionality before uncommenting this method
    // pub fn reload_templates(&mut self) -> Result<()> {
    //     // Rebuild the template engine
    //     self.tera = Tera::new(&format!("{}/**/*.html.tera", self.templates_dir.display()))?;
    //     self.tera.autoescape_on(vec![".html"]);
    //
    //     // Reload the resolver
    //     self.resolver = TemplateResolver::new(&self.templates_dir)?;
    //
    //     Ok(())
    // }

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

    /// Get statistics about template loading
    pub fn get_loading_stats(&self) -> (usize, usize) {
        (self.loaded_templates.len(), self.resolver.list_templates().unwrap_or_default().len())
    }

    /// Check if a specific template is loaded
    pub fn is_template_loaded(&self, template_name: &str) -> bool {
        let normalized_name = Self::normalize_template_name(template_name);
        self.loaded_templates.contains(&normalized_name) || self.loaded_templates.contains(template_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

    #[test]
    fn test_lazy_loading() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let templates_dir = temp_dir.path();

        // Create test templates
        fs::create_dir_all(templates_dir.join("blog"))?;
        fs::write(templates_dir.join("blog/[slug].html.tera"), "{{ title }}")?;
        fs::write(templates_dir.join("[...slug].html.tera"), "{{ title }}")?;

        let site_config = crate::config::SiteConfig {
            site: crate::config::SiteSection {
                theme: Some("default".to_string()),
                navigation: None,
            },
        };

        let mut manager = TemplateManager::new(templates_dir, site_config)?;

        // Initially no templates should be loaded
        let (loaded, total) = manager.get_loading_stats();
        assert_eq!(loaded, 0);
        assert!(total >= 2); // At least 2 templates exist

        // Template should not be loaded initially
        assert!(!manager.is_template_loaded("[...slug].html"));

        // Render a template - should load it on demand
        let context = TemplateContext {
            title: "Test Title".to_string(),
            content: "Test content".to_string(),
            frontmatter: serde_json::Value::Null,
            path: "about.html".to_string(),
            url: "about".to_string(),
            site: crate::template::SiteContext {
                title: None,
                theme: Some("default".to_string()),
            },
            navigation: vec![],
        };

        manager.render_template(Path::new("about.html"), context)?;

        // Now the template should be loaded
        assert!(manager.is_template_loaded("[...slug].html"));

        let (loaded_after, _) = manager.get_loading_stats();
        assert!(loaded_after > loaded);

        Ok(())
    }
}
