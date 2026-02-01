use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SiteConfig {
    pub site: SiteSection,
    pub templates: Option<TemplateSection>,
    pub css: Option<CssSection>,
    pub breadcrumb: Option<BreadcrumbSection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteSection {
    pub title: String,
    pub footer_text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateSection {
    pub page_template: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CssSection {
    pub minify: Option<bool>,
    pub targets: Option<CssTargets>,
    pub source_maps: Option<bool>,
    pub remove_unused: Option<bool>,
    pub nesting: Option<bool>,
    pub entry_point: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CssTargets {
    pub chrome: Option<String>,
    pub firefox: Option<String>,
    pub safari: Option<String>,
    pub edge: Option<String>,
    pub browserslist: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BreadcrumbSection {
    pub max_items: Option<usize>,
    pub enabled: Option<bool>,
}

/// Configuration for site generation that replaces multiple function parameters
#[derive(Default)]
pub struct SiteGeneratorConfig {
    /// Whether to run in development mode (affects CSS minification, etc.)
    pub is_development: bool,
    /// Optional plugin registry for custom content parsers
    pub plugin_registry: Option<crate::plugins::PluginRegistry>,
    /// Optional template registry for custom templates
    pub template_registry: Option<crate::templates::TemplateRegistry>,
}

impl SiteGeneratorConfig {
    /// Create a new configuration with default values (production mode)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new configuration for development mode
    pub fn development() -> Self {
        Self {
            is_development: true,
            ..Default::default()
        }
    }

    /// Set the development mode
    pub fn with_development(mut self, is_development: bool) -> Self {
        self.is_development = is_development;
        self
    }

    /// Set the plugin registry
    pub fn with_plugins(mut self, plugin_registry: crate::plugins::PluginRegistry) -> Self {
        self.plugin_registry = Some(plugin_registry);
        self
    }

    /// Set the template registry
    pub fn with_templates(mut self, template_registry: crate::templates::TemplateRegistry) -> Self {
        self.template_registry = Some(template_registry);
        self
    }

    /// Set optional plugin registry
    pub fn with_optional_plugins(
        mut self,
        plugin_registry: Option<crate::plugins::PluginRegistry>,
    ) -> Self {
        self.plugin_registry = plugin_registry;
        self
    }

    /// Set optional template registry
    pub fn with_optional_templates(
        mut self,
        template_registry: Option<crate::templates::TemplateRegistry>,
    ) -> Self {
        self.template_registry = template_registry;
        self
    }
}

/// Configuration for server setup that encompasses all server options
pub struct ServerConfig {
    /// Port number for the development server
    pub port: u16,
    /// Site generation configuration
    pub site_config: SiteGeneratorConfig,
}

impl ServerConfig {
    /// Create a new server configuration with default port 3000
    pub fn new() -> Self {
        Self {
            port: 3000,
            site_config: SiteGeneratorConfig::development(),
        }
    }

    /// Create a new server configuration with specified port
    pub fn with_port(port: u16) -> Self {
        Self {
            port,
            site_config: SiteGeneratorConfig::development(),
        }
    }

    /// Set the port number
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the site generation configuration
    pub fn site_config(mut self, site_config: SiteGeneratorConfig) -> Self {
        self.site_config = site_config;
        self
    }

    /// Set the development mode for site generation
    pub fn development(mut self, is_development: bool) -> Self {
        self.site_config.is_development = is_development;
        self
    }

    /// Set the plugin registry
    pub fn with_plugins(mut self, plugin_registry: crate::plugins::PluginRegistry) -> Self {
        self.site_config.plugin_registry = Some(plugin_registry);
        self
    }

    /// Set the template registry
    pub fn with_templates(mut self, template_registry: crate::templates::TemplateRegistry) -> Self {
        self.site_config.template_registry = Some(template_registry);
        self
    }

    /// Set optional plugin registry
    pub fn with_optional_plugins(
        mut self,
        plugin_registry: Option<crate::plugins::PluginRegistry>,
    ) -> Self {
        self.site_config.plugin_registry = plugin_registry;
        self
    }

    /// Set optional template registry
    pub fn with_optional_templates(
        mut self,
        template_registry: Option<crate::templates::TemplateRegistry>,
    ) -> Self {
        self.site_config.template_registry = template_registry;
        self
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}
