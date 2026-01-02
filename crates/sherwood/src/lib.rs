pub mod components;
pub mod generator;
pub mod server;

pub use components::config::{SiteConfig, SiteSection, TemplateSection};
pub use components::templates::{TemplateManager, validate_templates};
pub use components::themes::{Theme, ThemeManager};
pub use generator::generate_site;
pub use server::run_dev_server;
