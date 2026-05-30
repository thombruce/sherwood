# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                          # compile
cargo test                           # run all tests
cargo test frontmatter               # run tests in a specific module
cargo run -- build                   # build site: content/ → _site/
cargo run -- build --content-dir src --output-dir out  # custom dirs
cargo run -- serve                   # dev server at http://127.0.0.1:4000
cargo run -- serve --port 4001       # custom port
```

## Architecture

Sherwood is a dual-delivery crate: a **library** (`src/lib.rs`) and a **binary** (`src/main.rs`) built from the same package.

### Library (public API)

The library exposes the build pipeline without any template dependency. Advanced users add `sherwood` as a crate dependency, define their own Sailfish templates, and call `build_site` with a render closure:

```rust
sherwood::build_site(&config, |page, ctx| {
    MyTemplate {
        title: page.frontmatter.title.clone(),
        nav: ctx.nav.clone(),
        // ...
    }.render_once()
})
```

Public API surface: `SiteConfig`, `FrontMatter`, `Page`, `PageContext`, `NavItem`, `Breadcrumb`, `build_site`, `BuildError`.

### Binary (standalone)

`src/main.rs` declares two binary-only modules — `mod serve` and `mod templates` — which are not re-exported by the library. `src/templates.rs` owns the baked-in Sailfish template (`templates/page.stpl`) compiled into the binary at build time.

### Build pipeline flow

Two-pass pipeline — all pages collected and sorted before any rendering begins:

```
Pass 1 — collect:
  content/**/*.md
    └─ load_page()         [page.rs]         read file → parse frontmatter + markdown → Page

Pass 2 — sort + render:
  pages.sort_by(output_path)
  for each page:
    └─ nav::compute_context()  [nav.rs]       build PageContext (nav, breadcrumbs, prev, next)
    └─ renderer closure()      [caller]       PageTemplate { ... }.render_once() → HTML string
    └─ write_page()            [build.rs]     create dirs, write _site/path/to/file.html
```

Output paths mirror source structure: `content/blog/post.md` → `_site/blog/post.html`.

### Key constraints

**Sailfish templates must use `#[derive(TemplateSimple)]`**, not `TemplateOnce`. `TemplateSimple` destructures struct fields into local variables so templates can reference them as bare identifiers (`<%= title %>`). `TemplateOnce` does not destructure and will produce "cannot find value in scope" compile errors.

**`gray_matter` TOML support** requires `features = ["toml"]` in `Cargo.toml` (not enabled by default). TOML delimiter is `+++`; a separate `Matter::<TOML>` instance with `matter.delimiter = "+++".to_owned()` is required since gray_matter defaults all engines to `---`.

**axum 0.8 static files**: use `Router::fallback_service(ServeDir::new(...))` — `nest_service("/", ...)` panics at runtime.
