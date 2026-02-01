pub mod config;
pub mod content;
pub mod core;
pub mod generator;
pub mod plugins;
pub mod presentation;
pub mod server;
pub mod templates;

pub use config::{ServerConfig, SiteConfig, SiteGeneratorConfig, SiteSection, TemplateSection};
pub use core::{content_generation, sherwood};
pub use generator::{SiteGenerator, generate_site_with_config};
pub use plugins::{ContentParser, ParsedContent, PluginRegistry};
pub use presentation::styles::StyleManager;
pub use server::run_dev_server_with_config;
pub use templates::TemplateManager;
pub use templates::{TemplateRegistry, partials, partials::ContentItem};

pub use sherwood::Sherwood;

// Re-export the generator builder for convenience
pub use generator::SiteGeneratorBuilder;
