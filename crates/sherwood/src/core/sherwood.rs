use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::{ServerConfig, SiteGeneratorConfig};
use crate::plugins::PluginRegistry;
use crate::templates::TemplateRegistry;

/// A configurable CLI for Sherwood static site generator
#[derive(Parser)]
#[command(about = "A static site generator for Markdown content")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,

    /// Input directory containing Markdown files
    #[arg(short = 'i', long = "input", default_value = "content", global = true)]
    input: PathBuf,

    /// Output directory for generated site
    #[arg(short = 'o', long = "output", default_value = "dist", global = true)]
    output: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a static site from Markdown content
    Generate,
    /// Start a development server for a Sherwood static site
    Dev {
        /// Port for development server
        #[arg(short = 'p', long = "port", default_value = "3000")]
        port: u16,
    },
}

/// A configurable CLI for Sherwood static site generator
pub struct Sherwood {
    plugin_registry: Option<PluginRegistry>,
    template_registry: Option<TemplateRegistry>,
}

impl Default for Sherwood {
    fn default() -> Self {
        Self::new()
    }
}

impl Sherwood {
    /// Create a new Sherwood CLI
    pub fn new() -> Self {
        Self {
            plugin_registry: None,
            template_registry: None,
        }
    }

    /// Add custom content parsers to the CLI
    pub fn with_plugins(mut self, registry: PluginRegistry) -> Self {
        self.plugin_registry = Some(registry);
        self
    }

    /// Add custom template renderers to the CLI
    pub fn with_templates(mut self, registry: TemplateRegistry) -> Self {
        self.template_registry = Some(registry);
        self
    }

    /// Run the CLI and handle the parsed command
    pub async fn run(self) -> Result<()> {
        // Parse command line arguments using clap
        let args = CliArgs::parse();

        match args.command {
            Commands::Generate => {
                let config = SiteGeneratorConfig::new()
                    .with_optional_plugins(self.plugin_registry)
                    .with_optional_templates(self.template_registry);
                crate::generate_site_with_config(&args.input, &args.output, config).await
            }
            Commands::Dev { port } => {
                let site_config = SiteGeneratorConfig::development()
                    .with_optional_plugins(self.plugin_registry)
                    .with_optional_templates(self.template_registry);
                let server_config = ServerConfig::with_port(port).site_config(site_config);
                crate::run_dev_server_with_config(&args.input, &args.output, server_config).await
            }
        }
    }
}
