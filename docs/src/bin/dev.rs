use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dev")]
#[command(about = "A dev server for Sherwood static sites")]
struct Cli {
    #[arg(short, long, default_value = "content")]
    input: PathBuf,
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = sherwood::run_dev_server(&cli.input, &cli.output, cli.port).await {
        eprintln!("Error running dev server: {}", e);
        std::process::exit(1);
    }
}
