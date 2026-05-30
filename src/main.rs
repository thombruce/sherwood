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
        /// Override the bundled stylesheet with a file from disk
        #[arg(long)]
        style: Option<PathBuf>,
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
        Commands::Build { content_dir, output_dir, style } => {
            let css = match &style {
                Some(path) => match std::fs::read_to_string(path) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to read style file {}: {}", path.display(), e);
                        std::process::exit(1);
                    }
                },
                None => templates::DEFAULT_STYLE.to_string(),
            };
            let config = SiteConfig { content_dir, output_dir };
            let result = build_site(&config, templates::render_page, |page| {
                println!("{} -> {}", page.source_path.display(), page.output_path.display());
            });
            match result {
                Ok(()) => {
                    let css_path = config.output_dir.join("style.css");
                    if let Err(e) = std::fs::write(&css_path, &css) {
                        eprintln!("Failed to write {}: {}", css_path.display(), e);
                        std::process::exit(1);
                    }
                    println!("Build complete.");
                }
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
