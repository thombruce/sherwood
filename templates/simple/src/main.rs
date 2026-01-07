#[tokio::main]
async fn main() {
    let cli = sherwood::Sherwood::with_defaults();

    if let Err(e) = cli.run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
