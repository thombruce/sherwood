use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{Request, Response, header},
    middleware::{self, Next},
    response::{IntoResponse, Redirect},
    routing::get,
};
use thiserror::Error;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

use crate::core::build::BuildError;

#[derive(Debug, Error)]
pub enum ServeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Initial build failed: {0}")]
    Build(BuildError),
    #[error("Watcher error: {0}")]
    Watcher(String),
}

const LIVE_RELOAD_PATH: &str = "/_sherwood/reload";

const LIVE_RELOAD_SNIPPET: &str = "\n<script>\n(function(){function c(){var p=location.protocol==='https:'?'wss':'ws';var w=new WebSocket(p+'://'+location.host+'/_sherwood/reload');w.onmessage=function(){location.reload();};w.onclose=function(){setTimeout(c,1000);};}c();})();\n</script>\n";

/// Build a router for static-only serving (no live reload).
pub fn router(output_dir: &Path, base_path: &str) -> Router {
    mount(Router::new(), output_dir, base_path)
}

/// Build a router with live-reload wiring: a `/_sherwood/reload` websocket
/// endpoint that pushes `reload` messages on the broadcast channel, plus a
/// middleware that injects a tiny script into every served HTML response so
/// the browser connects to that socket.
pub fn router_with_reload(
    output_dir: &Path,
    reload_tx: broadcast::Sender<()>,
    base_path: &str,
) -> Router {
    let state = Arc::new(reload_tx);
    let router = Router::new().route(LIVE_RELOAD_PATH, get(ws_handler).with_state(state));
    mount(router, output_dir, base_path).layer(middleware::from_fn(inject_reload_script))
}

/// Attach the static-file service to `router`. With an empty `base_path` the
/// site is served at the root; with a base path (e.g. `/sherwood`) the site is
/// mounted under it and `/` redirects there, mirroring production hosting on a
/// subpath. The live-reload websocket route stays at the root either way.
fn mount(router: Router, output_dir: &Path, base_path: &str) -> Router {
    let serve = ServeDir::new(output_dir);
    if base_path.is_empty() {
        router.fallback_service(serve)
    } else {
        let target = format!("{base_path}/");
        router
            .route(
                "/",
                get(move || {
                    let target = target.clone();
                    async move { Redirect::permanent(&target) }
                }),
            )
            .nest_service(base_path, serve)
    }
}

/// Start the dev server. If `watch` is `Some`, also watches `content_dir`,
/// reruns `rebuild` on changes, and pushes live-reload notifications.
pub async fn serve_with_watch<F>(
    content_dir: PathBuf,
    output_dir: PathBuf,
    base_path: String,
    port: u16,
    mut rebuild: F,
    watch: bool,
) -> Result<(), ServeError>
where
    F: FnMut() -> Result<(), BuildError> + Send + 'static,
{
    // Initial build before the server comes up. Bail out loudly if it fails
    // — the user's first request would 404 otherwise.
    rebuild().map_err(ServeError::Build)?;

    let app = if watch {
        let (tx, _rx) = broadcast::channel::<()>(16);
        let tx_for_watcher = tx.clone();
        let content_dir_for_watcher = content_dir.clone();
        tokio::task::spawn_blocking(move || {
            watch_loop(content_dir_for_watcher, tx_for_watcher, rebuild);
        });
        router_with_reload(&output_dir, tx, &base_path)
    } else {
        router(&output_dir, &base_path)
    };

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let url = format!("http://{addr}{base_path}/");
    if watch {
        println!(
            "Serving {} at {} (watching {} for changes)",
            output_dir.display(),
            url,
            content_dir.display()
        );
    } else {
        println!("Serving {} at {}", output_dir.display(), url);
    }
    axum::serve(listener, app).await?;
    Ok(())
}

fn watch_loop<F>(content_dir: PathBuf, reload_tx: broadcast::Sender<()>, mut rebuild: F)
where
    F: FnMut() -> Result<(), BuildError> + Send + 'static,
{
    use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};

    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let mut debouncer = match new_debouncer(Duration::from_millis(300), move |res| {
        let _ = event_tx.send(res);
    }) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to start file watcher: {e}");
            return;
        }
    };
    if let Err(e) = debouncer
        .watcher()
        .watch(&content_dir, RecursiveMode::Recursive)
    {
        eprintln!("Failed to watch {}: {e}", content_dir.display());
        return;
    }

    // Snapshot content-file mtimes so we can ignore spurious events.
    // Reading files during a rebuild updates `atime`, which fires `IN_ATTRIB`
    // events on Linux even though the data hasn't changed — without this
    // guard, every rebuild self-triggers another rebuild.
    let mut snapshot = snapshot_mtimes(&content_dir);

    for res in event_rx {
        match res {
            Ok(events) if !events.is_empty() => {
                let current = snapshot_mtimes(&content_dir);
                if current == snapshot {
                    continue;
                }
                eprintln!("Change detected — rebuilding...");
                match rebuild() {
                    Ok(()) => {
                        eprintln!("Rebuild complete.");
                        let _ = reload_tx.send(());
                    }
                    Err(e) => eprintln!("Rebuild failed: {e}"),
                }
                snapshot = snapshot_mtimes(&content_dir);
            }
            Ok(_) => {}
            Err(e) => eprintln!("Watcher error: {e}"),
        }
    }
}

fn snapshot_mtimes(root: &Path) -> std::collections::HashMap<PathBuf, std::time::SystemTime> {
    use walkdir::WalkDir;
    let mut map = std::collections::HashMap::new();
    for entry in WalkDir::new(root).into_iter().flatten() {
        if entry.file_type().is_file()
            && let Ok(meta) = entry.metadata()
            && let Ok(mtime) = meta.modified()
        {
            map.insert(entry.path().to_owned(), mtime);
        }
    }
    map
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(reload_tx): State<Arc<broadcast::Sender<()>>>,
) -> impl IntoResponse {
    let mut rx = reload_tx.subscribe();
    ws.on_upgrade(move |socket| async move {
        handle_socket(socket, &mut rx).await;
    })
}

async fn handle_socket(mut socket: WebSocket, rx: &mut broadcast::Receiver<()>) {
    while rx.recv().await.is_ok() {
        if socket.send(Message::Text("reload".into())).await.is_err() {
            break;
        }
    }
}

async fn inject_reload_script(req: Request<Body>, next: Next) -> Response<Body> {
    let resp = next.run(req).await;
    let (mut parts, body) = resp.into_parts();
    let is_html = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("text/html"))
        .unwrap_or(false);
    if !is_html {
        return Response::from_parts(parts, body);
    }
    let bytes = match to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };
    let mut html = String::from_utf8_lossy(&bytes).into_owned();
    if let Some(pos) = html.rfind("</body>") {
        html.insert_str(pos, LIVE_RELOAD_SNIPPET);
    } else {
        html.push_str(LIVE_RELOAD_SNIPPET);
    }
    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from(html))
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
        let resp = router(tmp.path(), "")
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
        let resp = router(tmp.path(), "")
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
        let resp = router(tmp.path(), "")
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

    #[tokio::test]
    async fn base_path_mounts_under_prefix_and_redirects_root() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("index.html"), "<h1>hi</h1>").unwrap();

        // Served under the base path.
        let resp = router(tmp.path(), "/docs")
            .oneshot(
                Request::builder()
                    .uri("/docs/index.html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Root redirects to the base path.
        let resp = router(tmp.path(), "/docs")
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(resp.headers()[header::LOCATION], "/docs/");

        // The un-prefixed path is no longer served.
        let resp = router(tmp.path(), "/docs")
            .oneshot(
                Request::builder()
                    .uri("/index.html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn reload_router_injects_script_into_html() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("index.html"),
            "<html><body><p>hi</p></body></html>",
        )
        .unwrap();
        let (tx, _rx) = broadcast::channel::<()>(4);
        let resp = router_with_reload(tmp.path(), tx, "")
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
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains("/_sherwood/reload"));
        assert!(body.contains("WebSocket"));
        // Script must come before the closing body tag.
        let body_pos = body.find("</body>").unwrap();
        let script_pos = body.find("WebSocket").unwrap();
        assert!(script_pos < body_pos);
    }

    #[test]
    fn snapshot_mtimes_changes_when_content_changes() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("page.md");
        fs::write(&f, "v1").unwrap();
        let snap1 = snapshot_mtimes(tmp.path());
        // mtime has filesystem-dependent resolution; sleep to ensure tick.
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&f, "v2 with more bytes").unwrap();
        let snap2 = snapshot_mtimes(tmp.path());
        assert_ne!(snap1, snap2, "rewriting a file must change snapshot");
    }

    #[test]
    fn snapshot_mtimes_unchanged_when_file_only_read() {
        // This is the load-bearing assertion for the live-reload watch loop:
        // reading content files during a rebuild must not change the mtime
        // snapshot, otherwise the loop would keep self-triggering.
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("page.md");
        fs::write(&f, "data").unwrap();
        let snap1 = snapshot_mtimes(tmp.path());
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = fs::read_to_string(&f).unwrap();
        let snap2 = snapshot_mtimes(tmp.path());
        assert_eq!(snap1, snap2, "reading a file must not change snapshot");
    }

    #[test]
    fn snapshot_mtimes_detects_added_file() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.md"), "a").unwrap();
        let snap1 = snapshot_mtimes(tmp.path());
        fs::write(tmp.path().join("b.md"), "b").unwrap();
        let snap2 = snapshot_mtimes(tmp.path());
        assert_ne!(snap1, snap2);
    }

    #[tokio::test]
    async fn reload_router_leaves_non_html_alone() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("style.css"), "body{}").unwrap();
        let (tx, _rx) = broadcast::channel::<()>(4);
        let resp = router_with_reload(tmp.path(), tx, "")
            .oneshot(
                Request::builder()
                    .uri("/style.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&bytes[..], b"body{}");
    }
}
