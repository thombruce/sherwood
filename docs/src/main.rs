#[tokio::main]
async fn main() {
    let cli = sherwood::SherwoodCli::new(
        "ssg",
        "A static site generator for Markdown content"
    );

    if let Err(e) = cli.run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
