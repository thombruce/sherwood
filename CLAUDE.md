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

### Source layout

`src/lib.rs` is a **facade**: it declares the module tree as private (`mod core;`, feature-gated `mod cli;` / `mod default_template;`) and re-exports the public API via `pub use`. Internal module paths (e.g. `core::content::page`) are *not* part of the public API — only the re-exported items are. Reshape the tree freely behind the facade; just keep the `pub use` list curated.

```
src/
  lib.rs              facade (pub use re-exports only)
  main.rs             binary shim
  core/               always-on pipeline, no features
    mod.rs
    build.rs          build_site orchestration + BuildError
    config.rs         SiteConfig
    content/          file → Page
      mod.rs
      page.rs         load_page, Page, PageError
      frontmatter.rs  split_frontmatter, FrontMatter, FrontmatterError
      parser/         pluggable ContentParser system (growth zone)
        mod.rs        ContentParser, Parsed, ParserError, ParserRegistry
        markdown.rs   built-in MarkdownParser + markdown_to_html
        text.rs       built-in TextParser (.txt → <pre>)
    nav/              Page + siblings → PageContext
      mod.rs, url.rs, breadcrumb.rs, test_support.rs
  default_template.rs feature = "default-template" (single-file render layer)
  cli/                feature = "cli"
    mod.rs            clap args, run_cli / try_run_cli
    serve.rs          dev server + file-watch live reload
```

Grouping follows the two real seams: the **feature gate** (core vs `default-template` vs `cli`) and the **pipeline stage** (load/parse → context → render → deliver). `default_template.rs` stays a single flat file — it's the render layer but doesn't yet warrant a folder; promote it to `render/` when it grows.

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

`build_site(&config, &registry, renderer, progress)`. The render closure is `FnMut(&Page, &PageContext) -> Result<String, BuildError>`. The progress callback is `FnMut(&Page)` and is invoked after each page is written. Pass `|_| {}` to silence build logging.

Core public API surface: `SiteConfig`, `FrontMatter`, `Page`, `PageContext`, `NavItem`, `Breadcrumb`, `build_site`, `Pod` (re-exported from `gray_matter`), plus the parser plugin API below.

**Error types are layered per module, each owning its own enum:** `FrontmatterError` (core/content/frontmatter.rs — `MissingDelimiters` / `Invalid(String)`, no file path) → `ParserError` (core/content/parser/mod.rs — `Frontmatter(#[from] FrontmatterError)` / `Message(String)` / `Other(Box<dyn Error + Send + Sync>)`) → `PageError` (core/content/page.rs — `Read` / `Parse`, both carrying the source `PathBuf`) → `BuildError` (core/build.rs — `Io` / `Walk` / `Page(#[from] PageError)` / `Render`). Lower modules never import a higher module's error; `?` bubbles up through `#[from]`. `BuildError::Page` and `PageError::Parse`'s inner `ParserError::Frontmatter` are `#[error(transparent)]`, so the displayed message is the chain (path + frontmatter snippet) with no extra prefixes.

### Content parsers (plugin system)

Parsing is pluggable. A `ContentParser` (core/content/parser/mod.rs) turns one file's raw source into a `Parsed { frontmatter, content_html, excerpt_html }`; it never computes paths/URLs — that stays in `load_page`. Parsers are `Send + Sync` (the dev server shares the registry across threads) and object-safe (`dyn ContentParser`).

```rust
pub trait ContentParser: Send + Sync {
    fn extensions(&self) -> &[&str];                 // ["md", "markdown"], no dot
    fn parse(&self, source: &str, path: &Path) -> Result<Parsed, ParserError>;
}
```

A `ParserRegistry` maps extension → `Arc<dyn ContentParser>`. Constructors: `default()` / `with_builtins()` register **all** built-ins (markdown + text); `with_markdown()` registers markdown only; `empty()` starts with none. `register(Arc::new(MyParser))` adds one (last registration for an extension wins). The build walks **all** files and skips any whose extension has no registered parser (so images/CSS can live in the content tree) — `load_page` returns `Ok(None)` for those. **To add a built-in format:** new file in `core/content/parser/`, `mod`/`pub use` it, register it in `with_builtins`, add it to the lib.rs facade.

Third-party parsers own their whole file, including their metadata convention. Formats that use the `---`/`+++` convention call the public `split_frontmatter(source) -> Result<(FrontMatter, String), FrontmatterError>` helper; others ignore it (e.g. `TextParser` takes its title from the first line). Parser-API exports: `ContentParser`, `Parsed`, `ParserError`, `ParserRegistry`, `MarkdownParser`, `TextParser`, `markdown_to_html`, `split_frontmatter`.

Built-ins: `MarkdownParser` (core/content/parser/markdown.rs) owns markdown rendering (`markdown_to_html`, `pulldown-cmark`) and the `<!-- more -->` excerpt split — excerpt is a markdown concern, not a frontmatter one, so `split_frontmatter` no longer touches it. `TextParser` (core/content/parser/text.rs) handles `.txt`: first line = title, remaining lines = HTML-escaped `<pre>` body, no frontmatter.

### Feature modules

The bundled template and CLI live behind cargo features and are re-exported from `src/lib.rs` (not binary-only):

- **`default-template`** → `src/default_template.rs`. Owns the baked-in Sailfish template (`templates/page.stpl`, compiled at build time) and embeds `templates/style.css` via `include_str!` as `DEFAULT_STYLE`. Public exports: `render_page` (the ready-made render closure) and `DEFAULT_STYLE`. The core library does not ship or prescribe a stylesheet — pure-library users embed their own CSS in their downstream binary.
- **`cli`** → `src/cli/mod.rs` (+ `src/cli/serve.rs` dev server). Owns the clap arg parsing and the `run_cli` / `try_run_cli` entry points. Public exports: `run_cli`, `try_run_cli`, `Asset`, `CliError`.

`src/main.rs` is a thin shim: it calls `run_cli(ParserRegistry::default(), render_page, vec![Asset::new("style.css", DEFAULT_STYLE.as_bytes())])`. `run_cli` / `try_run_cli` take the registry as their first argument so binary authors can register custom parsers. Assets are written to `<output_dir>` after `build_site`. `--asset <name>=<path>` overrides a bundled asset (matched by its `dest`) with a file from disk; the flag is repeatable.

### Build pipeline flow

Two-pass pipeline — all pages collected and sorted before any rendering begins:

```
Pass 1 — collect:
  content/**/*  (every file)
    └─ load_page()  [core/content/page.rs]
         registry.get(ext)? → read file → parser.parse() → Page
         (returns None — skipped — if no parser claims the extension)

Pass 2 — sort + render:
  pages.sort_by(root index first, then output_path)
  for each page:
    └─ nav::compute_context()  [core/nav/]    build PageContext (nav, breadcrumbs, prev, next)
    └─ renderer closure()      [caller]       PageTemplate { ... }.render_once() → HTML string
    └─ write_page()            [core/build.rs] create dirs, write _site/<dir>/index.html
    └─ progress callback()     [caller]       optional per-page hook (e.g. CLI logging)
```

Sort key is `(!is_root_index, output_path)` — keeps the root `index.html` at the front of the nav rather than buried after alphabetical siblings.

Output paths mirror source structure but use pretty URLs — each page becomes a `<dir>/index.html` so it serves at a trailing-slash URL: `content/blog/post.md` → `_site/blog/post/index.html` (served at `/blog/post/`). The root `content/index.md` and any `content/<dir>/index.md` section index map straight to `<dir>/index.html`.

### nav module layout

`src/core/nav/` is a directory module split by concern: `mod.rs` (`PageContext`, `NavItem`, `compute_context`, nav-inclusion rules, `is_root_index`), `url.rs` (`href_for` / `path_to_url` URL building), and `breadcrumb.rs` (`Breadcrumb` + breadcrumb trail). Shared test fixtures live in `src/core/nav/test_support.rs` (`#[cfg(test)]` only). Cross-crate callers reach the helpers through the re-exports in `mod.rs` (`nav::href_for`, `nav::is_root_index`).

### Key constraints

**Sailfish templates must use `#[derive(TemplateSimple)]`**, not `TemplateOnce`. `TemplateSimple` destructures struct fields into local variables so templates can reference them as bare identifiers (`<%= title %>`). `TemplateOnce` does not destructure and will produce "cannot find value in scope" compile errors.

**`gray_matter` TOML support** requires `features = ["toml"]` in `Cargo.toml` (not enabled by default). TOML delimiter is `+++`; a separate `Matter::<TOML>` instance with `matter.delimiter = "+++".to_owned()` is required since gray_matter defaults all engines to `---`.

**axum 0.8 static files**: use `Router::fallback_service(ServeDir::new(...))` — `nest_service("/", ...)` panics at runtime.

**`SiteConfig` is `#[non_exhaustive]`** so fields can be added without a breaking change. In-crate code may still use struct-literal construction; downstream library users cannot — they build via `SiteConfig::new()` / `default()` plus the `with_content_dir` / `with_output_dir` builder methods. Add a matching `with_*` method (and a default) for any new field.

**URL building from `Path`**: do not use `Path::display()` when constructing href strings. On Windows it emits `\` separators, producing invalid URLs like `/blog\post.html`. `path_to_url` (in `src/core/nav/url.rs`) walks `Component::Normal` and joins with `/` — use it for any new URL output.
