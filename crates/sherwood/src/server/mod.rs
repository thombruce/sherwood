mod server_handlers;
mod server_setup;

pub use server_handlers::create_404_response;
pub use server_setup::{create_server_setup, create_server_setup_with_plugins, start_server};

use anyhow::Result;
use std::path::Path;

/// Run development server
pub async fn run_dev_server(input_dir: &Path, output_dir: &Path, port: u16) -> Result<()> {
    let setup = server_setup::create_server_setup(input_dir, output_dir, port).await?;
    start_server(setup).await
}

/// Run development server with plugins
pub async fn run_dev_server_with_plugins(
    input_dir: &Path,
    output_dir: &Path,
    port: u16,
    plugin_registry: Option<crate::plugins::PluginRegistry>,
) -> Result<()> {
    let setup = server_setup::create_server_setup_with_plugins(
        input_dir,
        output_dir,
        port,
        plugin_registry,
    )
    .await?;
    start_server(setup).await
}
