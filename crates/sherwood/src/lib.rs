pub mod config;
pub mod generator;
pub mod html_renderer;
pub mod markdown_parser;
pub mod page_generator;
pub mod server;
pub mod templates;
pub mod themes;
pub mod utils;

pub use config::{SiteConfig, SiteSection, TemplateSection};
pub use generator::generate_site;
pub use server::run_dev_server;
pub use templates::{TemplateManager, validate_templates};
pub use themes::{Theme, ThemeManager};
