# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
