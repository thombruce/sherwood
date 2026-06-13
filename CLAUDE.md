# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                          # compile
cargo test                           # run all tests
cargo test frontmatter               # run tests in a specific module
cargo run -- build                   # build site: content/ → _site/
cargo run -- build --content-dir src --output-dir out  # custom dirs
cargo run -- build --asset style.css=my.css  # override a bundled asset from disk
cargo run -- serve                   # dev server at http://127.0.0.1:4000
cargo run -- serve --port 4001       # custom port
cargo run -- serve --no-watch        # static server, no file-watch/live-reload
```

The crate has two features, both on by default: `cli` (clap/axum/tokio dev server) and `default-template` (the bundled Sailfish template + stylesheet). The `sherwood` binary requires both. Library-only consumers can disable them with `default-features = false`.

## Architecture

Sherwood is a dual-delivery crate: a **library** (`src/lib.rs`) and a **binary** (`src/main.rs`) built from the same package.

### Library (public API)

The core library (the always-on part, no features) exposes the build pipeline without any template dependency. Advanced users add `sherwood` as a crate dependency, define their own Sailfish templates, and call `build_site` with a render closure and a progress callback:

```rust
sherwood::build_site(
    &config,
    |page, ctx| {
        MyTemplate {
            title: page.frontmatter.title.clone(),
            nav: ctx.nav.clone(),
            // ...
        }.render_once().map_err(|e| BuildError::Render(e.to_string()))
    },
    |page| println!("{} -> {}", page.source_path.display(), page.output_path.display()),
)
```

The render closure is `FnMut(&Page, &PageContext) -> Result<String, BuildError>`. The progress callback is `FnMut(&Page)` and is invoked after each page is written. Pass `|_| {}` to silence build logging.

Core public API surface: `SiteConfig`, `FrontMatter`, `Page`, `PageContext`, `NavItem`, `Breadcrumb`, `build_site`, `Pod` (re-exported from `gray_matter`).

**Error types are layered per module, each owning its own enum:** `FrontmatterError` (frontmatter.rs — `MissingDelimiters` / `Invalid(String)`, no file path) → wrapped by `PageError` (page.rs — `Read` / `Frontmatter`, both carrying the source `PathBuf`) → wrapped by `BuildError` (build.rs — `Io` / `Walk` / `Page(#[from] PageError)` / `Render`). Lower modules never import a higher module's error; `?` bubbles up through `#[from]`. `BuildError::Page` is `#[error(transparent)]`, so the displayed message is the `PageError` chain (path + frontmatter snippet) with no extra prefix.

### Feature modules

The bundled template and CLI live behind cargo features and are re-exported from `src/lib.rs` (not binary-only):

- **`default-template`** → `src/default_template.rs`. Owns the baked-in Sailfish template (`templates/page.stpl`, compiled at build time) and embeds `templates/style.css` via `include_str!` as `DEFAULT_STYLE`. Public exports: `render_page` (the ready-made render closure) and `DEFAULT_STYLE`. The core library does not ship or prescribe a stylesheet — pure-library users embed their own CSS in their downstream binary.
- **`cli`** → `src/cli.rs` (+ `src/serve.rs` dev server). Owns the clap arg parsing and the `run_cli` / `try_run_cli` entry points. Public exports: `run_cli`, `try_run_cli`, `Asset`, `CliError`.

`src/main.rs` is a thin shim: it calls `run_cli(render_page, vec![Asset::new("style.css", DEFAULT_STYLE.as_bytes())])`. Assets are written to `<output_dir>` after `build_site`. `--asset <name>=<path>` overrides a bundled asset (matched by its `dest`) with a file from disk; the flag is repeatable.

### Build pipeline flow

Two-pass pipeline — all pages collected and sorted before any rendering begins:

```
Pass 1 — collect:
  content/**/*.md
    └─ load_page()         [page.rs]         read file → parse frontmatter + markdown → Page

Pass 2 — sort + render:
  pages.sort_by(root index first, then output_path)
  for each page:
    └─ nav::compute_context()  [nav/]         build PageContext (nav, breadcrumbs, prev, next)
    └─ renderer closure()      [caller]       PageTemplate { ... }.render_once() → HTML string
    └─ write_page()            [build.rs]     create dirs, write _site/path/to/file.html
    └─ progress callback()     [caller]       optional per-page hook (e.g. CLI logging)
```

Sort key is `(!is_root_index, output_path)` — keeps the root `index.html` at the front of the nav rather than buried after alphabetical siblings.

Output paths mirror source structure but use pretty URLs — each page becomes a `<dir>/index.html` so it serves at a trailing-slash URL: `content/blog/post.md` → `_site/blog/post/index.html` (served at `/blog/post/`). The root `content/index.md` and any `content/<dir>/index.md` section index map straight to `<dir>/index.html`.

### nav module layout

`src/nav/` is a directory module split by concern: `mod.rs` (`PageContext`, `NavItem`, `compute_context`, nav-inclusion rules, `is_root_index`), `url.rs` (`href_for` / `path_to_url` URL building), and `breadcrumb.rs` (`Breadcrumb` + breadcrumb trail). Shared test fixtures live in `src/nav/test_support.rs` (`#[cfg(test)]` only). Cross-crate callers reach the helpers through the re-exports in `mod.rs` (`nav::href_for`, `nav::is_root_index`).

### Key constraints

**Sailfish templates must use `#[derive(TemplateSimple)]`**, not `TemplateOnce`. `TemplateSimple` destructures struct fields into local variables so templates can reference them as bare identifiers (`<%= title %>`). `TemplateOnce` does not destructure and will produce "cannot find value in scope" compile errors.

**`gray_matter` TOML support** requires `features = ["toml"]` in `Cargo.toml` (not enabled by default). TOML delimiter is `+++`; a separate `Matter::<TOML>` instance with `matter.delimiter = "+++".to_owned()` is required since gray_matter defaults all engines to `---`.

**axum 0.8 static files**: use `Router::fallback_service(ServeDir::new(...))` — `nest_service("/", ...)` panics at runtime.

**`SiteConfig` is `#[non_exhaustive]`** so fields can be added without a breaking change. In-crate code may still use struct-literal construction; downstream library users cannot — they build via `SiteConfig::new()` / `default()` plus the `with_content_dir` / `with_output_dir` builder methods. Add a matching `with_*` method (and a default) for any new field.

**URL building from `Path`**: do not use `Path::display()` when constructing href strings. On Windows it emits `\` separators, producing invalid URLs like `/blog\post.html`. `path_to_url` (in `src/nav/url.rs`) walks `Component::Normal` and joins with `/` — use it for any new URL output.
