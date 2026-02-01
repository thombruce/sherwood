use crate::config::{SiteConfig, SiteGeneratorConfig, SiteSection, TemplateSection};
use crate::content::parsing::MarkdownFile;
use crate::content::renderer::HtmlRenderer;
use crate::content::universal_parser::UniversalContentParser;
use crate::core::utils::{ensure_directory_exists, ensure_parent_exists};
use crate::partials::BreadcrumbGenerator;
use crate::plugins::PluginRegistry;
use crate::presentation::pages::PageGenerator;
use crate::presentation::styles::StyleManager;
use crate::templates::TemplateManager;
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
const DEFAULT_PAGE_TEMPLATE: &str = "sherwood.stpl";

/// Builder for SiteGenerator that provides a fluent API for configuration
pub struct SiteGeneratorBuilder {
    input_dir: PathBuf,
    output_dir: PathBuf,
    config: SiteGeneratorConfig,
}

impl SiteGeneratorBuilder {
    /// Create a new builder with the specified input and output directories
    pub fn new(input_dir: &Path, output_dir: &Path) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            config: SiteGeneratorConfig::new(),
        }
    }

    /// Set development mode
    pub fn development(mut self, is_development: bool) -> Self {
        self.config.is_development = is_development;
        self
    }

    /// Set plugin registry
    pub fn with_plugins(mut self, plugin_registry: PluginRegistry) -> Self {
        self.config.plugin_registry = Some(plugin_registry);
        self
    }

    /// Set template registry
    pub fn with_templates(mut self, template_registry: crate::templates::TemplateRegistry) -> Self {
        self.config.template_registry = Some(template_registry);
        self
    }

    /// Set optional plugin registry
    pub fn with_optional_plugins(mut self, plugin_registry: Option<PluginRegistry>) -> Self {
        self.config.plugin_registry = plugin_registry;
        self
    }

    /// Set optional template registry
    pub fn with_optional_templates(
        mut self,
        template_registry: Option<crate::templates::TemplateRegistry>,
    ) -> Self {
        self.config.template_registry = template_registry;
        self
    }

    /// Set the complete configuration
    pub fn config(mut self, config: SiteGeneratorConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the SiteGenerator instance
    pub fn build(self) -> Result<SiteGenerator> {
        SiteGenerator::build_with_config(&self.input_dir, &self.output_dir, self.config)
    }
}

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
    /// Create a builder for configuring and building a SiteGenerator
    pub fn builder(input_dir: &Path, output_dir: &Path) -> SiteGeneratorBuilder {
        SiteGeneratorBuilder::new(input_dir, output_dir)
    }

    /// Build a SiteGenerator with the specified configuration
    fn build_with_config(
        input_dir: &Path,
        output_dir: &Path,
        config: SiteGeneratorConfig,
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
                site: SiteSection {
                    title: String::new(), // Will be validated below
                    footer_text: None,
                },
                templates: Some(TemplateSection {
                    page_template: Some(DEFAULT_PAGE_TEMPLATE.to_string()),
                }),
                css: None,
                breadcrumb: None,
            }
        };

        // Validate required site configuration
        if site_config.site.title.is_empty() {
            return Err(anyhow::anyhow!(
                "Site title is required but not found in configuration.\n\
                 Please add a [site] section with a 'title' field to your Sherwood.toml:\n\
                 [site]\n\
                 title = \"Your Site Title\"\n\
                 footer_text = \"Optional footer text\""
            ));
        }

        let template_manager =
            TemplateManager::new_with_registry(&templates_dir, config.template_registry)?;
        let html_renderer = HtmlRenderer::new(input_dir, template_manager.clone());

        // Create breadcrumb generator if configured
        let breadcrumb_generator = site_config
            .breadcrumb
            .as_ref()
            .map(|config| BreadcrumbGenerator::new(input_dir, Some(config.clone())));

        let page_generator = PageGenerator::new_with_breadcrumb(
            template_manager,
            breadcrumb_generator,
            site_config.site.clone(),
        );

        // Create style manager based on mode and configuration
        let style_manager = StyleManager::new_with_config(
            &styles_dir,
            site_config.css.as_ref(),
            config.is_development,
        );

        // Create content parser with optional plugins
        let content_parser = UniversalContentParser::new(config.plugin_registry);

        Ok(Self {
            input_dir: input_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            style_manager,
            html_renderer,
            page_generator,
            content_parser,
            site_config,
            is_development: config.is_development,
        })
    }

    // Legacy constructors for backward compatibility
    pub fn new(input_dir: &Path, output_dir: &Path) -> Result<Self> {
        Self::builder(input_dir, output_dir).build()
    }

    pub fn new_development(input_dir: &Path, output_dir: &Path) -> Result<Self> {
        Self::builder(input_dir, output_dir)
            .development(true)
            .build()
    }

    pub fn new_with_plugins(
        input_dir: &Path,
        output_dir: &Path,
        plugin_registry: PluginRegistry,
    ) -> Result<Self> {
        Self::builder(input_dir, output_dir)
            .with_plugins(plugin_registry)
            .build()
    }

    pub fn new_development_with_plugins(
        input_dir: &Path,
        output_dir: &Path,
        plugin_registry: PluginRegistry,
    ) -> Result<Self> {
        Self::builder(input_dir, output_dir)
            .development(true)
            .with_plugins(plugin_registry)
            .build()
    }

    pub fn new_with_plugins_and_templates(
        input_dir: &Path,
        output_dir: &Path,
        plugin_registry: Option<PluginRegistry>,
        template_registry: Option<crate::templates::TemplateRegistry>,
    ) -> Result<Self> {
        Self::builder(input_dir, output_dir)
            .with_optional_plugins(plugin_registry)
            .with_optional_templates(template_registry)
            .build()
    }

    pub fn new_development_with_plugins_and_templates(
        input_dir: &Path,
        output_dir: &Path,
        plugin_registry: Option<PluginRegistry>,
        template_registry: Option<crate::templates::TemplateRegistry>,
    ) -> Result<Self> {
        Self::builder(input_dir, output_dir)
            .development(true)
            .with_optional_plugins(plugin_registry)
            .with_optional_templates(template_registry)
            .build()
    }

    // Legacy internal constructor - now delegates to build_with_config
    #[allow(dead_code)]
    fn new_with_mode_and_plugins_and_templates(
        input_dir: &Path,
        output_dir: &Path,
        is_development: bool,
        plugin_registry: Option<PluginRegistry>,
        template_registry: Option<crate::templates::TemplateRegistry>,
    ) -> Result<Self> {
        let config = SiteGeneratorConfig {
            is_development,
            plugin_registry,
            template_registry,
        };
        Self::build_with_config(input_dir, output_dir, config)
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
                .process_markdown_file(file, &html_content, list_data)?
        } else {
            self.page_generator
                .process_markdown_file(file, &html_content, None)?
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

/// Generate a site using the specified configuration
///
/// This is the unified function that replaces all previous generate_site variants
pub async fn generate_site_with_config(
    input_dir: &Path,
    output_dir: &Path,
    config: SiteGeneratorConfig,
) -> Result<()> {
    let generator = SiteGenerator::build_with_config(input_dir, output_dir, config)?;
    generator.generate().await
}

// Legacy functions for backward compatibility
#[deprecated(
    since = "0.6.0",
    note = "Use generate_site_with_config with SiteGeneratorConfig instead"
)]
pub async fn generate_site(input_dir: &Path, output_dir: &Path) -> Result<()> {
    generate_site_with_config(input_dir, output_dir, SiteGeneratorConfig::new()).await
}

#[deprecated(
    since = "0.6.0",
    note = "Use generate_site_with_config with SiteGeneratorConfig::development() instead"
)]
pub async fn generate_site_development(input_dir: &Path, output_dir: &Path) -> Result<()> {
    generate_site_with_config(input_dir, output_dir, SiteGeneratorConfig::development()).await
}

#[deprecated(
    since = "0.6.0",
    note = "Use generate_site_with_config with SiteGeneratorConfig::new().with_optional_plugins() instead"
)]
pub async fn generate_site_with_plugins(
    input_dir: &Path,
    output_dir: &Path,
    plugin_registry: Option<PluginRegistry>,
) -> Result<()> {
    let config = SiteGeneratorConfig::new().with_optional_plugins(plugin_registry);
    generate_site_with_config(input_dir, output_dir, config).await
}

#[deprecated(
    since = "0.6.0",
    note = "Use generate_site_with_config with SiteGeneratorConfig::new().with_optional_plugins().with_optional_templates() instead"
)]
pub async fn generate_site_with_plugins_and_templates(
    input_dir: &Path,
    output_dir: &Path,
    plugin_registry: Option<PluginRegistry>,
    template_registry: Option<crate::templates::TemplateRegistry>,
) -> Result<()> {
    let config = SiteGeneratorConfig::new()
        .with_optional_plugins(plugin_registry)
        .with_optional_templates(template_registry);
    generate_site_with_config(input_dir, output_dir, config).await
}

#[deprecated(
    since = "0.6.0",
    note = "Use generate_site_with_config with SiteGeneratorConfig::development().with_optional_plugins() instead"
)]
pub async fn generate_site_development_with_plugins(
    input_dir: &Path,
    output_dir: &Path,
    plugin_registry: Option<PluginRegistry>,
) -> Result<()> {
    let config = SiteGeneratorConfig::development().with_optional_plugins(plugin_registry);
    generate_site_with_config(input_dir, output_dir, config).await
}

#[deprecated(
    since = "0.6.0",
    note = "Use generate_site_with_config with SiteGeneratorConfig::development().with_optional_plugins().with_optional_templates() instead"
)]
pub async fn generate_site_development_with_plugins_and_templates(
    input_dir: &Path,
    output_dir: &Path,
    plugin_registry: Option<PluginRegistry>,
    template_registry: Option<crate::templates::TemplateRegistry>,
) -> Result<()> {
    let config = SiteGeneratorConfig::development()
        .with_optional_plugins(plugin_registry)
        .with_optional_templates(template_registry);
    generate_site_with_config(input_dir, output_dir, config).await
}
