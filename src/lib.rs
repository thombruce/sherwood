mod core;

#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "default-template")]
mod default_template;

pub use core::build::{BuildError, build_site};
pub use core::config::SiteConfig;
pub use core::content::frontmatter::{FrontMatter, FrontmatterError, split_frontmatter};
pub use core::content::page::{Page, PageError};
pub use core::content::parser::{
    ContentParser, MarkdownParser, Parsed, ParserError, ParserRegistry, markdown_to_html,
};
pub use core::nav::{Breadcrumb, NavItem, PageContext};
pub use gray_matter::Pod;

#[cfg(feature = "cli")]
pub use cli::{Asset, CliError, run_cli, try_run_cli};

#[cfg(feature = "default-template")]
pub use default_template::{DEFAULT_STYLE, render_page};
