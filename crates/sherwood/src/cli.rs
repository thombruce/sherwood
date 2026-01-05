use anyhow::Result;
use std::path::PathBuf;

/// A configurable CLI for Sherwood static site generator
pub struct SherwoodCli {
    name: String,
    about: String,
}

impl SherwoodCli {
    /// Create a new Sherwood CLI with custom name and description
    pub fn new(name: &str, about: &str) -> Self {
        Self {
            name: name.to_string(),
            about: about.to_string(),
        }
    }

    /// Create a new Sherwood CLI with default name and description
    pub fn with_defaults() -> Self {
        Self::new("sherwood", "A static site generator for Markdown content")
    }

    /// Run the CLI and handle the parsed command
    pub async fn run(self) -> Result<()> {
        // Get command line arguments
        let args: Vec<String> = std::env::args().collect();

        if args.len() < 2 {
            self.print_help();
            return Ok(());
        }

        match args[1].as_str() {
            "generate" => {
                let input = self.get_arg(&args, "-i", "--input", "content");
                let output = self.get_arg(&args, "-o", "--output", "dist");
                let input_path = PathBuf::from(input);
                let output_path = PathBuf::from(output);
                crate::generate_site(&input_path, &output_path).await
            }
            "dev" => {
                let input = self.get_arg(&args, "-i", "--input", "content");
                let output = self.get_arg(&args, "-o", "--output", "dist");
                let port = self.get_arg(&args, "-p", "--port", "3000");
                let input_path = PathBuf::from(input);
                let output_path = PathBuf::from(output);
                let port_num = port
                    .parse::<u16>()
                    .map_err(|_| anyhow::anyhow!("Invalid port number: {}", port))?;
                crate::run_dev_server(&input_path, &output_path, port_num).await
            }
            "--help" | "-h" => {
                self.print_help();
                Ok(())
            }
            cmd => {
                eprintln!("Unknown command: {}", cmd);
                self.print_help();
                std::process::exit(1);
            }
        }
    }

    fn get_arg(&self, args: &[String], short: &str, long: &str, default: &str) -> String {
        for i in 0..args.len() {
            if (args[i] == short || args[i] == long) && i + 1 < args.len() {
                return args[i + 1].clone();
            }
        }
        default.to_string()
    }

    fn print_help(&self) {
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
    }
}
