pub mod build;
pub mod config;
pub mod frontmatter;
pub mod nav;
pub mod page;
pub mod parser;

#[cfg(feature = "cli")]
pub mod serve;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "default-template")]
pub mod default_template;

pub use build::{BuildError, build_site};
pub use config::SiteConfig;
pub use frontmatter::{FrontMatter, FrontmatterError, split_frontmatter};
pub use gray_matter::Pod;
pub use nav::{Breadcrumb, NavItem, PageContext};
pub use page::{Page, PageError};
pub use parser::{ContentParser, MarkdownParser, Parsed, ParserError, ParserRegistry};

#[cfg(feature = "cli")]
pub use cli::{Asset, CliError, run_cli, try_run_cli};

#[cfg(feature = "default-template")]
pub use default_template::{DEFAULT_STYLE, render_page};
