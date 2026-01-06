use crate::config::{SiteConfig, SiteSection, TemplateSection};
use crate::content::parser::MarkdownFile;
use crate::content::renderer::HtmlRenderer;
use crate::content::universal_parser::UniversalContentParser;
use crate::core::utils::{ensure_directory_exists, ensure_parent_exists};
use crate::partials::BreadcrumbGenerator;
use crate::plugins::PluginRegistry;
use crate::presentation::pages::PageGenerator;
use crate::presentation::styles::StyleManager;
use crate::presentation::templates::TemplateManager;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

// Constants for relative path construction
const STYLES_DIR_RELATIVE: &str = "../styles";
const TEMPLATES_DIR_RELATIVE: &str = "../templates";
const CONFIG_PATH_RELATIVE: &str = "../Sherwood.toml";

// Constants for template names
const DEFAULT_PAGE_TEMPLATE: &str = "default.stpl";

pub struct SiteGenerator {
    input_dir: PathBuf,
    output_dir: PathBuf,
    style_manager: StyleManager,
    html_renderer: HtmlRenderer,
    page_generator: PageGenerator,
    content_parser: UniversalContentParser,
    #[allow(dead_code)]
    site_config: SiteConfig,
    #[allow(dead_code)]
    is_development: bool,
}

impl SiteGenerator {
    pub fn new(input_dir: &Path, output_dir: &Path) -> Result<Self> {
        Self::new_with_mode(input_dir, output_dir, false)
    }

    pub fn new_development(input_dir: &Path, output_dir: &Path) -> Result<Self> {
        Self::new_with_mode(input_dir, output_dir, true)
    }

    pub fn new_with_plugins(
        input_dir: &Path,
        output_dir: &Path,
        plugin_registry: PluginRegistry,
    ) -> Result<Self> {
        Self::new_with_mode_and_plugins(input_dir, output_dir, false, Some(plugin_registry))
    }

    pub fn new_development_with_plugins(
        input_dir: &Path,
        output_dir: &Path,
        plugin_registry: PluginRegistry,
    ) -> Result<Self> {
        Self::new_with_mode_and_plugins(input_dir, output_dir, true, Some(plugin_registry))
    }

    fn new_with_mode(input_dir: &Path, output_dir: &Path, is_development: bool) -> Result<Self> {
        Self::new_with_mode_and_plugins(input_dir, output_dir, is_development, None)
    }

    fn new_with_mode_and_plugins(
        input_dir: &Path,
        output_dir: &Path,
        is_development: bool,
        plugin_registry: Option<PluginRegistry>,
    ) -> Result<Self> {
        let styles_dir = input_dir.join(STYLES_DIR_RELATIVE);
        let templates_dir = input_dir.join(TEMPLATES_DIR_RELATIVE);

        // Load site configuration
        let config_path = input_dir.join(CONFIG_PATH_RELATIVE);
        let site_config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            SiteConfig {
                site: SiteSection {},
                templates: Some(TemplateSection {
                    page_template: Some(DEFAULT_PAGE_TEMPLATE.to_string()),
                }),
                css: None,
                breadcrumb: None,
            }
        };

        let template_manager = TemplateManager::new(&templates_dir)?;
        let html_renderer = HtmlRenderer::new(input_dir, template_manager.clone());

        // Create breadcrumb generator if configured
        let breadcrumb_generator = site_config
            .breadcrumb
            .as_ref()
            .map(|config| BreadcrumbGenerator::new(input_dir, Some(config.clone())));

        let page_generator =
            PageGenerator::new_with_breadcrumb(template_manager, breadcrumb_generator);

        // Create style manager based on mode and configuration
        let style_manager =
            StyleManager::new_with_config(&styles_dir, site_config.css.as_ref(), is_development);

        // Create content parser with optional plugins
        let content_parser = UniversalContentParser::new(plugin_registry);

        Ok(Self {
            input_dir: input_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            style_manager,
            html_renderer,
            page_generator,
            content_parser,
            site_config,
            is_development,
        })
    }

    pub async fn generate(&self) -> Result<()> {
        // Clean output directory
        if self.output_dir.exists() {
            fs::remove_dir_all(&self.output_dir)?;
        }
        ensure_directory_exists(&self.output_dir)?;

        // Generate CSS
        self.generate_css()?;

        // Find all content files
        let content_files = self.find_content_files(&self.input_dir)?;

        if content_files.is_empty() {
            println!("No content files found in {}", self.input_dir.display());
            return Ok(());
        }

        // Parse all content files to extract metadata
        let mut parsed_files = Vec::new();
        for file_path in content_files {
            let parsed = self.content_parser.parse_file(&file_path)?;
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

    fn find_content_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut dirs_to_visit = Vec::new();
        let supported_extensions = self.content_parser.supported_extensions();

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
                    && supported_extensions.contains(&extension.to_string_lossy().to_string())
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

        // Process content intelligently (HTML passes through, markdown gets converted)
        let html_content = self.html_renderer.process_content(&file.content)?;

        // Generate complete HTML document
        let full_html = if file.frontmatter.list.unwrap_or(false) {
            let parent_dir = relative_path.parent().unwrap_or_else(|| Path::new(""));
            let list_data = Some(
                self.html_renderer
                    .generate_list_data(parent_dir, list_pages)?,
            );

            self.page_generator
                .process_markdown_file_with_list(file, &html_content, list_data)?
        } else {
            self.page_generator
                .process_markdown_file(file, &html_content)?
        };

        fs::write(&html_path, full_html)?;
        println!("Generated: {}", html_path.display());

        Ok(())
    }

    fn generate_css(&self) -> Result<()> {
        let css_path = self
            .style_manager
            .generate_css_file(&self.output_dir, self.site_config.css.as_ref())?;
        println!("Generated CSS: {}", css_path.display());
        Ok(())
    }
}

pub async fn generate_site(input_dir: &Path, output_dir: &Path) -> Result<()> {
    let generator = SiteGenerator::new(input_dir, output_dir)?;
    generator.generate().await
}

pub async fn generate_site_development(input_dir: &Path, output_dir: &Path) -> Result<()> {
    let generator = SiteGenerator::new_development(input_dir, output_dir)?;
    generator.generate().await
}

pub async fn generate_site_with_plugins(
    input_dir: &Path,
    output_dir: &Path,
    plugin_registry: Option<PluginRegistry>,
) -> Result<()> {
    let generator = if let Some(registry) = plugin_registry {
        SiteGenerator::new_with_plugins(input_dir, output_dir, registry)?
    } else {
        SiteGenerator::new(input_dir, output_dir)?
    };
    generator.generate().await
}

pub async fn generate_site_development_with_plugins(
    input_dir: &Path,
    output_dir: &Path,
    plugin_registry: Option<PluginRegistry>,
) -> Result<()> {
    let generator = if let Some(registry) = plugin_registry {
        SiteGenerator::new_development_with_plugins(input_dir, output_dir, registry)?
    } else {
        SiteGenerator::new_development(input_dir, output_dir)?
    };
    generator.generate().await
}
