use std::path::Path;
use axum::Router;
use thiserror::Error;
use tower_http::services::ServeDir;

#[derive(Debug, Error)]
pub enum ServeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn router(output_dir: &Path) -> Router {
    Router::new().fallback_service(ServeDir::new(output_dir))
}

pub async fn serve(output_dir: &Path, port: u16) -> Result<(), ServeError> {
    let app = router(output_dir);
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Serving {} at http://{}", output_dir.display(), addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use std::fs;
    use tempfile::TempDir;
    use tower::ServiceExt;

    #[tokio::test]
    async fn serves_existing_file() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("index.html"), "<h1>hi</h1>").unwrap();
        let resp = router(tmp.path())
            .oneshot(
                Request::builder()
                    .uri("/index.html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&bytes[..], b"<h1>hi</h1>");
    }

    #[tokio::test]
    async fn serves_nested_file() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("blog")).unwrap();
        fs::write(tmp.path().join("blog/post.html"), "post").unwrap();
        let resp = router(tmp.path())
            .oneshot(
                Request::builder()
                    .uri("/blog/post.html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn returns_404_for_missing() {
        let tmp = TempDir::new().unwrap();
        let resp = router(tmp.path())
            .oneshot(
                Request::builder()
                    .uri("/nope.html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
