---
title: Home
---

# Sherwood

A small, opinionated **static site generator** written in Rust. Markdown (and
more) in, a clean static site out — with pretty URLs, live-reload, and a
library API you can build your own site on.

This very site is built with Sherwood, using Sherwood as a library. If the
build were awkward, you'd be reading about it here.

## Why Sherwood

- **Dual delivery.** Use the `sherwood` binary for zero-config builds, or depend
  on the crate and bring your own templates.
- **Pretty URLs.** Every page is written as `<dir>/index.html`, so it serves at
  a clean trailing-slash URL.
- **Pluggable parsers.** Markdown and plain text ship built in; add your own
  content format by implementing one trait.
- **Live reload.** `sherwood serve` watches your content and refreshes the
  browser on save.

## Get going

```bash
cargo install sherwood
sherwood build      # content/ -> _site/
sherwood serve      # dev server at http://127.0.0.1:4000
```

Head to [Getting Started](/getting-started/) for the full tour, or the
[Guide](/guide/) to build on the library.
