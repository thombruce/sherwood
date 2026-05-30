use sailfish::TemplateSimple;
use sherwood::{Breadcrumb, BuildError, NavItem, Page, PageContext};

pub const DEFAULT_STYLE: &str = include_str!("../templates/style.css");

#[derive(TemplateSimple)]
#[template(path = "page.stpl")]
struct PageTemplate {
    title: String,
    content: String,
    nav: Vec<NavItem>,
    breadcrumbs: Vec<Breadcrumb>,
    prev: Option<NavItem>,
    next: Option<NavItem>,
}

pub fn render_page(page: &Page, ctx: &PageContext) -> Result<String, BuildError> {
    PageTemplate {
        title: page.frontmatter.title.clone(),
        content: page.content_html.clone(),
        nav: ctx.nav.clone(),
        breadcrumbs: ctx.breadcrumbs.clone(),
        prev: ctx.prev.clone(),
        next: ctx.next.clone(),
    }
    .render_once()
    .map_err(|e| BuildError::Render(e.to_string()))
}
