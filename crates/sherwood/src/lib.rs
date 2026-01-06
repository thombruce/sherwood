pub mod cli;
pub mod config;
pub mod content;
pub mod core;
pub mod generator;
pub mod partials;
pub mod plugins;
pub mod presentation;
pub mod server;

pub use cli::SherwoodCli;
pub use config::{SiteConfig, SiteSection, TemplateSection};
pub use generator::{
    generate_site, generate_site_development, generate_site_development_with_plugins,
    generate_site_with_plugins,
};
pub use partials::ContentItem;
pub use plugins::{ContentParser, ParsedContent, PluginRegistry};
pub use presentation::styles::StyleManager;
pub use presentation::templates::TemplateManager;
pub use server::{run_dev_server, run_dev_server_with_plugins};
