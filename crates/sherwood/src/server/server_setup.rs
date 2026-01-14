//! Server utilities and shared setup functions

use crate::plugins::PluginRegistry;
use crate::{
    generate_site_development, generate_site_development_with_plugins,
    generate_site_development_with_plugins_and_templates,
};
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

/// Create server setup for regular development server
pub async fn create_server_setup(
    input_dir: &Path,
    output_dir: &Path,
    port: u16,
) -> Result<ServerSetup> {
    println!("Generating site in development mode...");
    generate_site_development(input_dir, output_dir).await?;

    let output_dir_buf = output_dir.to_path_buf();
    let fallback_handler = move |uri: Uri| async move {
        super::server_handlers::handle_fallback(uri, output_dir_buf.clone()).await
    };

    let app = Router::new()
        .fallback(fallback_handler)
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    let addr = format!("127.0.0.1:{}", port);
    println!("Development server running at http://{}", addr);

    Ok(ServerSetup { addr, app })
}

/// Create server setup for plugin-enabled development server
pub async fn create_server_setup_with_plugins(
    input_dir: &Path,
    output_dir: &Path,
    port: u16,
    plugin_registry: Option<PluginRegistry>,
) -> Result<ServerSetup> {
    println!("Generating site in development mode...");
    generate_site_development_with_plugins(input_dir, output_dir, plugin_registry).await?;

    let output_dir_buf = output_dir.to_path_buf();
    let fallback_handler = move |uri: Uri| async move {
        super::server_handlers::handle_fallback(uri, output_dir_buf.clone()).await
    };

    let app = Router::new()
        .fallback(fallback_handler)
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    let addr = format!("127.0.0.1:{}", port);
    println!("Development server running at http://{}", addr);

    Ok(ServerSetup { addr, app })
}

/// Create server setup for plugin and template-enabled development server
pub async fn create_server_setup_with_plugins_and_templates(
    input_dir: &Path,
    output_dir: &Path,
    port: u16,
    plugin_registry: Option<PluginRegistry>,
    template_registry: Option<crate::templates::TemplateRegistry>,
) -> Result<ServerSetup> {
    println!("Generating site in development mode...");
    generate_site_development_with_plugins_and_templates(
        input_dir,
        output_dir,
        plugin_registry,
        template_registry,
    )
    .await?;

    let output_dir_buf = output_dir.to_path_buf();
    let fallback_handler = move |uri: Uri| async move {
        super::server_handlers::handle_fallback(uri, output_dir_buf.clone()).await
    };

    let app = Router::new()
        .fallback(fallback_handler)
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    let addr = format!("127.0.0.1:{}", port);
    println!("Development server running at http://{}", addr);

    Ok(ServerSetup { addr, app })
}

/// Start the server with the given setup
pub async fn start_server(setup: ServerSetup) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(setup.addr).await?;
    axum::serve(listener, setup.app).await?;
    Ok(())
}
