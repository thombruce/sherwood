mod server_handlers;
mod server_setup;

pub use server_handlers::create_404_response;
pub use server_setup::{create_server_setup_with_config, start_server};

use crate::config::ServerConfig;
use anyhow::Result;
use std::path::Path;

/// Run development server using the specified configuration
///
/// This is the unified function that replaces all previous run_dev_server variants
pub async fn run_dev_server_with_config(
    input_dir: &Path,
    output_dir: &Path,
    config: ServerConfig,
) -> Result<()> {
    let setup =
        server_setup::create_server_setup_with_config(input_dir, output_dir, config).await?;
    start_server(setup).await
}

// Legacy functions for backward compatibility
#[deprecated(
    since = "0.6.0",
    note = "Use run_dev_server_with_config with ServerConfig::with_port() instead"
)]
pub async fn run_dev_server(input_dir: &Path, output_dir: &Path, port: u16) -> Result<()> {
    let config = ServerConfig::with_port(port);
    run_dev_server_with_config(input_dir, output_dir, config).await
}

#[deprecated(
    since = "0.6.0",
    note = "Use run_dev_server_with_config with ServerConfig::with_port().with_optional_plugins() instead"
)]
pub async fn run_dev_server_with_plugins(
    input_dir: &Path,
    output_dir: &Path,
    port: u16,
    plugin_registry: Option<crate::plugins::PluginRegistry>,
) -> Result<()> {
    let config = ServerConfig::with_port(port).with_optional_plugins(plugin_registry);
    run_dev_server_with_config(input_dir, output_dir, config).await
}

#[deprecated(
    since = "0.6.0",
    note = "Use run_dev_server_with_config with ServerConfig::with_port().with_optional_plugins().with_optional_templates() instead"
)]
pub async fn run_dev_server_with_plugins_and_templates(
    input_dir: &Path,
    output_dir: &Path,
    port: u16,
    plugin_registry: Option<crate::plugins::PluginRegistry>,
    template_registry: Option<crate::templates::TemplateRegistry>,
) -> Result<()> {
    let config = ServerConfig::with_port(port)
        .with_optional_plugins(plugin_registry)
        .with_optional_templates(template_registry);
    run_dev_server_with_config(input_dir, output_dir, config).await
}
