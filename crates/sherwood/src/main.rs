use clap::{Parser, Subcommand};
use sherwood::{ServerConfig, SiteGeneratorConfig};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sherwood")]
#[command(about = "A static site generator for Markdown content")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[arg(short, long, default_value = "content")]
        input: PathBuf,
        #[arg(short, long, default_value = "dist")]
        output: PathBuf,
    },
    Dev {
        #[arg(short, long, default_value = "content")]
        input: PathBuf,
        #[arg(short, long, default_value = "dist")]
        output: PathBuf,
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { input, output } => {
            let config = SiteGeneratorConfig::new();
            if let Err(e) = sherwood::generate_site_with_config(&input, &output, config).await {
                eprintln!("Error generating site: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Dev {
            input,
            output,
            port,
        } => {
            let config = ServerConfig::with_port(port);
            if let Err(e) = sherwood::run_dev_server_with_config(&input, &output, config).await {
                eprintln!("Error running dev server: {}", e);
                std::process::exit(1);
            }
        }
    }
}
