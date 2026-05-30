use std::path::Path;
use axum::Router;
use thiserror::Error;
use tower_http::services::ServeDir;

#[derive(Debug, Error)]
pub enum ServeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub async fn serve(output_dir: &Path, port: u16) -> Result<(), ServeError> {
    let app = Router::new().fallback_service(ServeDir::new(output_dir));
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Serving {} at http://{}", output_dir.display(), addr);
    axum::serve(listener, app).await?;
    Ok(())
}
