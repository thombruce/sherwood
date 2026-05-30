# Sherwood

A static site generator written in Rust. Converts a directory of Markdown files into a functional HTML website.

## Quick Start

```bash
cargo install sherwood
```

Create a content directory with Markdown files:

```
my-site/
└── content/
    ├── index.md
    ├── about.md
    └── blog/
        └── first-post.md
```

Build and serve:

```bash
sherwood build        # generates _site/
sherwood serve        # http://127.0.0.1:4000
```

## Content Format

Files are Markdown with either YAML or TOML frontmatter.

**YAML** (`---` delimiters):

```markdown
---
title: My Page
---

# My Page

Content goes here.
```

**TOML** (`+++` delimiters):

```toml
+++
title = "My Page"
+++

Content goes here.
```

The `title` field is required.

## Output Structure

Output mirrors the source structure under `_site/`:

```
content/index.md          →  _site/index.html
content/about.md          →  _site/about.html
content/blog/first.md     →  _site/blog/first.html
```

## Built-in Navigation

Every page includes:

- **Global nav bar** — links to all pages; current page marked with `aria-current="page"`
- **Breadcrumbs** — directory hierarchy (hidden on root page)
- **Prev / Next links** — sequential navigation between pages (root `index.html` first, then alphabetical by output path)

## Custom Options

```bash
sherwood build --content-dir src --output-dir dist
sherwood serve --port 8080
```

## Library Usage

For custom templates, add `sherwood` as a dependency and call `build_site` with your own renderer:

```rust
use sherwood::{SiteConfig, build_site};
use sailfish::TemplateSimple;

#[derive(TemplateSimple)]
#[template(path = "page.stpl")]
struct MyTemplate {
    title: String,
    content: String,
}

fn main() {
    let config = SiteConfig::default();
    build_site(
        &config,
        |page, _ctx| {
            MyTemplate {
                title: page.frontmatter.title.clone(),
                content: page.content_html.clone(),
            }
            .render_once()
            .map_err(|e| sherwood::BuildError::Render(e.to_string()))
        },
        |page| println!("{} -> {}", page.source_path.display(), page.output_path.display()),
    ).unwrap();
}
```

`build_site` takes a renderer `FnMut(&Page, &PageContext) -> Result<String, BuildError>` and a progress callback `FnMut(&Page)` invoked after each page is written. Pass `|_| {}` to silence build logging.

`PageContext` provides `nav`, `breadcrumbs`, `prev`, and `next` for building navigation in custom templates.
