# Sherwood

> [!WARNING]
> Sherwood is a work in progress. Some of the features described below may not yet be stable or even working.
> Some may be removed in future updates.

A fast and simple static site generator written in Rust that converts Markdown content to semantic HTML.

## Features

- 🚀 **Fast static site generation** written in Rust
- 📝 **Markdown to HTML5** conversion with semantic structure
- 🎨 **Theme support** with built-in themes (default, kanagawa)
- 🔧 **Frontmatter support** for metadata (title, date, theme, etc.)
- 📱 **Responsive design** with semantic HTML
- 🛠️ **Development server** for local testing
- 📋 **Blog post lists** with automatic generation
- ⚙️ **Configurable** via `sherwood.toml`

## Installation

```bash
cargo install sherwood
```

Or build from source:

```bash
git clone <repository-url>
cd sherwood
cargo build --release
```

## Quick Start

1. Create a `content` directory with Markdown files
2. Configure your site in `sherwood.toml` (optional)
3. Generate your site or run the development server

### Commands

#### Generate static site
```bash
sherwood generate
sherwood generate -i content -o dist
```

#### Run development server
```bash
sherwood dev
sherwood dev -i content -o dist -p 3000
```

## Configuration

Create a `sherwood.toml` file in your project root:

```toml
[site]
theme = "kanagawa"  # Options: default, kanagawa
```

## Frontmatter

Add metadata to your Markdown files:

```yaml
---
title: "My Blog Post"
date: "2025-01-01"
theme: "kanagawa"
theme_variant: "dark"
list: true  # For blog index pages
---

# Your content here...
```

## Directory Structure

```
project/
├── content/           # Markdown files
│   ├── index.md
│   ├── about.md
│   └── blog/
│       ├── index.md   # Blog list page
│       └── post.md
├── themes/            # Custom themes (optional)
├── sherwood.toml      # Site configuration
└── dist/             # Generated site (output)
```

## Markdown Support

- Standard Markdown syntax
- Tables
- Footnotes
- Strikethrough
- Code blocks with syntax highlighting
- Semantic HTML generation

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Development mode
cargo run -- dev
```

## License

[Add your license here]
