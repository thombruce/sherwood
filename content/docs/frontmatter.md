---
title: "Frontmatter Reference"
date: "2024-01-12"
---

# Frontmatter Reference

Frontmatter is optional YAML metadata at the beginning of your Markdown files. It allows you to add structured data to your content.

## Syntax

Frontmatter must be the first thing in your file and must be valid YAML set between triple-dashed lines:

```yaml
---
title: "My Page Title"
date: "2024-01-15"
list: true
---
```

## Available Fields

### `title` (string)

Overrides the page title. If not provided, Sherwood will extract the first H1 heading or use the filename.

```yaml
---
title: "Custom Page Title"
---
```

### `date` (string)

Publication date for content sorting and display. Used in blog listings and other time-sensitive content.

```yaml
---
date: "2024-01-15"
---
```

### `list` (boolean)

Marks an index page as a list page that automatically displays all other content in the same directory.

```yaml
---
list: true
title: "Blog"
---
```

## Usage Examples

### Blog Post

```markdown
---
title: "Understanding Rust Ownership"
date: "2024-01-20"
---

# Understanding Rust Ownership

Rust's ownership system is one of its most distinctive features...
```

### Documentation Page

```markdown
---
title: "API Reference"
---

# API Reference

Here are the available API endpoints...
```

### Blog Index

```markdown
---
list: true
title: "Blog"
---

# Blog

<!-- BLOG_POSTS_LIST -->
```

## Placeholder

For list pages, include the placeholder `<!-- BLOG_POSTS_LIST -->` where you want the automatic content listing to appear.

## Field Priority

1. **Title Resolution:**
   - `title` from frontmatter (highest priority)
   - First H1 heading in content
   - Filename (lowest priority)

2. **Content Types:**
   - Pages without `list: true` → rendered as individual pages
   - Pages with `list: true` → automatically generate listings of sibling content

## Best Practices

- Always use quotes around string values in YAML
- Use ISO date format (`YYYY-MM-DD`) for dates
- Keep frontmatter simple - complex nesting isn't currently supported
- Use meaningful filenames that reflect your content

## Future Extensions

Sherwood may support additional frontmatter fields in future versions:

- `description` for SEO meta tags
- `tags` for categorization
- `author` for bylines
- `draft` for unpublished content
- `layout` for different page templates