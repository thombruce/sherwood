# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
