# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
