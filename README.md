# Sherwood

A small, opinionated static site generator written in Rust. Converts a directory of Markdown files into a functional HTML website — with pretty URLs, built-in navigation, file-watch live reload, and a library API for building your own site binary.

Documentation site (built with Sherwood itself): <https://sherwood.thombruce.com>

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
        ├── index.md
        └── first-post.md
```

Build and serve:

```bash
sherwood build        # generates _site/
sherwood serve        # http://127.0.0.1:4000, rebuilds + reloads on change
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

```markdown
+++
title = "My Page"
+++

Content goes here.
```

The `title` field is required; a file missing it fails the build with a line-numbered snippet of the offending frontmatter. Any other fields are allowed and exposed to templates via `FrontMatter::get` / `get_string`.

### Excerpts

An optional `<!-- more -->` delimiter splits a page: everything before it is rendered separately into `Page.excerpt_html` (for post previews on index pages). The full body always renders into `Page.content_html`.

## Output Structure

Each page (other than `index.md` files) is wrapped in a directory so that the dev server and most static hosts serve clean URLs without a `.html` suffix:

```
content/index.md          →  _site/index.html               →  /
content/about.md          →  _site/about/index.html         →  /about/
content/blog/index.md     →  _site/blog/index.html          →  /blog/
content/blog/first.md     →  _site/blog/first/index.html    →  /blog/first/
```

Files with no registered parser (images, downloads, extra CSS, …) are copied verbatim to the mirrored output path: `content/blog/img.png` → `_site/blog/img.png`.

Two sources that would write the same output file (e.g. `content/about.md` and `content/about/index.md`) fail the build with an error naming both, rather than one silently overwriting the other.

## Built-in Navigation

Every page's render context includes:

- **Global nav** — top-level pages and section indexes (`<dir>/index.md`), current page marked with `aria-current="page"`. Deep leaf pages (e.g. individual blog posts) are excluded by default; frontmatter `nav: true` force-includes a page, `nav: false` force-excludes one.
- **Breadcrumbs** — directory hierarchy (hidden on the root page).
- **Prev / Next links** — sequential navigation scoped to the page's section: pages chain to siblings under the same URL parent (a blog post's neighbours are other posts; top-level pages and section indexes chain in the root sequence), in build order (root `index.md` first, then alphabetical by output path).

## Styling

The binary ships a minimal default stylesheet (CSS reset, readable typography, nav/breadcrumb baseline) embedded at compile time. Every build writes it to `<output_dir>/style.css` and links it from each page.

Override with your own CSS:

```bash
sherwood build --asset style.css=my.css
```

`--asset name=path` is a generic override: it replaces any bundled asset whose destination matches `name`, or adds a new asset if no match is found. May be repeated.

## CLI Reference

```bash
sherwood build [--content-dir content] [--output-dir _site]
               [--base-path /prefix] [--asset name=path]...

sherwood serve [--content-dir content] [--output-dir _site] [--port 4000]
               [--base-path /prefix] [--asset name=path]... [--no-watch]
```

`serve` builds first, then serves on `127.0.0.1`. By default it watches the content directory, rebuilds on change, and pushes a live-reload message to the browser over a websocket. `--no-watch` disables watching and serves statically.

### Subpath hosting (`--base-path`)

Generated URLs are absolute (`/about/`, `/style.css`), so by default a site must be served from a domain root. To host under a subpath like `https://user.github.io/project/`, build with `--base-path /project` — every generated href gets the prefix. Output *paths* are unaffected; the host maps the subpath to the artifact root. The dev server mounts the site under the base path too, so the preview matches production.

## Library Usage

Sherwood ships in two layers. Most projects want the high-level CLI helper; advanced users can call `build_site` directly. The [`site/`](site/) directory in this repo is a full downstream example (its own template, stylesheet, and binary).

### High-level: `run_cli`

`run_cli(registry, renderer, assets)` parses the standard `build` / `serve` CLI, runs `build_site` with your renderer, writes your assets, and exits with an appropriate code. It owns its own tokio runtime, so you don't need `#[tokio::main]`.

```toml
[dependencies]
sherwood = { version = "0.9", default-features = false, features = ["cli"] }
sailfish = "0.11"
```

```rust
use std::process::ExitCode;

use sailfish::TemplateSimple;
use sherwood::{
    Asset, Breadcrumb, BuildError, NavItem, Page, PageContext, ParserRegistry, run_cli,
};

#[derive(TemplateSimple)]
#[template(path = "page.stpl")]
struct PageTemplate<'a> {
    title: &'a str,
    content: &'a str,
    nav: &'a [NavItem],
    breadcrumbs: &'a [Breadcrumb],
    prev: Option<&'a NavItem>,
    next: Option<&'a NavItem>,
    base_path: &'a str,
}

fn render(page: &Page, ctx: &PageContext) -> Result<String, BuildError> {
    PageTemplate {
        title: &page.frontmatter.title,
        content: &page.content_html,
        nav: &ctx.nav,
        breadcrumbs: &ctx.breadcrumbs,
        prev: ctx.prev.as_ref(),
        next: ctx.next.as_ref(),
        base_path: &ctx.base_path,
    }
    .render_once()
    .map_err(|e| BuildError::Render(e.to_string()))
}

fn main() -> ExitCode {
    run_cli(
        ParserRegistry::default(),
        render,
        vec![Asset::new(
            "style.css",
            include_bytes!("../assets/style.css").as_slice(),
        )],
    )
}
```

`Asset::new` takes any `Into<Cow<'static, [u8]>>`, so compile-time `include_bytes!`, a `&'static str` slice, or a runtime `Vec<u8>` all work.

Use `try_run_cli` instead of `run_cli` if you want a `Result<(), CliError>` rather than process exit, and `try_run_cli_from(args, ...)` to supply the arguments yourself (e.g. in tests) instead of reading `std::env::args`.

### Low-level: `build_site`

For full control — non-CLI driver, custom asset pipeline, embedded use — call `build_site` directly. Disable all default features to keep clap/axum/tokio/sailfish out of your build:

```toml
[dependencies]
sherwood = { version = "0.9", default-features = false }
```

```rust
use sherwood::{ParserRegistry, SiteConfig, build_site};

fn main() {
    let config = SiteConfig::new()
        .with_content_dir("content")
        .with_output_dir("_site");
    build_site(
        &config,
        &ParserRegistry::default(),
        |page, _ctx| Ok(format!("<h1>{}</h1>{}", page.frontmatter.title, page.content_html)),
        |page| println!("{} -> {}", page.source_path.display(), page.output_path.display()),
    )
    .unwrap();
}
```

`build_site(&config, &registry, renderer, progress)` takes a renderer `FnMut(&Page, &PageContext) -> Result<String, BuildError>` and a progress callback `FnMut(&Page)` invoked after each page is written (pass `|_| {}` to silence).

`PageContext` provides `nav`, `breadcrumbs`, `prev`, `next`, plus:

- `pages` — every page in the site, in build order, for arbitrary filtering and sorting.
- `pages_under("/blog/")` — pages whose URL starts with a prefix; drives section indexes and post listings.
- `base_path` / `resolve(url)` — for prefixing hrefs you build yourself when the site uses `--base-path`.

### Custom content parsers

Parsing is pluggable: implement `ContentParser` for a new format and register it. Parsers claim file extensions and turn one file's source into frontmatter + HTML; formats using the `---`/`+++` convention can reuse `sherwood::split_frontmatter`.

```rust
use std::path::Path;
use std::sync::Arc;

use sherwood::{ContentParser, Parsed, ParserError, ParserRegistry, markdown_to_html, split_frontmatter};

struct WikiParser;

impl ContentParser for WikiParser {
    fn extensions(&self) -> &[&str] {
        &["wiki"]
    }
    fn parse(&self, source: &str, _path: &Path) -> Result<Parsed, ParserError> {
        let (frontmatter, body) = split_frontmatter(source)?;
        Ok(Parsed {
            frontmatter,
            content_html: markdown_to_html(&body), // your format's rendering here
            excerpt_html: None,
        })
    }
}

let mut registry = ParserRegistry::default(); // markdown built in
registry.register(Arc::new(WikiParser));
// pass the registry to run_cli or build_site
```

See the [custom parsers guide](https://sherwood.thombruce.com/guide/custom-parsers/) for a fuller walkthrough.

## Cargo Features

| Feature | Default | Pulls in | Enables |
|---|---|---|---|
| `cli` | ✅ | clap, tokio, axum, tower-http, notify | `run_cli`, `try_run_cli`, `Asset`, `CliError` |
| `default-template` | ✅ | sailfish | `render_page`, `DEFAULT_STYLE` (the bundled theme) |

Both features are required to build the `sherwood` binary. Library users can disable either or both:

- `default-features = false` — headless: `build_site`, the parser API, nav types. No clap/axum/tokio/sailfish.
- `default-features = false, features = ["cli"]` — CLI helper without the bundled Sailfish template (bring your own). This is what [`site/`](site/) uses.

## License

MIT
