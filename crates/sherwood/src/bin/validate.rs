use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "validate")]
#[command(about = "Validate Sherwood templates")]
struct Cli {
    /// Templates directory to validate (defaults to ../templates relative to content)
    #[arg(short, long)]
    templates: Option<PathBuf>,
    /// Show detailed template information
    #[arg(long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = sherwood::validate_templates(&cli.templates, cli.verbose) {
        eprintln!("Error validating templates: {}", e);
        std::process::exit(1);
    }
}