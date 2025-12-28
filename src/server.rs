use anyhow::Result;
use axum::{
    Router,
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use std::path::Path;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

pub async fn run_dev_server(input_dir: &Path, output_dir: &Path, port: u16) -> Result<()> {
    println!("Generating site...");
    super::generate_site(input_dir, output_dir).await?;

    let app = Router::new()
        .nest_service("/static", ServeDir::new(output_dir))
        .fallback(handle_fallback)
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    let addr = format!("127.0.0.1:{}", port);
    println!("Development server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_fallback(uri: Uri) -> impl IntoResponse {
    let path = uri.path();

    // Handle clean URLs: /about -> /about.html, /about/ -> /about.html
    let clean_path = if path == "/" {
        "/index.html".to_string()
    } else if path.ends_with('/') && path != "/" {
        format!("{}.html", path.trim_end_matches('/'))
    } else if !path.contains('.') {
        format!("{}.html", path)
    } else {
        path.to_string()
    };

    let file_path = Path::new("dist")
        .strip_prefix("/")
        .unwrap_or_else(|_| Path::new("dist"))
        .join(clean_path.strip_prefix('/').unwrap_or(&clean_path));

    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            let content_type = if clean_path.ends_with(".html") {
                "text/html"
            } else {
                "text/plain"
            };

            Response::builder()
                .status(200)
                .header("Content-Type", content_type)
                .body(content.into())
                .unwrap()
        }
        Err(_) => {
            let html = format!(
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
    <p>Requested path: {}</p>
    <p>Tried to serve: {}</p>
</body>
</html>"#,
                path,
                file_path.display()
            );

            (StatusCode::NOT_FOUND, Html(html)).into_response()
        }
    }
}

