use std::path::Path;
use axum::{
    http::StatusCode,
    response::Html,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
use anyhow::Result;

pub async fn run_dev_server(input_dir: &Path, output_dir: &Path, port: u16) -> Result<()> {
    println!("Generating site...");
    super::generate_site(input_dir, output_dir).await?;
    
    let app = Router::new()
        .nest_service("/", ServeDir::new(output_dir))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .fallback(handle_404);

    let addr = format!("127.0.0.1:{}", port);
    println!("Development server running at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn handle_404() -> (StatusCode, Html<String>) {
    (
        StatusCode::NOT_FOUND,
        Html(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Page Not Found</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            text-align: center;
            color: #333;
        }}
        h1 {{
            font-size: 2rem;
            margin-bottom: 1rem;
        }}
    </style>
</head>
<body>
    <h1>Page Not Found</h1>
    <p>The page you're looking for doesn't exist.</p>
</body>
</html>"#
        )),
    )
}