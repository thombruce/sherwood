use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ssg")]
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
            if let Err(e) = sherwood::generate_site(&input, &output).await {
                eprintln!("Error generating site: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Dev {
            input,
            output,
            port,
        } => {
            if let Err(e) = sherwood::run_dev_server(&input, &output, port).await {
                eprintln!("Error running dev server: {}", e);
                std::process::exit(1);
            }
        }
    }
}
