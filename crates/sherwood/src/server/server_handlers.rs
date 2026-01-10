//! Server request handlers

use axum::{
    http::Uri,
    response::{Html, IntoResponse, Response},
};
use std::path::{Path, PathBuf};

/// Handle fallback requests for static files
pub async fn handle_fallback(uri: Uri, output_dir: PathBuf) -> impl IntoResponse {
    let path = uri.path();

    if let Some(file_path) = resolve_file_path(path, &output_dir) {
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                let content_type = determine_content_type(&file_path);
                create_file_response(content_type, content)
            }
            Err(_) => create_404_response(path),
        }
    } else {
        create_404_response(path)
    }
}

/// Resolve the actual file path for a given URL path
fn resolve_file_path(path: &str, output_dir: &Path) -> Option<PathBuf> {
    let possible_paths = generate_possible_paths(path);

    for clean_path in possible_paths {
        let file_path = output_dir.join(clean_path.strip_prefix('/').unwrap_or(&clean_path));
        if file_path.exists() {
            return Some(file_path);
        }
    }

    None
}

/// Generate possible file paths to try for a given URL path
fn generate_possible_paths(path: &str) -> Vec<String> {
    if path == "/" {
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
    }
}

/// Determine content type based on file extension
fn determine_content_type(file_path: &Path) -> &'static str {
    if let Some(extension) = file_path.extension() {
        match extension.to_str().unwrap_or("") {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "xml" => "application/xml",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "pdf" => "application/pdf",
            _ => "text/plain",
        }
    } else {
        "text/plain"
    }
}

/// Create a successful file response
fn create_file_response(content_type: &'static str, content: String) -> Response {
    Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .body(content.into())
        .unwrap()
}

/// Create a 404 response page
pub fn create_404_response(path: &str) -> Response {
    let html = render_404_template(path);
    (axum::http::StatusCode::NOT_FOUND, Html(html)).into_response()
}

/// Render the 404 error page template
fn render_404_template(path: &str) -> String {
    // Use the embedded template if available, otherwise fall back to the inline template
    match get_embedded_404_template() {
        Some(template) => template.replace("{{path}}", path),
        None => create_fallback_404_template(path),
    }
}

/// Get the embedded 404 template
fn get_embedded_404_template() -> Option<String> {
    // In a real implementation, this would load from the embedded templates
    // For now, return None to use the fallback template
    None
}

/// Create a fallback 404 template when embedded template is not available
fn create_fallback_404_template(path: &str) -> String {
    format!(
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
            color: #e53e3e;
        }}
        p {{
            margin-bottom: 1rem;
        }}
        code {{
            background-color: #f7fafc;
            padding: 0.2rem 0.4rem;
            border-radius: 0.25rem;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-size: 0.9rem;
        }}
        a {{
            color: #3182ce;
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
    </style>
</head>
<body>
    <h1>Page Not Found</h1>
    <p>The page you're looking for doesn't exist.</p>
    <p>Requested path: <code>{}</code></p>
    <p><a href="/">Go back to the homepage</a></p>
</body>
</html>"#,
        path
    )
}
