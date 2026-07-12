//! A small, opinionated static site generator: Markdown content in, HTML site
//! out, with pretty URLs, built-in navigation context, and pluggable content
//! parsers.
//!
//! Sherwood is dual-delivery — the same crate ships:
//!
//! - **A binary.** `cargo install sherwood`, then `sherwood build` /
//!   `sherwood serve` over a `content/` directory of Markdown files with YAML
//!   (`---`) or TOML (`+++`) frontmatter. `serve` watches the content tree and
//!   live-reloads the browser on change.
//! - **A library.** Depend on the crate, bring your own templates, and drive
//!   [`build_site`] with a render closure — or wrap your renderer in
//!   `run_cli` (feature `cli`) for a ready-made `build`/`serve` CLI.
//!
//! # Pipeline
//!
//! [`build_site`] walks the content tree and dispatches each file to the
//! [`ContentParser`] registered for its extension in a [`ParserRegistry`];
//! files no parser claims are copied through verbatim as static assets. Every
//! page then gets a [`PageContext`] (nav, breadcrumbs, prev/next, the full
//! page corpus) and is rendered by your closure to a pretty URL:
//! `content/blog/post.md` → `_site/blog/post/index.html`, served as
//! `/blog/post/`.
//!
//! ```no_run
//! use sherwood::{BuildError, ParserRegistry, SiteConfig, build_site};
//!
//! fn main() -> Result<(), BuildError> {
//!     let config = SiteConfig::new()
//!         .with_content_dir("content")
//!         .with_output_dir("_site");
//!     build_site(
//!         &config,
//!         &ParserRegistry::default(),
//!         // Any templating you like; return the final HTML for one page.
//!         |page, _ctx| Ok(format!("<h1>{}</h1>{}", page.frontmatter.title, page.content_html)),
//!         |page| println!("built {}", page.url),
//!     )
//! }
//! ```
//!
//! # Cargo features
//!
//! Both are enabled by default; the `sherwood` binary needs both.
//!
//! - `cli` — the clap `build`/`serve` CLI (`run_cli`, `try_run_cli`, `Asset`)
//!   with a file-watching, live-reloading dev server.
//! - `default-template` — the bundled Sailfish template and stylesheet
//!   (`render_page`, `DEFAULT_STYLE`).
//!
//! With `default-features = false` the headless core remains: [`build_site`],
//! the parser API, and the nav types — no clap, tokio, axum, or sailfish in
//! your dependency tree.

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
pub use cli::{Asset, CliError, run_cli, try_run_cli, try_run_cli_from};

#[cfg(feature = "default-template")]
pub use default_template::{DEFAULT_STYLE, render_page};
