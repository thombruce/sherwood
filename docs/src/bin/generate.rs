use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "generate")]
#[command(about = "A static site generator for Markdown content")]
struct Cli {
    #[arg(short, long, default_value = "content")]
    input: PathBuf,
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = sherwood::generate_site(&cli.input, &cli.output).await {
        eprintln!("Error generating site: {}", e);
        std::process::exit(1);
    }
}
