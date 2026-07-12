# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                          # compile
cargo test                           # run all tests
cargo test frontmatter               # run tests in a specific module
cargo run -- build --content-dir src --output-dir out  # build a content dir → output dir
cargo run -- build --asset style.css=my.css  # override a bundled asset from disk
cargo run -- build --base-path /sherwood  # prefix generated URLs for subpath hosting
cargo run -- serve                   # dev server at http://127.0.0.1:4000
cargo run -- serve --port 4001       # custom port
cargo run -- serve --no-watch        # static server, no file-watch/live-reload
```

The root `sherwood` crate ships **no demo content** — for a runnable build pass `--content-dir` (or use the `site/` member below); the bare `cargo run -- build`/`serve` need a `content/` dir to exist.

This is a cargo **workspace** with two members: the `sherwood` crate (root) and `site/` (the dogfooding site). `cargo build` / `cargo test` from the root default to the `sherwood` package only — the feature-matrix commands below still target it. Use `-p sherwood-site` for the site:

```bash
cargo run -p sherwood-site -- build --content-dir site/content --output-dir site/_site
cargo run -p sherwood-site -- serve --content-dir site/content --output-dir site/_site
```

The `sherwood` crate has two features, both on by default: `cli` (clap/axum/tokio dev server) and `default-template` (the bundled Sailfish template + stylesheet). The `sherwood` binary requires both. Library-only consumers can disable them with `default-features = false`.

`site/` is a `publish = false` workspace member that depends on `sherwood` (`default-features = false, features = ["cli"]`) and ships its own Sailfish template + stylesheet — it's the canonical example of the library/`run_cli` path. The root `[package]` sets `exclude = ["/site"]` so the site never lands in the published `sherwood` crate; `site/_site/` is gitignored. CI builds the site as a smoke test (the `site` job).

The site deploys to GitHub Pages via `.github/workflows/pages.yml` (official Pages actions, source = "GitHub Actions" — no `gh-pages` branch) on push to `main`. **Sherwood emits absolute URLs (`/style.css`, `/guide/`), so the site must be served from a domain root, not a project subpath.** It uses the custom domain in `site/CNAME` (copied into the build output by the deploy job); change that file to rehome it. One-time setup: repo Settings → Pages → Source = "GitHub Actions", and point the domain's DNS at GitHub Pages. To instead host on the project subpath `thombruce.github.io/sherwood/`, drop the CNAME and build with `--base-path /sherwood` (see the base-path constraint below).

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
    &ParserRegistry::default(),
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

**Error types are layered per module, each owning its own enum:** `FrontmatterError` (core/content/frontmatter.rs — `MissingDelimiters` / `Invalid(String)`, no file path) → `ParserError` (core/content/parser/mod.rs — `Frontmatter(#[from] FrontmatterError)` / `Message(String)`) → `PageError` (core/content/page.rs — `Read` / `Parse`, both carrying the source `PathBuf`) → `BuildError` (core/build.rs — `Io` / `Walk` / `Page(#[from] PageError)` / `Render` / `DuplicateOutput`). Lower modules never import a higher module's error; `?` bubbles up through `#[from]`. `BuildError::Page` and `PageError::Parse`'s inner `ParserError::Frontmatter` are `#[error(transparent)]`, so the displayed message is the chain (path + frontmatter snippet) with no extra prefixes.

### Content parsers (plugin system)

Parsing is pluggable. A `ContentParser` (core/content/parser/mod.rs) turns one file's raw source into a `Parsed { frontmatter, content_html, excerpt_html }`; it never computes paths/URLs — that stays in `load_page`. Parsers are `Send + Sync` (the dev server shares the registry across threads) and object-safe (`dyn ContentParser`).

```rust
pub trait ContentParser: Send + Sync {
    fn extensions(&self) -> &[&str];                 // ["md", "markdown"], no dot
    fn parse(&self, source: &str, path: &Path) -> Result<Parsed, ParserError>;
}
```

A `ParserRegistry` maps extension → `Arc<dyn ContentParser>`. Constructors: `default()` registers the built-in markdown parser; `empty()` starts with none. `register(Arc::new(MyParser))` adds one (last registration for an extension wins). The build walks **all** files; any file whose extension has no registered parser (`load_page` returns `Ok(None)`) is a **static asset** and is copied verbatim to the mirrored output path (`content/blog/img.png` → `_site/blog/img.png`), so images/CSS can live in the content tree. **To add a built-in format:** new file in `core/content/parser/`, `mod`/`pub use` it, register it in `ParserRegistry::default`, add it to the lib.rs facade.

Third-party parsers own their whole file, including their metadata convention. Formats that use the `---`/`+++` convention call the public `split_frontmatter(source) -> Result<(FrontMatter, String), FrontmatterError>` helper; others ignore it (taking their title from elsewhere). Parser-API exports: `ContentParser`, `Parsed`, `ParserError`, `ParserRegistry`, `MarkdownParser`, `markdown_to_html`, `split_frontmatter`.

Built-in: `MarkdownParser` (core/content/parser/markdown.rs) owns markdown rendering (`markdown_to_html`, `pulldown-cmark`) and the `<!-- more -->` excerpt split — excerpt is a markdown concern, not a frontmatter one, so `split_frontmatter` no longer touches it.

### Feature modules

The bundled template and CLI live behind cargo features and are re-exported from `src/lib.rs` (not binary-only):

- **`default-template`** → `src/default_template.rs`. Owns the baked-in Sailfish template (`templates/page.stpl`, compiled at build time) and embeds `templates/style.css` via `include_str!` as `DEFAULT_STYLE`. Public exports: `render_page` (the ready-made render closure) and `DEFAULT_STYLE`. The core library does not ship or prescribe a stylesheet — pure-library users embed their own CSS in their downstream binary.
- **`cli`** → `src/cli/mod.rs` (+ `src/cli/serve.rs` dev server). Owns the clap arg parsing and the `run_cli` / `try_run_cli` entry points. Public exports: `run_cli`, `try_run_cli`, `try_run_cli_from` (injectable args, for tests/embedders), `Asset`, `CliError`.

`src/main.rs` is a thin shim: it calls `run_cli(ParserRegistry::default(), render_page, vec![Asset::new("style.css", DEFAULT_STYLE.as_bytes())])`. `run_cli` / `try_run_cli` take the registry as their first argument so binary authors can register custom parsers. Assets are written to `<output_dir>` after `build_site`. `--asset <name>=<path>` overrides a bundled asset (matched by its `dest`) with a file from disk; the flag is repeatable.

### Build pipeline flow

Two-pass pipeline — all pages collected and sorted before any rendering begins:

```
Pass 1 — collect:
  content/**/*  (every file)
    └─ load_page()  [core/content/page.rs]
         registry.get(ext)? → read file → parser.parse() → Page
         (returns None if no parser claims the extension → the file is a
          static asset, copied verbatim to the mirrored output path)
         every output path is claimed in an output→source map; two sources
         mapping to the same output (e.g. about.md + about/index.md) fail
         the build with BuildError::DuplicateOutput

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

`src/core/nav/` is a directory module split by concern: `mod.rs` (`PageContext`, `NavItem`, `compute_context`, nav-inclusion rules, `is_root_index`), `url.rs` (`href_for` / `path_to_url` URL building, `section_of` URL-parent grouping), and `breadcrumb.rs` (`Breadcrumb` + breadcrumb trail). Prev/next is **section-scoped**: pages chain only among siblings with the same URL parent (`section_of(page.url)`), so `/blog/first/` links to other posts while section indexes like `/blog/` chain in the parent (root) sequence. Shared test fixtures live in `src/core/nav/test_support.rs` (`#[cfg(test)]` only). Cross-crate callers reach the helpers through the re-exports in `mod.rs` (`nav::href_for`, `nav::is_root_index`).

### Key constraints

**Sailfish templates must use `#[derive(TemplateSimple)]`**, not `TemplateOnce`. `TemplateSimple` destructures struct fields into local variables so templates can reference them as bare identifiers (`<%= title %>`). `TemplateOnce` does not destructure and will produce "cannot find value in scope" compile errors.

**`gray_matter` TOML support** requires `features = ["toml"]` in `Cargo.toml` (not enabled by default). TOML delimiter is `+++`; a separate `Matter::<TOML>` instance with `matter.delimiter = "+++".to_owned()` is required since gray_matter defaults all engines to `---`.

**axum 0.8 static files**: use `Router::fallback_service(ServeDir::new(...))` — `nest_service("/", ...)` panics at runtime.

**`SiteConfig` is `#[non_exhaustive]`** so fields can be added without a breaking change. In-crate code may still use struct-literal construction; downstream library users cannot — they build via `SiteConfig::new()` / `default()` plus the `with_content_dir` / `with_output_dir` builder methods. Add a matching `with_*` method (and a default) for any new field.

**URL building from `Path`**: do not use `Path::display()` when constructing href strings. On Windows it emits `\` separators, producing invalid URLs like `/blog\post.html`. `path_to_url` (in `src/core/nav/url.rs`) walks `Component::Normal` and joins with `/` — use it for any new URL output.

**Base path (subpath hosting).** `SiteConfig.base_path` (set via `with_base_path` / `--base-path`, normalized to `""` or `"/prefix"`) prefixes generated URLs so a site can serve from `https://host/sherwood/`. The model is **canonical-internal, resolve-at-the-render-boundary**: `page.url` and `pages_under` stay canonical (un-prefixed) for matching/identity; only rendered hrefs carry the prefix. The library pre-resolves `NavItem.href`, `Breadcrumb.href`, and prev/next hrefs. Templates: use those directly, but wrap hrefs you build from `page.url`/`pages_under` in `ctx.resolve(...)`, and prefix static assets with `ctx.base_path` (`<%= base_path %>/style.css`). `nav::resolve(canonical, base)` is the primitive; `PageContext::{base_path, resolve}` expose it to render closures. **Base path affects URLs only — never output paths** (files stay at `_site/<dir>/index.html`; the host maps the subpath to the artifact root). `serve` mounts the dev server under the base path (`nest_service` + `/`→`/base/` redirect) so the preview matches production.
