//! Server utilities and shared setup functions

use crate::config::ServerConfig;
use crate::plugins::PluginRegistry;
use anyhow::Result;
use axum::{Router, http::Uri};
use std::path::Path;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

/// Shared server setup logic for both regular and plugin-enabled servers
pub struct ServerSetup {
    pub addr: String,
    pub app: Router,
}

/// Create server setup using the specified configuration
///
/// This is the unified function that replaces all previous create_server_setup variants
pub async fn create_server_setup_with_config(
    input_dir: &Path,
    output_dir: &Path,
    config: ServerConfig,
) -> Result<ServerSetup> {
    println!("Generating site in development mode...");
    crate::generate_site_with_config(input_dir, output_dir, config.site_config).await?;

    let output_dir_buf = output_dir.to_path_buf();
    let fallback_handler = move |uri: Uri| async move {
        super::server_handlers::handle_fallback(uri, output_dir_buf.clone()).await
    };

    let app = Router::new()
        .fallback(fallback_handler)
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    let addr = format!("127.0.0.1:{}", config.port);
    println!("Development server running at http://{}", addr);

    Ok(ServerSetup { addr, app })
}

// Legacy functions for backward compatibility
#[deprecated(
    since = "0.6.0",
    note = "Use create_server_setup_with_config with ServerConfig::with_port() instead"
)]
#[allow(dead_code)]
pub async fn create_server_setup(
    input_dir: &Path,
    output_dir: &Path,
    port: u16,
) -> Result<ServerSetup> {
    let config = ServerConfig::with_port(port);
    create_server_setup_with_config(input_dir, output_dir, config).await
}

#[deprecated(
    since = "0.6.0",
    note = "Use create_server_setup_with_config with ServerConfig::with_port().with_optional_plugins() instead"
)]
#[allow(dead_code)]
pub async fn create_server_setup_with_plugins(
    input_dir: &Path,
    output_dir: &Path,
    port: u16,
    plugin_registry: Option<PluginRegistry>,
) -> Result<ServerSetup> {
    let config = ServerConfig::with_port(port).with_optional_plugins(plugin_registry);
    create_server_setup_with_config(input_dir, output_dir, config).await
}

#[deprecated(
    since = "0.6.0",
    note = "Use create_server_setup_with_config with ServerConfig::with_port().with_optional_plugins().with_optional_templates() instead"
)]
#[allow(dead_code)]
pub async fn create_server_setup_with_plugins_and_templates(
    input_dir: &Path,
    output_dir: &Path,
    port: u16,
    plugin_registry: Option<PluginRegistry>,
    template_registry: Option<crate::templates::TemplateRegistry>,
) -> Result<ServerSetup> {
    let config = ServerConfig::with_port(port)
        .with_optional_plugins(plugin_registry)
        .with_optional_templates(template_registry);
    create_server_setup_with_config(input_dir, output_dir, config).await
}

/// Start the server with the given setup
pub async fn start_server(setup: ServerSetup) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(setup.addr).await?;
    axum::serve(listener, setup.app).await?;
    Ok(())
}
