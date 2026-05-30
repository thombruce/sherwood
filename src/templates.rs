use sailfish::TemplateSimple;
use sherwood::{Page, BuildError};

#[derive(TemplateSimple)]
#[template(path = "page.stpl")]
struct PageTemplate {
    title: String,
    content: String,
}

pub fn render_page(page: &Page) -> Result<String, BuildError> {
    PageTemplate {
        title: page.frontmatter.title.clone(),
        content: page.content_html.clone(),
    }
    .render_once()
    .map_err(|e| BuildError::Render(e.to_string()))
}
