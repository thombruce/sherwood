use anyhow::{anyhow, Result};
use glob::glob;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct TemplateResolver {
    templates: HashMap<String, PathBuf>,
}

impl TemplateResolver {
    pub fn new(templates_dir: &Path) -> Result<Self> {
        let mut templates = HashMap::new();
        
        // Find all .html.tera files
        let pattern = format!("{}/**/*.html.tera", templates_dir.display());
        
        for entry in glob(&pattern)? {
            let path = entry?;
            
            // Get the template name relative to templates_dir
            let relative_path = path.strip_prefix(templates_dir)?;
            let template_name = relative_path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid template path: {:?}", path))?
                .to_string();
            
            // Store template name both with and without .tera extension for flexibility
            let name_without_tera = template_name.trim_end_matches(".tera").to_string();
            templates.insert(name_without_tera, path.clone());
            templates.insert(template_name, path);
        }
        
        Ok(Self { templates })
    }

    pub fn find_best_template(&self, content_path: &Path) -> Result<Option<String>> {
        // Convert content path to template-style path
        let content_str = content_path.to_str()
            .ok_or_else(|| anyhow!("Invalid content path"))?;
        
        // Convert .md to .html for template matching
        let html_path = content_str.replace(".md", ".html");
        
        // Generate template candidates in priority order
        let candidates = self.generate_template_candidates(&html_path);
        
        // Return the first matching template
        for candidate in &candidates {
            if self.templates.contains_key(candidate) {
                return Ok(Some(candidate.clone()));
            }
        }
        
        Ok(None)
    }

    fn generate_template_candidates(&self, content_path: &str) -> Vec<String> {
        let mut candidates = Vec::new();
        let path = Path::new(content_path);
        
        // Extract parts of the path
        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        let filename = path.file_stem().unwrap_or_else(|| std::ffi::OsStr::new(""));
        let filename_str = filename.to_str().unwrap_or("");
        
        // 1. Exact path match (docs/getting-started.html)
        candidates.push(content_path.to_string());
        
        // 2. Directory pattern matches (docs/[slug].html, docs/[...slug].html)
        let parent_str = parent.to_str().unwrap_or("");
        if !parent_str.is_empty() {
            candidates.push(format!("{}/[slug].html", parent_str));
            candidates.push(format!("{}/[...slug].html", parent_str));
        }
        
        // 3. Nested pattern matches ([slug]/getting-started.html, [slug]/[slug].html, [slug]/[...slug].html)
        if !filename_str.is_empty() {
            candidates.push(format!("[slug]/{}.html", filename_str));
            candidates.push(format!("[slug]/[slug].html"));
            candidates.push(format!("[slug]/[...slug].html"));
        }
        
        // 4. Deep pattern matches ([...slug]/getting-started.html, [...slug]/[slug].html)
        if !filename_str.is_empty() {
            candidates.push(format!("[...slug]/{}.html", filename_str));
            candidates.push(format!("[...slug]/[slug].html"));
        }
        
        // 5. Root pattern match ([...slug].html)
        candidates.push("[...slug].html".to_string());
        
        candidates
    }

    pub fn list_templates(&self) -> Result<Vec<String>> {
        let mut templates = self.templates.keys().cloned().collect::<Vec<_>>();
        templates.sort();
        Ok(templates)
    }

    pub fn template_exists(&self, template_name: &str) -> bool {
        self.templates.contains_key(template_name)
    }

    pub fn get_template_path(&self, template_name: &str) -> Option<&PathBuf> {
        self.templates.get(template_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_template_candidate_generation() {
        let resolver = TemplateResolver::new(Path::new("/nonexistent")).unwrap();
        
        let candidates = resolver.generate_template_candidates("docs/getting-started.html");
        
        assert_eq!(candidates, vec![
            "docs/getting-started.html".to_string(),
            "docs/[slug].html".to_string(),
            "docs/[...slug].html".to_string(),
            "[slug]/getting-started.html".to_string(),
            "[slug]/[slug].html".to_string(),
            "[slug]/[...slug].html".to_string(),
            "[...slug]/getting-started.html".to_string(),
            "[...slug]/[slug].html".to_string(),
            "[...slug].html".to_string(),
        ]);
    }

    #[test]
    fn test_template_candidate_generation_root_level() {
        let resolver = TemplateResolver::new(Path::new("/nonexistent")).unwrap();
        
        let candidates = resolver.generate_template_candidates("about.html");
        
        assert_eq!(candidates, vec![
            "about.html".to_string(),
            "[slug]/about.html".to_string(),
            "[slug]/[slug].html".to_string(),
            "[slug]/[...slug].html".to_string(),
            "[...slug]/about.html".to_string(),
            "[...slug]/[slug].html".to_string(),
            "[...slug].html".to_string(),
        ]);
    }

    #[test]
    fn test_template_matching() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let templates_dir = temp_dir.path();
        
        // Create test templates
        fs::create_dir_all(templates_dir.join("docs"))?;
        fs::write(templates_dir.join("docs/[slug].html.tera"), "{{ title }}")?;
        fs::write(templates_dir.join("[...slug].html.tera"), "{{ title }}")?;
        
        let resolver = TemplateResolver::new(templates_dir)?;
        
        // Should find docs/[slug].html for docs/getting-started.html
        let template = resolver.find_best_template(Path::new("docs/getting-started.html"))?;
        assert_eq!(template, Some("docs/[slug].html".to_string()));
        
        // Should find [...slug].html for about.html
        let template = resolver.find_best_template(Path::new("about.html"))?;
        assert_eq!(template, Some("[...slug].html".to_string()));
        
        Ok(())
    }
}