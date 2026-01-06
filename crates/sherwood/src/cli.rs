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
}

/// A configurable CLI for Sherwood static site generator
pub struct SherwoodCli {
    name: String,
    about: String,
    plugin_registry: Option<PluginRegistry>,
}

impl SherwoodCli {
    /// Create a new Sherwood CLI with custom name and description
    pub fn new(name: &str, about: &str) -> Self {
        Self {
            name: name.to_string(),
            about: about.to_string(),
            plugin_registry: None,
        }
    }

    /// Create a new Sherwood CLI with default name and description
    pub fn with_defaults() -> Self {
        Self::new("sherwood", "A static site generator for Markdown content")
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

        // Print custom name and help if needed (maintain compatibility with custom CLI names)
        if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
            println!("{} {}", self.name, env!("CARGO_PKG_VERSION"));
            println!("{}", self.about);
            println!();
            println!("Usage: {} [COMMAND] [OPTIONS]", self.name);
            println!();
            println!("Commands:");
            println!("  generate    Generate a static site from Markdown content");
            println!("  dev         Start a development server for a Sherwood static site");
            println!();
            println!("Options:");
            println!(
                "  -i, --input <DIR>     Input directory containing Markdown files [default: content]"
            );
            println!("  -o, --output <DIR>    Output directory for generated site [default: dist]");
            println!("  -p, --port <PORT>     Port for development server [default: 3000]");
            println!("  -h, --help            Print help");
            println!();
            println!("Examples:");
            println!("  {} generate", self.name);
            println!("  {} generate -i content -o dist", self.name);
            println!("  {} dev -p 8080", self.name);
            return Ok(());
        }

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
