+++
title = "Getting Started"
date = "2024-01-10"
+++

# Getting Started

Sherwood is a simple yet powerful static site generator for Markdown content. This guide will help you get up and running quickly.

## Installation

Sherwood is written in Rust. To install, you'll need to have Rust installed on your system.

### Prerequisites

- Rust 1.70+ (recommended)
- Cargo (comes with Rust)

### Building from Source

```bash
git clone <repository-url>
cd sherwood
cargo build --release
```

## Quick Start

1. **Create a content directory:**
   ```bash
   mkdir content
   echo "# Welcome to My Site" > content/index.md
   ```

2. **Generate your site:**
   ```bash
   sherwood generate
   ```

3. **Start development server:**
   ```bash
   sherwood dev
   ```

4. **Open your browser:**
   Navigate to `http://localhost:3000`

## Directory Structure

Sherwood expects a simple, intuitive directory structure:

```
content/
├── index.md          # Homepage
├── about.md          # About page
├── blog/             # Blog section
│   ├── index.md      # Blog index (list page)
│   └── post.md       # Individual blog post
└── docs/             # Documentation section
    ├── index.md      # Docs index (list page)
    └── guide.md      # Documentation page
```

## Content Types

### Regular Pages

Simple Markdown files that get converted to HTML:

```markdown
# About Us

We are a company that makes amazing things.
```

### Blog Posts

Markdown files with optional frontmatter:

```markdown
---
title: "My First Post"
date: "2024-01-15"
---

# My First Post

This is the content of my post.
```

### List Pages

Directory index pages that automatically list all posts in that directory:

```markdown
---
list: true
title: "Blog"
---

# Blog

<!-- BLOG_POSTS_LIST -->
```

## Next Steps

- Learn about [Frontmatter](/docs/frontmatter)
- Explore [Blog Features](/docs/blog-features)
- Check out [Deployment Options](/docs/deployment)