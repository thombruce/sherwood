use crate::config::{SiteConfig, SiteSection};
use crate::template::{SiteContext, TemplateContext, TemplateManager};
use crate::themes::ThemeManager;
use crate::utils::{ensure_directory_exists, ensure_parent_exists};
use anyhow::Result;
use pulldown_cmark::{Options, Parser, html};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

#[derive(Debug, Deserialize, Serialize, Default)]
struct Frontmatter {
    title: Option<String>,
    date: Option<String>,
    list: Option<bool>,
    theme: Option<String>,
    theme_variant: Option<String>,
    template: Option<String>,
}

#[derive(Debug)]
struct MarkdownFile {
    path: PathBuf,
    content: String,
    frontmatter: Frontmatter,
    title: String,
}

pub struct SiteGenerator {
    input_dir: PathBuf,
    output_dir: PathBuf,
    theme_manager: ThemeManager,
    site_config: SiteConfig,
    template_manager: TemplateManager,
}

impl SiteGenerator {
    pub fn new(input_dir: &Path, output_dir: &Path) -> Result<Self> {
        let themes_dir = input_dir
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("themes");

        let templates_dir = input_dir
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("templates");

        // Load site configuration
        let config_path = input_dir
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("sherwood.toml");

        let site_config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            SiteConfig {
                site: SiteSection {
                    theme: Some("default".to_string()),
                    navigation: None,
                },
            }
        };

        Ok(Self {
            input_dir: input_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            theme_manager: ThemeManager::new(&themes_dir),
            template_manager: TemplateManager::new(&templates_dir, site_config.clone())?,
            site_config,
        })
    }

    pub async fn generate(&self) -> Result<()> {
        // Clean output directory
        if self.output_dir.exists() {
            fs::remove_dir_all(&self.output_dir)?;
        }
        ensure_directory_exists(&self.output_dir)?;

        // Generate CSS if theme is configured
        self.generate_theme_css()?;

        // Find all markdown files
        let markdown_files = self.find_markdown_files(&self.input_dir)?;

        if markdown_files.is_empty() {
            println!("No markdown files found in {}", self.input_dir.display());
            return Ok(());
        }

        // Parse all markdown files to extract metadata
        let mut parsed_files = Vec::new();
        for file_path in markdown_files {
            let parsed = self.parse_markdown_file(&file_path)?;
            parsed_files.push(parsed);
        }

        // Find list pages and generate their content
        let mut list_pages = HashMap::new();
        for file in &parsed_files {
            if file.frontmatter.list.unwrap_or(false) {
                let relative_path = file.path.strip_prefix(&self.input_dir)?;
                let parent_dir = relative_path.parent().unwrap_or_else(|| Path::new(""));
                list_pages.insert(parent_dir.to_path_buf(), file);
            }
        }

        // Process each markdown file
        for file in &parsed_files {
            self.process_markdown_file(file, &list_pages).await?;
        }

        println!(
            "Site generated successfully in {}",
            self.output_dir.display()
        );
        Ok(())
    }

    fn find_markdown_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut dirs_to_visit = Vec::new();

        if !dir.exists() {
            println!("Content directory {} does not exist", dir.display());
            return Ok(files);
        }

        dirs_to_visit.push(dir.to_path_buf());

        while let Some(current_dir) = dirs_to_visit.pop() {
            for entry in fs::read_dir(&current_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    dirs_to_visit.push(path);
                } else if let Some(extension) = path.extension()
                    && (extension == "md" || extension == "markdown")
                {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    fn parse_markdown_file(&self, file_path: &Path) -> Result<MarkdownFile> {
        let content = fs::read_to_string(file_path)?;

        // Parse frontmatter and extract content
        let (frontmatter, markdown_content) = self.parse_frontmatter(&content)?;

        // Extract title from frontmatter, first h1, or filename
        let title = frontmatter
            .title
            .clone()
            .or_else(|| self.extract_title(&markdown_content))
            .unwrap_or_else(|| {
                file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string()
            });

        Ok(MarkdownFile {
            path: file_path.to_path_buf(),
            content: markdown_content,
            frontmatter,
            title,
        })
    }

    fn parse_frontmatter(&self, content: &str) -> Result<(Frontmatter, String)> {
        if content.starts_with("---\n") {
            let parts: Vec<&str> = content.splitn(3, "---\n").collect();
            if parts.len() >= 3 {
                let frontmatter_str = parts[1];
                let markdown_content = parts[2];

                let frontmatter: Frontmatter =
                    serde_yaml::from_str(frontmatter_str).unwrap_or_default();
                return Ok((frontmatter, markdown_content.to_string()));
            }
        }

        Ok((Frontmatter::default(), content.to_string()))
    }

    async fn process_markdown_file(
        &self,
        file: &MarkdownFile,
        list_pages: &HashMap<PathBuf, &MarkdownFile>,
    ) -> Result<()> {
        let relative_path = file.path.strip_prefix(&self.input_dir)?;

        // Convert .md to .html
        let html_path = self.output_dir.join(relative_path).with_extension("html");

        // Create parent directories if needed
        ensure_parent_exists(&html_path)?;

        // Check if this is a list page that needs post generation
        let content = if file.frontmatter.list.unwrap_or(false) {
            let parent_dir = relative_path.parent().unwrap_or_else(|| Path::new(""));
            let blog_list = self.generate_blog_list_content(parent_dir, list_pages)?;

            // Replace the placeholder with the actual blog list
            file.content.replace("<!-- BLOG_POSTS_LIST -->", &blog_list)
        } else {
            file.content.clone()
        };

        // Convert markdown to HTML with semantic structure
        let html_content = self.markdown_to_semantic_html(&content)?;

        // Try to use template rendering first
        let full_html = if let Some(template_name) = &file.frontmatter.template {
            // Use explicitly specified template - normalize the name
            let template_path = if template_name.ends_with(".html.tera") {
                template_name.clone()
            } else if template_name.ends_with(".tera") {
                // Has .tera but missing .html
                template_name.trim_end_matches(".tera").to_string() + ".html.tera"
            } else if template_name.ends_with(".html") {
                // Has .html but missing .tera
                template_name.to_string() + ".tera"
            } else {
                // No extension - add both
                template_name.to_string() + ".html.tera"
            };

            let context = self.build_template_context(file, relative_path, &html_content)?;

            match self
                .template_manager
                .render_template(Path::new(&template_path), context)
            {
                Ok(rendered) => rendered,
                Err(_) => {
                    // Fallback to template resolver if explicit template fails
                    self.render_with_template_resolver(file, relative_path, &html_content)?
                }
            }
        } else {
            // Use template resolver
            self.render_with_template_resolver(file, relative_path, &html_content)?
        };

        fs::write(&html_path, full_html)?;
        println!("Generated: {}", html_path.display());

        Ok(())
    }

    fn extract_title(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix("# ") {
                return Some(stripped.trim().to_string());
            }
        }
        None
    }

    fn generate_blog_list_content(
        &self,
        dir: &Path,
        _list_pages: &HashMap<PathBuf, &MarkdownFile>,
    ) -> Result<String> {
        let mut list_content = String::new();

        // Find all markdown files in this directory (excluding index.md)
        for entry in fs::read_dir(self.input_dir.join(dir))? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(extension) = path.extension()
                && (extension == "md" || extension == "markdown")
            {
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                // Skip index files and other list pages
                if !file_name.starts_with("index") {
                    let parsed = self.parse_markdown_file(&path)?;

                    // Generate post list entry
                    let date = parsed.frontmatter.date.as_deref().unwrap_or("");
                    let relative_url_path = path
                        .strip_prefix(&self.input_dir)
                        .unwrap_or(&path)
                        .with_extension("");
                    let relative_url = relative_url_path.to_string_lossy();

                    list_content.push_str(&format!(
                        r##"<article class="blog-post">
    <h2><a href="/{}">{}</a></h2>
    {}"##,
                        relative_url,
                        parsed.title,
                        if !date.is_empty() {
                            format!("<p class=\"post-date\">{}</p>", date)
                        } else {
                            String::new()
                        }
                    ));

                    // Extract first paragraph as excerpt
                    let first_paragraph = self.extract_first_paragraph(&parsed.content);
                    if !first_paragraph.is_empty() {
                        let parser = Parser::new(&first_paragraph);
                        let mut excerpt_html = String::new();
                        html::push_html(&mut excerpt_html, parser);
                        list_content
                            .push_str(&format!("<p class=\"post-excerpt\">{}</p>", excerpt_html));
                    }

                    list_content.push_str("</article>\n\n");
                }
            }
        }

        // If no list content was found, return empty string
        if list_content.is_empty() {
            Ok("<!-- No posts found -->".to_string())
        } else {
            Ok(list_content)
        }
    }

    fn extract_first_paragraph(&self, content: &str) -> String {
        let mut in_code_block = false;
        let mut lines_since_heading = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip code blocks
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Skip headings and empty lines right after headings
            if trimmed.starts_with('#') {
                lines_since_heading = 0;
                continue;
            }
            if lines_since_heading < 1 {
                lines_since_heading += 1;
                continue;
            }

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Found a paragraph, return it
            return trimmed.to_string();
        }

        String::new()
    }

    fn markdown_to_semantic_html(&self, markdown: &str) -> Result<String> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Ok(self.enhance_semantics(&html_output))
    }

    fn enhance_semantics(&self, html: &str) -> String {
        let mut enhanced = html.to_string();

        // Wrap paragraphs in semantic sections if they seem like articles
        enhanced = self.wrap_articles(&enhanced);

        // Add semantic structure to lists
        enhanced = self.enhance_lists(&enhanced);

        enhanced
    }

    fn wrap_articles(&self, html: &str) -> String {
        // Simple heuristic: if content has multiple headings, wrap in article tags
        let heading_count = html.matches("<h").count();
        if heading_count > 1 {
            format!("<article>\n{}\n</article>", html)
        } else {
            html.to_string()
        }
    }

    fn enhance_lists(&self, html: &str) -> String {
        // Convert plain lists to more semantic versions when appropriate
        html.replace("<ul>", "<ul class=\"content-list\">")
            .replace("<ol>", "<ol class=\"numbered-list\">")
    }

    fn generate_theme_css(&self) -> Result<()> {
        // Check if theme is configured
        if let Some(theme_name) = self.site_config.site.theme.clone() {
            // Only generate theme if explicitly configured in sherwood.toml
            let theme = self.theme_manager.load_theme(&theme_name)?;
            let css_path = self
                .theme_manager
                .generate_css_file(&theme, &self.output_dir)?;
            println!("Generated CSS: {}", css_path.display());
        } else {
            // No theme configured - skip CSS generation
            println!("No theme configured - skipping CSS generation");
        }
        Ok(())
    }

    fn build_template_context(
        &self,
        file: &MarkdownFile,
        relative_path: &Path,
        html_content: &str,
    ) -> Result<TemplateContext> {
        let site_context = SiteContext {
            title: None, // Could be added to site config later
            theme: self.site_config.site.theme.clone(),
        };

        let context = TemplateContext {
            title: file.title.clone(),
            content: html_content.to_string(),
            frontmatter: serde_json::to_value(&file.frontmatter)?,
            path: relative_path.to_string_lossy().to_string(),
            url: relative_path
                .with_extension("")
                .to_string_lossy()
                .to_string(),
            site: site_context,
            navigation: vec![], // Will be populated by template manager
        };

        Ok(context)
    }

    fn render_with_template_resolver(
        &self,
        file: &MarkdownFile,
        relative_path: &Path,
        html_content: &str,
    ) -> Result<String> {
        let context = self.build_template_context(file, relative_path, html_content)?;

        match self
            .template_manager
            .render_template(relative_path, context)
        {
            Ok(rendered) => Ok(rendered),
            Err(e) => {
                eprintln!(
                    "Warning: Template rendering failed: {}. Using fallback HTML.",
                    e
                );
                Ok(self.generate_fallback_html(&file.title, html_content))
            }
        }
    }

    fn generate_fallback_html(&self, title: &str, content: &str) -> String {
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
            title = title,
            content = content
        )
    }
}

pub async fn generate_site(input_dir: &Path, output_dir: &Path) -> Result<()> {
    let generator = SiteGenerator::new(input_dir, output_dir)?;
    generator.generate().await
}
