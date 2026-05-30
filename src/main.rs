mod serve;
mod templates;

use clap::{Parser, Subcommand};
use sherwood::{SiteConfig, build_site};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sherwood", version, about = "A static site generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the site from content/ to _site/
    Build {
        #[arg(long, default_value = "content")]
        content_dir: PathBuf,
        #[arg(long, default_value = "_site")]
        output_dir: PathBuf,
    },
    /// Serve _site/ on a local dev server
    Serve {
        #[arg(long, default_value = "_site")]
        output_dir: PathBuf,
        #[arg(long, default_value_t = 4000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build { content_dir, output_dir } => {
            let config = SiteConfig { content_dir, output_dir };
            let result = build_site(&config, templates::render_page, |page| {
                println!("{} -> {}", page.source_path.display(), page.output_path.display());
            });
            match result {
                Ok(()) => println!("Build complete."),
                Err(e) => {
                    eprintln!("Build failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Serve { output_dir, port } => {
            if let Err(e) = serve::serve(&output_dir, port).await {
                eprintln!("Serve failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}
