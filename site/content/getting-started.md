---
title: Getting Started
---

# Getting Started

## Install

```bash
cargo install sherwood
```

## Author content

Drop Markdown files under `content/`. Each file needs a frontmatter block with a
`title` — YAML (`---`) or TOML (`+++`):

```markdown
---
title: My First Post
---

# Hello

Body text goes here.
```

Directory structure becomes URL structure:

| Source | Output | URL |
| --- | --- | --- |
| `content/index.md` | `_site/index.html` | `/` |
| `content/about.md` | `_site/about/index.html` | `/about/` |
| `content/blog/index.md` | `_site/blog/index.html` | `/blog/` |
| `content/blog/post.md` | `_site/blog/post/index.html` | `/blog/post/` |

Files with no registered parser (images, CSS) are left alone, so they can live
alongside your content.

## Build and serve

```bash
sherwood build                       # content/ -> _site/
sherwood build --content-dir src --output-dir out
sherwood serve                       # build + watch + live reload
sherwood serve --port 4001
sherwood serve --no-watch            # plain static server
```

## Customise the look

The bundled binary writes a default stylesheet. Override it from disk without
touching the binary:

```bash
sherwood build --asset style.css=my-theme.css
```

For full control over markup, depend on the library and supply your own
templates — see the [Guide](/guide/).
