use crate::config::{SiteConfig, SiteSection, TemplateSection};
use crate::templates::TemplateManager;
use crate::themes::ThemeManager;
use crate::utils::{ensure_directory_exists, ensure_parent_exists};
use anyhow::Result;
use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

// Constants for relative path construction
const THEMES_DIR_RELATIVE: &str = "../themes";
const TEMPLATES_DIR_RELATIVE: &str = "../templates";
const CONFIG_PATH_RELATIVE: &str = "../sherwood.toml";

// Constants for template names
const DEFAULT_PAGE_TEMPLATE: &str = "default.stpl";

#[derive(Debug, Deserialize, Default)]
struct Frontmatter {
    title: Option<String>,
    date: Option<String>,
    list: Option<bool>,
    theme: Option<String>,
    theme_variant: Option<String>,
    page_template: Option<String>,
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
    template_manager: TemplateManager,
    site_config: SiteConfig,
}

impl SiteGenerator {
    pub fn new(input_dir: &Path, output_dir: &Path) -> Result<Self> {
        let themes_dir = input_dir.join(THEMES_DIR_RELATIVE);
        let templates_dir = input_dir.join(TEMPLATES_DIR_RELATIVE);

        // Load site configuration
        let config_path = input_dir.join(CONFIG_PATH_RELATIVE);
        let site_config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            SiteConfig {
                site: SiteSection {
                    theme: Some("default".to_string()),
                },
                templates: Some(TemplateSection {
                    page_template: Some(DEFAULT_PAGE_TEMPLATE.to_string()),
                }),
            }
        };

        Ok(Self {
            input_dir: input_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            theme_manager: ThemeManager::new(&themes_dir),
            template_manager: TemplateManager::new(&templates_dir),
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

        // Get theme for this file
        let theme_name = file
            .frontmatter
            .theme
            .clone()
            .or_else(|| self.site_config.site.theme.clone());

        // Get theme variant for this file
        let theme_variant = file
            .frontmatter
            .theme_variant
            .clone()
            .unwrap_or_else(|| "default".to_string());

        // Convert markdown to HTML with semantic structure
        let html_content = if file.frontmatter.list.unwrap_or(false) {
            // For list pages, process content around the blog list placeholder
            let parts: Vec<&str> = file.content.split("<!-- BLOG_POSTS_LIST -->").collect();
            let mut html_parts = Vec::new();

            for (i, part) in parts.iter().enumerate() {
                // Process markdown content before/after blog list
                if !part.trim().is_empty() {
                    let part_html = self.markdown_to_semantic_html(part)?;
                    html_parts.push(part_html);
                }

                // Insert blog list between parts (but not after the last part)
                if i < parts.len() - 1 {
                    let parent_dir = relative_path.parent().unwrap_or_else(|| Path::new(""));
                    let blog_list = self.generate_blog_list_content(parent_dir, list_pages)?;
                    html_parts.push(blog_list);
                }
            }

            html_parts.join("\n")
        } else {
            // For regular pages, process entire content
            self.markdown_to_semantic_html(&file.content)?
        };

        // Generate complete HTML document
        let full_html = if let Some(theme_name) = &theme_name {
            self.generate_html_document_with_template(
                file,
                &html_content,
                theme_name,
                &theme_variant,
            )?
        } else {
            self.generate_html_document_no_theme(&file.title, &html_content)
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

                    // Generate post list entry using template
                    let date = parsed.frontmatter.date.as_deref();
                    let relative_url_path = path
                        .strip_prefix(&self.input_dir)
                        .unwrap_or(&path)
                        .with_extension("");
                    let relative_url = relative_url_path.to_string_lossy();

                    // Extract first paragraph as excerpt
                    let excerpt = if !self.extract_first_paragraph(&parsed.content).is_empty() {
                        let first_paragraph = self.extract_first_paragraph(&parsed.content);
                        let parser = Parser::new(&first_paragraph);
                        let mut excerpt_html = String::new();
                        html::push_html(&mut excerpt_html, parser);
                        Some(excerpt_html)
                    } else {
                        None
                    };

                    let blog_post_html = self.template_manager.render_blog_post(
                        &parsed.title,
                        &relative_url,
                        date,
                        excerpt.as_deref(),
                    )?;

                    list_content.push_str(&blog_post_html);
                    list_content.push_str("\n\n");
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

    fn generate_html_document_with_template(
        &self,
        file: &MarkdownFile,
        content: &str,
        theme_name: &str,
        theme_variant: &str,
    ) -> Result<String> {
        // Note: template_name is used for future extensibility
        let _template_name = file
            .frontmatter
            .page_template
            .as_ref()
            .or_else(|| {
                self.site_config
                    .templates
                    .as_ref()
                    .and_then(|t| t.page_template.as_ref())
            })
            .map_or(DEFAULT_PAGE_TEMPLATE, |s| s.as_str());

        let css_file = Some(format!("/css/{theme_name}.css", theme_name = theme_name));
        let body_attrs = if theme_variant != "default" {
            format!(r#" data-theme="{}""#, theme_variant)
        } else {
            String::new()
        };

        self.template_manager
            .render_page(&file.title, content, css_file.as_deref(), &body_attrs)
    }

    fn generate_html_document_no_theme(&self, title: &str, content: &str) -> String {
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
