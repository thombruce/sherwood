use crate::components::config::{SiteConfig, SiteSection, TemplateSection};
use crate::components::html_renderer::HtmlRenderer;
use crate::components::markdown_parser::{MarkdownFile, MarkdownParser};
use crate::components::page_generator::PageGenerator;
use crate::components::templates::TemplateManager;
use crate::components::themes::ThemeManager;
use crate::components::utils::{ensure_directory_exists, ensure_parent_exists};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

// Constants for relative path construction
const THEMES_DIR_RELATIVE: &str = "../themes";
const TEMPLATES_DIR_RELATIVE: &str = "../templates";
const CONFIG_PATH_RELATIVE: &str = "../Sherwood.toml";

// Constants for template names
const DEFAULT_PAGE_TEMPLATE: &str = "default.stpl";

// Constants for file extensions
const MARKDOWN_EXT: &str = "md";
const MARKDOWN_LONG_EXT: &str = "markdown";

pub struct SiteGenerator {
    input_dir: PathBuf,
    output_dir: PathBuf,
    theme_manager: ThemeManager,
    html_renderer: HtmlRenderer,
    page_generator: PageGenerator,
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
                site: SiteSection { theme: None },
                templates: Some(TemplateSection {
                    page_template: Some(DEFAULT_PAGE_TEMPLATE.to_string()),
                }),
            }
        };

        let template_manager = TemplateManager::new(&templates_dir)?;
        let html_renderer = HtmlRenderer::new(input_dir);
        let page_generator = PageGenerator::new(template_manager);

        // Validate all templates during initialization
        let validation_errors = page_generator.template_manager.validate_all_templates()?;
        if !validation_errors.is_empty() {
            eprintln!("⚠️  Template validation warnings detected, but continuing...");
        }

        Ok(Self {
            input_dir: input_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            theme_manager: ThemeManager::new(&themes_dir),
            html_renderer,
            page_generator,
            site_config,
        })
    }

    pub async fn generate(&self) -> Result<()> {
        // Clean output directory
        if self.output_dir.exists() {
            fs::remove_dir_all(&self.output_dir)?;
        }
        ensure_directory_exists(&self.output_dir)?;

        // Generate CSS (uses default theme if none configured)
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
            let parsed = MarkdownParser::parse_markdown_file(&file_path)?;
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
                    && (extension == MARKDOWN_EXT || extension == MARKDOWN_LONG_EXT)
                {
                    files.push(path);
                }
            }
        }

        Ok(files)
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
        let theme_name = self.theme_manager.resolve_theme(
            file.frontmatter.theme.clone(),
            self.site_config.site.theme.clone(),
        );

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
                    let part_html = self.html_renderer.markdown_to_semantic_html(part)?;
                    html_parts.push(part_html);
                }

                // Insert blog list between parts (but not after the last part)
                if i < parts.len() - 1 {
                    let parent_dir = relative_path.parent().unwrap_or_else(|| Path::new(""));
                    let blog_list = self
                        .html_renderer
                        .generate_blog_list_content(parent_dir, list_pages)?;
                    html_parts.push(blog_list);
                }
            }

            html_parts.join("\n")
        } else {
            // For regular pages, process entire content
            self.html_renderer
                .markdown_to_semantic_html(&file.content)?
        };

        // Generate complete HTML document
        let full_html = self.page_generator.process_markdown_file(
            file,
            &html_content,
            &theme_name,
            &theme_variant,
        )?;

        fs::write(&html_path, full_html)?;
        println!("Generated: {}", html_path.display());

        Ok(())
    }

    fn generate_theme_css(&self) -> Result<()> {
        // Use configured theme or fall back to default
        let theme_name = self
            .theme_manager
            .resolve_theme(None, self.site_config.site.theme.clone());

        let theme = self.theme_manager.load_theme(&theme_name)?;
        let css_path = self
            .theme_manager
            .generate_css_file(&theme, &self.output_dir)?;
        println!(
            "Generated CSS: {} (theme: {})",
            css_path.display(),
            theme.name
        );
        Ok(())
    }
}

pub async fn generate_site(input_dir: &Path, output_dir: &Path) -> Result<()> {
    let generator = SiteGenerator::new(input_dir, output_dir)?;
    generator.generate().await
}
