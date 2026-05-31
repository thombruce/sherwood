use crate::{Breadcrumb, BuildError, NavItem, Page, PageContext};
use sailfish::TemplateSimple;

pub const DEFAULT_STYLE: &str = include_str!("../templates/style.css");

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

pub fn render_page(page: &Page, ctx: &PageContext) -> Result<String, BuildError> {
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
