# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Pluggable content parsers. A `ContentParser` trait turns one file's raw source into a `Parsed { frontmatter, content_html, excerpt_html }` payload; a `ParserRegistry` maps file extensions to parsers (`default()` registers the built-in `MarkdownParser` for `.md`/`.markdown`; `empty()` starts bare; `register(Arc::new(MyParser))` adds one, last registration for an extension wins). New public exports: `ContentParser`, `Parsed`, `ParserError`, `ParserRegistry`, `MarkdownParser`, `markdown_to_html`, `split_frontmatter`, plus the layered error types `FrontmatterError` and `PageError`.
- Base-path support for subpath hosting (e.g. `https://host/sherwood/`). `SiteConfig.base_path` (via `with_base_path` or `--base-path` on `build`/`serve`, normalized to `""` or `"/prefix"`) prefixes generated URLs: `NavItem.href`, `Breadcrumb.href`, and prev/next hrefs come pre-resolved; `PageContext::{base_path, resolve}` cover hrefs templates build themselves. `page.url` and `pages_under` stay canonical (un-prefixed). Output paths are unaffected. The dev server mounts the site under the base path and redirects `/` to it, matching production.
- Static-asset passthrough: files in the content tree whose extension has no registered parser (images, downloads, extra CSS, …) are now copied verbatim to the mirrored output path (`content/blog/img.png` → `_site/blog/img.png`) instead of being silently dropped from the build.
- Duplicate-output detection: two sources that map to the same output file (e.g. `content/about.md` and `content/about/index.md`, both → `_site/about/index.html`; or a static `about/index.html` colliding with a rendered page) now fail the build with `BuildError::DuplicateOutput` naming both sources, instead of one silently overwriting the other.
- Dogfooding site: `site/` is a `publish = false` workspace member that builds the project's documentation site (<https://sherwood.thombruce.com>) through the library/`run_cli` path with its own template and stylesheet, deployed to GitHub Pages on push to `main`. Excluded from the published crate.
- CI now builds and tests the full feature matrix (default, no-default, `cli`-only, `default-template`-only) plus the site as a smoke test.

### Changed

- **Breaking:** `build_site` gains a `&ParserRegistry` second parameter: `build_site(&config, &registry, renderer, progress)`. Pass `&ParserRegistry::default()` for the previous markdown-only behaviour.
- **Breaking:** `run_cli` / `try_run_cli` gain a `ParserRegistry` first parameter so binary authors can register custom parsers: `run_cli(ParserRegistry::default(), renderer, assets)`.
- **Breaking:** the module tree is now private behind a facade — `lib.rs` re-exports the public API and internal paths (`sherwood::build::…`, `sherwood::page::…`, etc.) are no longer reachable. In particular the `serve` module (`router`, `router_with_reload`, `serve_with_watch`) is no longer public; the supported serving entry point is the CLI's `serve` subcommand.
- **Breaking:** `SiteConfig` is `#[non_exhaustive]`. Downstream crates construct via `SiteConfig::new()` / `default()` and the `with_content_dir` / `with_output_dir` / `with_base_path` builder methods instead of struct literals.
- **Breaking:** error types are restructured per module. `BuildError::FrontmatterParse { path, message }` is gone; parse failures now arrive as `BuildError::Page(PageError::Parse { path, source: ParserError })` with `FrontmatterError` at the bottom of the chain, and display transparently (path + line-numbered snippet, no stacked prefixes).
- **Breaking:** `parse_frontmatter(source, path) -> (FrontMatter, String, Option<String>)` is replaced by `split_frontmatter(source) -> Result<(FrontMatter, String), FrontmatterError>`. Excerpt extraction (`<!-- more -->`) moved into `MarkdownParser` — it's a markdown concern, not a frontmatter one. Third-party parsers using the `---`/`+++` convention call `split_frontmatter` and handle excerpts themselves.
- The build walks every file and dispatches by extension through the registry, instead of hardcoding `.md`; `.markdown` files are now recognized.

### Removed

- The root demo `content/` directory. The bare `cargo run -- build` / `serve` now need a `content/` dir to exist; use `--content-dir` or the `site/` workspace member for a runnable example.

## [0.5.0] - 2026-05-31

### Added

- `sherwood serve` now watches the content directory and reruns the build on file changes. Each served HTML response gets a tiny `<script>` injected before `</body>` that connects to `/_sherwood/reload`; the server pushes a `reload` message after a successful rebuild and the browser reloads. Pass `--no-watch` to fall back to plain static serving.
- `Serve` CLI subcommand gains `--content-dir` (defaults to `content`), `--asset name=path` (same override semantics as `build`, re-applied on every rebuild), and `--no-watch`.
- `serve::router_with_reload(output_dir, broadcast::Sender<()>)` and `serve::serve_with_watch(content_dir, output_dir, port, rebuild_fn, watch)` exposed for library users who want to wire their own watcher or websocket-driven reload behaviour.
- Frontmatter parse errors now include the offending frontmatter block with line numbers. Example:
  ```
  Frontmatter parse error in content/bad.md: missing required field `title`

      1 | ---
      2 | foo: bar
      3 | ---
  ```

### Changed

- **Breaking:** `run_cli` / `try_run_cli` now require the renderer closure to be `Send + 'static`. The watcher's rebuild closure runs on a `tokio::task::spawn_blocking` worker, which needs `Send`. Most renderers (struct-construction + `render_once`) already satisfy this; closures capturing non-`Send` state will fail to compile.
- `Cargo.toml`: `cli` feature now also pulls in `notify-debouncer-mini` and `axum`'s `ws` feature.
- File-watch loop snapshots content-file mtimes between rebuilds and ignores events that don't change them. Reading `.md` files during a rebuild updates `atime`, which fires `IN_ATTRIB` events on Linux; without the snapshot guard, every rebuild would self-trigger another.

### Tests

- Test count: 74 → 79.
- New: `router_with_reload` injects the reload script into `text/html` responses and leaves non-HTML responses untouched; frontmatter parse errors include the source snippet with line numbers (malformed YAML, missing-title, no-delimiter cases).

## [0.4.0] - 2026-05-31

### Added

- Pretty URLs. Every page other than an `index.md` file is now written to `<dir>/<stem>/index.html` so the dev server (and any static host that auto-serves `index.html`) resolves clean URLs without a `.html` suffix. URL examples: `content/about.md` → `/about/`, `content/blog/first.md` → `/blog/first/`. Section indexes (`<dir>/index.md`) and the root `index.md` stay flat at `<dir>/index.html` and serve as `/<dir>/` and `/`. `axum` `ServeDir` returns a 307 redirect from `/about` to `/about/`, so omitting the trailing slash still resolves.
- `Page.is_section_index: bool` — `true` when the source file is named `index.md`. Used internally to distinguish wrapped regular pages from section landing pages; exposed for downstream renderers that need the same distinction.

### Changed

- **Breaking:** `Page.output_path` now points at `<dir>/index.html` for non-index sources. Build pipelines or tests that assert on output paths must update accordingly (`_site/about.html` → `_site/about/index.html`, etc.).
- **Breaking:** `Page.url` (and `PageContext.nav` hrefs, prev/next hrefs, breadcrumb hrefs) now use directory-style paths with trailing slashes (`/about/` rather than `/about.html`). Templates that hardcoded `.html`-suffixed URLs in markdown links or template fragments must be updated.
- `breadcrumbs_for` now reads dir-href via `href_for` so dir crumbs use the new pretty-URL form (`/blog/` rather than `/blog/index.html`).
- `include_in_nav` now branches on `Page.is_section_index` instead of inspecting the output filename, since every page's output filename is now `index.html`.

### Tests

- Test count: 69 → 74.
- New: pretty URL derivation (`href_for` for flat, nested, root index, section index), section-index flag set in `load_page`, output_path wraps non-index files.
- Migrated: every nav test fixture switched to a `make_page("source-stem", title)` helper that mirrors `load_page` (computes source path, output path, URL, and `is_section_index` from a single input).

## [0.3.0] - 2026-05-31

### Added

- Default stylesheet bundled into the binary via `include_str!("../templates/style.css")`. Written once to `<output_dir>/style.css` after `build_site` — zero per-page cost.
- `<link rel="stylesheet" href="/style.css">` injected into the bundled `page.stpl` template.
- `sherwood::run_cli(renderer, assets) -> ExitCode` and `try_run_cli(renderer, assets) -> Result<(), CliError>` — high-level library entry points that parse the standard `build` / `serve` clap CLI, run `build_site`, and write static assets. Owns its own tokio runtime; library consumers no longer need `#[tokio::main]`.
- `sherwood::Asset { dest, bytes }` — destination-path + `Cow<'static, [u8]>` pair for shipping static files (CSS, images, etc.) alongside a build. Replaces ad-hoc `fs::write` boilerplate in downstream binaries.
- `sherwood build --asset name=path` flag — generic asset override. Replaces a bundled asset whose `dest` matches `name`, or appends a new one. May be repeated.
- Cargo features `cli` (clap + tokio + axum + tower-http + `run_cli`/`Asset`/`serve` module) and `default-template` (sailfish + bundled `render_page` + `DEFAULT_STYLE`). Both enabled by default. Required to build the `sherwood` binary; library consumers can disable either or both.
- `FrontMatter` now exposes arbitrary frontmatter fields beyond `title` via a `data: gray_matter::Pod` field. New helpers `FrontMatter::get(key) -> Option<&Pod>` and `FrontMatter::get_string(key) -> Option<String>` for typed lookups. `gray_matter::Pod` re-exported as `sherwood::Pod` so downstream templates can pattern-match without depending on `gray_matter` directly.
- `Page` gains a precomputed `url: String` field (cross-platform absolute URL, e.g. `/blog/first-post.html`) so templates can build links and filter pages by prefix without re-computing href strings or handling path separators themselves.
- `Page` gains `excerpt_html: Option<String>`. When a source file contains a `<!-- more -->` delimiter, everything before it is extracted, rendered to HTML, and stored here. `None` otherwise. Powers blog/section index pages that show post previews.
- `PageContext::pages_under(url_prefix) -> Vec<&Page>` for collection queries — drives section indexes (e.g. a `/blog/index.html` page can list every post under `/blog/`). Backed by a new `pages: &[Page]` field on `PageContext` that exposes the full corpus for arbitrary filtering, sorting, and grouping.
- Nav scoping: the top-level `PageContext.nav` now lists top-level pages and section indexes only, hiding deep leaf pages (e.g. blog posts under `/blog/`). Frontmatter `nav: true` force-includes a page that would otherwise be hidden; `nav: false` force-excludes one that would otherwise appear. Prev/next still walks the full build order — only the global nav is scoped.

### Changed

- **Breaking:** `--style <path>` flag removed in favour of generic `--asset style.css=<path>`.
- **Breaking:** `clap`, `tokio`, `axum`, `tower-http`, and `sailfish` are now optional dependencies behind the `cli` and `default-template` features. Headless library consumers should add `default-features = false` to drop them. Consumers who want `run_cli` but bring their own template should use `default-features = false, features = ["cli"]`.
- `src/main.rs` collapsed from ~70 lines to ~10 by delegating to `sherwood::run_cli` with the bundled `render_page` + `DEFAULT_STYLE`. Equivalent shrink available to all downstream library consumers.
- Promoted `serve` and the default template (`src/templates.rs` → `src/default_template.rs`) from binary-only modules into library modules, gated by the new features.
- **Breaking:** Bundled `PageTemplate` is now lifetime-parameterised (`PageTemplate<'a>`) with borrowed fields (`&'a str`, `&'a [NavItem]`, `Option<&'a NavItem>`). `render_page` no longer clones per page — the O(N²) nav-vector cloning across an N-page build is gone. Downstream custom templates that mirror the bundled shape need to switch to borrowed fields, and any Sailfish expression that projects a `String` through a reference (e.g. `<%= item.href %>`) must take an explicit borrow (`<%= &item.href %>`) so Sailfish does not try to move the field out of the `&T`.
- **Breaking:** `FrontMatter` gains a required `data: Pod` field. Constructors must populate it (use `Pod::Null` if no other fields are needed). The struct no longer derives `Deserialize`; arbitrary fields are accessed at runtime via `get` / `get_string` rather than typed deserialization. Removes the direct `serde` dependency from the library.
- **Breaking:** `Page` gains required `url: String` and `excerpt_html: Option<String>` fields. Test fixtures and downstream constructors must populate them.
- **Breaking:** `PageContext` is now `PageContext<'a>` (lifetime-parameterised) with a new required `pages: &'a [Page]` field. `compute_context` returns `PageContext<'a>` borrowing from the `all_pages` slice. Renderer closures `FnMut(&Page, &PageContext) -> Result<String, BuildError>` still elide the lifetime so no signature change is needed at call sites.
- **Breaking:** Default nav inclusion changed. Flat sites that previously listed every page in the global nav will now hide deep leaf pages (anything under a subdirectory that isn't `index.html`). Add `nav: true` frontmatter to pages that should remain visible.

### Fixed

- Breadcrumbs no longer duplicate the leaf segment for `<dir>/index.html` pages. Viewing `/blog/index.html` now renders `Home > Blog` (with the `Blog` crumb unlinked to mark current), rather than `Home > Blog > Blog`.

### Tests

- Test count: 45 → 69.
- New: `parse_asset_override` happy + error paths, `apply_overrides` replace/append/missing-file behaviour, YAML + TOML extra-field access via `get_string`, missing-key returns `None`, TOML datetime coerces to string, YAML array fields accessible as `Pod::Array`, `Page.url` set from output path, `Page.excerpt_html` populated when `<!-- more -->` present (and `None` when absent), `parse_frontmatter` returns excerpt slot for both delimiter present + absent cases, `PageContext::pages_under` filters by URL prefix + returns empty for unknown prefix, `pages` slice exposed on context, `<dir>/index.html` breadcrumbs no longer duplicate the leaf, nav-scoping rules (top-level included, section indexes included, deep leaves excluded, `nav: false` hides, `nav: true` force-includes).

## [0.2.0] - 2026-05-30

### Fixed

- Windows path-separator bug in nav and breadcrumb URLs: `Path::display()` was emitting `\` on Windows, producing invalid hrefs like `/blog\post.html`. Replaced with `nav::path_to_url`, which joins `Component::Normal` segments with `/`.

### Changed

- Root `index.html` now sorts first in nav and prev/next ordering, instead of falling after alphabetical siblings like `about.html`. Sort key is `(!is_root_index, output_path)`.
- `build_site` signature gains a `progress: FnMut(&Page)` callback invoked after each page is written. CLI logging moved from the library into `src/main.rs`. Renderer bound relaxed from `Fn` to `FnMut`. **Breaking change** for direct library consumers — pass `|_| {}` to silence.

### Refactored

- Extracted `frontmatter::finalize<E: Engine>` to deduplicate YAML and TOML parse branches.
- Extracted `nav::nav_item_for` (shared by nav, prev, next construction) and `nav::is_root_index` (shared by breadcrumb home detection and build sort).
- Replaced `for i in 0..num_dirs` indexed loop in `breadcrumbs_for` with `components.iter().take(num_dirs)`.

### Tests

- Test count: 35 → 45.
- New: malformed YAML/TOML frontmatter, renderer-error propagation, progress-callback invocation, root-index sort, `path_to_url` Windows-safety, `is_root_index` detection, breadcrumb dir-href assertion.
- New `serve.rs` tests via `tower::ServiceExt::oneshot` — existing file, nested file, 404. Extracted `serve::router(&Path) -> Router` for testability.

### Dev dependencies

- Added `tower` (`features = ["util"]`) and `http-body-util` for serve-route tests.

## [0.1.0] - 2026-05-30

### Added

- Static site generator: converts `content/**/*.md` to `_site/**/*.html`
- YAML frontmatter support (`---` delimiter) via `gray_matter`
- TOML frontmatter support (`+++` delimiter) via `gray_matter`
- Markdown-to-HTML conversion via `pulldown-cmark`
- Output structure mirrors source: `content/blog/post.md` → `_site/blog/post.html`
- Global navigation bar on every page with `aria-current="page"` on active link
- Breadcrumb navigation from directory structure (hidden on root page)
- Prev/Next sequential page links (alphabetical order by output path)
- `sherwood build` CLI command with `--content-dir` and `--output-dir` flags
- `sherwood serve` CLI command with `--port` flag (default 4000)
- Library crate API: `build_site`, `SiteConfig`, `Page`, `FrontMatter`, `PageContext`, `NavItem`, `Breadcrumb`, `BuildError`
- Two-pass build pipeline: collect all pages, sort alphabetically, then render each with full navigation context
- 35 unit and integration tests
