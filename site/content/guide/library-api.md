---
title: Using the Library
---

# Using the Library

Add Sherwood as a dependency without the bundled template — you'll bring your
own:

```toml
[dependencies]
sherwood = { version = "0.5", default-features = false, features = ["cli"] }
sailfish = "0.11"
```

Define a template and a render closure, then hand it to `run_cli`:

```rust
use sailfish::TemplateSimple;
use sherwood::{Asset, BuildError, NavItem, Page, PageContext, ParserRegistry, run_cli};

#[derive(TemplateSimple)]
#[template(path = "page.stpl")]
struct PageTemplate<'a> {
    title: &'a str,
    content: &'a str,
    nav: &'a [NavItem],
}

fn render(page: &Page, ctx: &PageContext) -> Result<String, BuildError> {
    PageTemplate {
        title: &page.frontmatter.title,
        content: &page.content_html,
        nav: &ctx.nav,
    }
    .render_once()
    .map_err(|e| BuildError::Render(e.to_string()))
}

fn main() -> std::process::ExitCode {
    run_cli(
        ParserRegistry::default(),
        render,
        vec![Asset::new("style.css", include_str!("../assets/style.css").as_bytes())],
    )
}
```

That's the entire `site/` binary behind this very site.

## The render closure

`build_site` and `run_cli` accept any `FnMut(&Page, &PageContext) -> Result<String, BuildError>`.
Each call gets:

- **`Page`** — frontmatter (`title` plus arbitrary fields via `frontmatter.get`),
  rendered `content_html`, optional `excerpt_html`, and the source/output paths.
- **`PageContext`** — the computed `nav`, `breadcrumbs`, `prev` / `next`
  neighbours, and the full `pages` slice (use `ctx.pages_under("/blog/")` to
  drive section indexes).

## Templates must use `TemplateSimple`

Sailfish's `TemplateSimple` derive destructures struct fields into locals, so
the template can reference `<%= title %>` directly. `TemplateOnce` does not, and
will fail to compile.
