use anyhow::Result;
use axum::{
    Router,
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use std::path::{Path, PathBuf};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

pub async fn run_dev_server(input_dir: &Path, output_dir: &Path, port: u16) -> Result<()> {
    println!("Generating site...");
    super::generate_site(input_dir, output_dir).await?;

    let output_dir_buf = output_dir.to_path_buf();
    let fallback_handler =
        move |uri: Uri| async move { handle_fallback(uri, output_dir_buf.clone()).await };

    let app = Router::new()
        .fallback(fallback_handler)
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    let addr = format!("127.0.0.1:{}", port);
    println!("Development server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_fallback(uri: Uri, output_dir: PathBuf) -> impl IntoResponse {
    let path = uri.path();

    // Try multiple file path possibilities for robust URL handling
    let possible_paths = if path == "/" {
        vec!["/index.html".to_string()]
    } else if path.ends_with('/') && path != "/" {
        // For /about/, try about/index.html, then about.html
        let base_path = path.trim_end_matches('/');
        vec![
            format!("{}/index.html", base_path),
            format!("{}.html", base_path),
        ]
    } else if !path.contains('.') {
        // For /about, try about.html, then about/index.html
        let base_path = path.trim_start_matches('/');
        vec![
            format!("{}.html", base_path),
            format!("{}/index.html", base_path),
        ]
    } else {
        vec![path.to_string()]
    };

    // Try each possible path until we find one that exists
    let mut found_path = None;
    for clean_path in possible_paths {
        let file_path = output_dir.join(clean_path.strip_prefix('/').unwrap_or(&clean_path));

        if file_path.exists() {
            found_path = Some((clean_path, file_path));
            break;
        }
    }

    if let Some((clean_path, file_path)) = found_path {
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                let content_type = if clean_path.ends_with(".html") {
                    "text/html"
                } else if clean_path.ends_with(".css") {
                    "text/css"
                } else {
                    "text/plain"
                };

                Response::builder()
                    .status(200)
                    .header("Content-Type", content_type)
                    .body(content.into())
                    .unwrap()
            }
            Err(_) => create_404_response(path),
        }
    } else {
        create_404_response(path)
    }
}

fn create_404_response(path: &str) -> Response {
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
</body>
</html>"#,
        path
    );

    (StatusCode::NOT_FOUND, Html(html)).into_response()
}
