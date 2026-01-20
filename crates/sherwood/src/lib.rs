pub mod config;
pub mod content;
pub mod core;
pub mod generator;
pub mod plugins;
pub mod presentation;
pub mod server;
pub mod templates;

pub use config::{SiteConfig, SiteSection, TemplateSection};
pub use core::{content_generation, sherwood};
pub use generator::{
    generate_site, generate_site_development, generate_site_development_with_plugins,
    generate_site_development_with_plugins_and_templates, generate_site_with_plugins,
    generate_site_with_plugins_and_templates,
};
pub use plugins::{ContentParser, ParsedContent, PluginRegistry};
pub use presentation::styles::StyleManager;
pub use server::{
    run_dev_server, run_dev_server_with_plugins, run_dev_server_with_plugins_and_templates,
};
pub use templates::TemplateManager;
pub use templates::{TemplateRegistry, partials, partials::ContentItem};

pub use sherwood::Sherwood;
