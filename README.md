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

Each page (other than `index.md` files) is wrapped in a directory so that the dev server and most static hosts serve clean URLs without a `.html` suffix:

```
content/index.md          →  _site/index.html               →  /
content/about.md          →  _site/about/index.html         →  /about/
content/blog/index.md     →  _site/blog/index.html          →  /blog/
content/blog/first.md     →  _site/blog/first/index.html    →  /blog/first/
```

`index.md` files (root or section) stay flat as `<dir>/index.html`. Any other page is wrapped so it gets a directory-style URL.

## Built-in Navigation

Every page includes:

- **Global nav bar** — links to all pages; current page marked with `aria-current="page"`
- **Breadcrumbs** — directory hierarchy (hidden on root page)
- **Prev / Next links** — sequential navigation between pages (root `index.html` first, then alphabetical by output path)

## Styling

The binary ships a minimal default stylesheet (CSS reset, readable typography, nav/breadcrumb baseline) embedded at compile time. Every build writes it once to `<output_dir>/style.css` and links it from each page.

Override with your own CSS:

```bash
sherwood build --asset style.css=my.css
```

`--asset name=path` is a generic override: it replaces any bundled asset whose destination matches `name`, or adds a new asset if no match is found. May be repeated.

## Custom Options

```bash
sherwood build --content-dir src --output-dir dist
sherwood build --asset style.css=my.css
sherwood serve --port 8080
```

## Library Usage

Sherwood ships in two layers. Most projects want the high-level CLI helper; advanced users can call `build_site` directly.

### High-level: `run_cli`

`run_cli` parses a clap CLI (`build` / `serve`), runs `build_site` with your renderer, writes any assets you pass, and exits with an appropriate code. It owns its own tokio runtime, so you don't need `#[tokio::main]`.

```toml
[dependencies]
sherwood = { version = "0.2", default-features = false, features = ["cli"] }
sailfish = "0.11"
```

```rust
use std::process::ExitCode;

use sailfish::TemplateSimple;
use sherwood::{Asset, Breadcrumb, BuildError, NavItem, Page, PageContext, run_cli};

#[derive(TemplateSimple)]
#[template(path = "page.stpl")]
struct MyTemplate {
    title: String,
    content: String,
    nav: Vec<NavItem>,
    breadcrumbs: Vec<Breadcrumb>,
    prev: Option<NavItem>,
    next: Option<NavItem>,
}

fn render(page: &Page, ctx: &PageContext) -> Result<String, BuildError> {
    MyTemplate {
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

fn main() -> ExitCode {
    run_cli(
        render,
        vec![Asset::new("style.css", include_bytes!("../assets/style.css").as_slice())],
    )
}
```

`Asset::new` takes any `Into<Cow<'static, [u8]>>`, so compile-time `include_bytes!`, a `&'static str` slice, or a runtime `Vec<u8>` all work.

Use `try_run_cli` instead of `run_cli` if you want a `Result<(), CliError>` rather than process exit.

### Low-level: `build_site`

If you want full control — non-CLI driver, custom asset pipeline, embedded use — call `build_site` directly. Disable all default features to keep clap/axum/tokio/sailfish out of your build:

```toml
[dependencies]
sherwood = { version = "0.2", default-features = false }
```

```rust
use sherwood::{SiteConfig, build_site};

fn main() {
    let config = SiteConfig::default();
    build_site(
        &config,
        |page, _ctx| Ok(format!("<h1>{}</h1>{}", page.frontmatter.title, page.content_html)),
        |page| println!("{} -> {}", page.source_path.display(), page.output_path.display()),
    )
    .unwrap();
}
```

`build_site` takes a renderer `FnMut(&Page, &PageContext) -> Result<String, BuildError>` and a progress callback `FnMut(&Page)` invoked after each page is written. Pass `|_| {}` to silence build logging. `PageContext` provides `nav`, `breadcrumbs`, `prev`, and `next` for building navigation.

## Cargo features

| Feature | Default | Pulls in | Enables |
|---|---|---|---|
| `cli` | ✅ | clap, tokio, axum, tower-http | `run_cli`, `try_run_cli`, `Asset`, `serve` module |
| `default-template` | ✅ | sailfish | `render_page`, `DEFAULT_STYLE` (the bundled theme) |

Both features are required to build the `sherwood` binary. Library users can disable either or both:

- `default-features = false` — headless, just `build_site`. No clap/axum/tokio/sailfish.
- `default-features = false, features = ["cli"]` — CLI helper without the bundled Sailfish template (bring your own). This is what `sherwood-demo` uses.
