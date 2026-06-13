//! The Sherwood site — Sherwood building its own marketing/documentation site.
//!
//! This binary is a downstream consumer of the `sherwood` library: it defines
//! its own Sailfish template and stylesheet and drives the build through
//! `run_cli`, exactly as the README tells third-party users to. If anything
//! here is awkward, that's API feedback.
//!
//! Usage (from the repo root):
//!   cargo run -p sherwood-site -- build --content-dir site/content --output-dir site/_site
//!   cargo run -p sherwood-site -- serve --content-dir site/content --output-dir site/_site

use std::process::ExitCode;

use sailfish::TemplateSimple;
use sherwood::{
    Asset, Breadcrumb, BuildError, NavItem, Page, PageContext, ParserRegistry, run_cli,
};

/// Bundled stylesheet, embedded at compile time and written to the output as
/// `style.css` after the build.
const STYLE: &str = include_str!("../assets/style.css");

#[derive(TemplateSimple)]
#[template(path = "page.stpl")]
struct PageTemplate<'a> {
    title: &'a str,
    content: &'a str,
    nav: &'a [NavItem],
    breadcrumbs: &'a [Breadcrumb],
    prev: Option<&'a NavItem>,
    next: Option<&'a NavItem>,
}

/// The render closure handed to `build_site` via `run_cli`. Maps each page +
/// its computed context onto the site's own template.
fn render(page: &Page, ctx: &PageContext) -> Result<String, BuildError> {
    PageTemplate {
        title: &page.frontmatter.title,
        content: &page.content_html,
        nav: &ctx.nav,
        breadcrumbs: &ctx.breadcrumbs,
        prev: ctx.prev.as_ref(),
        next: ctx.next.as_ref(),
    }
    .render_once()
    .map_err(|e| BuildError::Render(e.to_string()))
}

fn main() -> ExitCode {
    run_cli(
        ParserRegistry::default(),
        render,
        vec![Asset::new("style.css", STYLE.as_bytes())],
    )
}
