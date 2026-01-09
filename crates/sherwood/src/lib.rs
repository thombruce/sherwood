pub mod config;
pub mod content;
pub mod content_generation;
pub mod core;
pub mod generator;
pub mod plugins;
pub mod presentation;
pub mod server;
mod sherwood;
pub mod templates;

pub use config::{SiteConfig, SiteSection, TemplateSection};
pub use generator::{
    generate_site, generate_site_development, generate_site_development_with_plugins,
    generate_site_with_plugins,
};
pub use plugins::{ContentParser, ParsedContent, PluginRegistry};
pub use presentation::styles::StyleManager;
pub use server::{run_dev_server, run_dev_server_with_plugins};
pub use sherwood::Sherwood;
pub use templates::TemplateManager;
pub use templates::{partials, partials::ContentItem};
