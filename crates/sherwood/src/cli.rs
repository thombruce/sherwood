use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::plugins::PluginRegistry;

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
    // TODO: Re-add validate. How useful is it to us to validate templates?
    // Alternatively: Address dead code.
}

/// A configurable CLI for Sherwood static site generator
pub struct SherwoodCli {
    plugin_registry: Option<PluginRegistry>,
}

impl Default for SherwoodCli {
    fn default() -> Self {
        Self::new()
    }
}

impl SherwoodCli {
    /// Create a new Sherwood CLI
    pub fn new() -> Self {
        Self {
            plugin_registry: None,
        }
    }

    /// Add custom content parsers to the CLI
    pub fn with_plugins(mut self, registry: PluginRegistry) -> Self {
        self.plugin_registry = Some(registry);
        self
    }

    /// Run the CLI and handle the parsed command
    pub async fn run(self) -> Result<()> {
        // Parse command line arguments using clap
        let args = CliArgs::parse();

        match args.command {
            Commands::Generate => {
                crate::generate_site_with_plugins(&args.input, &args.output, self.plugin_registry)
                    .await
            }
            Commands::Dev { port } => {
                crate::run_dev_server_with_plugins(
                    &args.input,
                    &args.output,
                    port,
                    self.plugin_registry,
                )
                .await
            }
        }
    }
}
