+++
title = "Frontmatter Reference"
date = "2024-01-12"
+++

# Frontmatter Reference

Frontmatter is optional metadata written in either **TOML** (recommended) or **YAML** at the beginning of your Markdown files. It allows you to add structured data to your content.

## TOML Syntax (Recommended)

Frontmatter written in TOML uses triple-plus delimiters:

```toml
+++
title = "My Page Title"
date = "2024-01-15"
list = true
+++
```

## YAML Syntax (Legacy)

Frontmatter written in YAML uses triple-dashed lines:

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

**TOML:**
```toml
+++
title = "Custom Page Title"
+++
```

**YAML:**
```yaml
---
title: "Custom Page Title"
---
```

### `date` (string)

Publication date for content sorting and display. Used in blog listings and other time-sensitive content.

**TOML:**
```toml
+++
date = "2024-01-15"
+++
```

**YAML:**
```yaml
---
date: "2024-01-15"
---
```

### `list` (boolean)

Marks an index page as a list page that automatically displays all other content in the same directory.

**TOML:**
```toml
+++
list = true
title = "Blog"
+++
```

**YAML:**
```yaml
---
list: true
title: "Blog"
---
```

### `page_template` (string)

Specifies a custom template to use for rendering this page. The template must exist in the templates directory. If not specified, uses the default template.

**TOML:**
```toml
+++
page_template = "custom.stpl"
+++
```

**YAML:**
```yaml
---
page_template: "custom.stpl"
---
```

## Usage Examples

### Blog Post

**TOML (Recommended):**
```markdown
+++
title = "Understanding Rust Ownership"
date = "2024-01-20"
+++

# Understanding Rust Ownership

Rust's ownership system is one of its most distinctive features...
```

**YAML (Legacy):**
```markdown
---
title: "Understanding Rust Ownership"
date: "2024-01-20"
---

# Understanding Rust Ownership

Rust's ownership system is one of its most distinctive features...
```

### Documentation Page

**TOML (Recommended):**
```markdown
+++
title = "API Reference"
+++

# API Reference

Here are the available API endpoints...
```

**YAML (Legacy):**
```markdown
---
title: "API Reference"
---

# API Reference

Here are the available API endpoints...
```

### Blog Index

**TOML (Recommended):**
```markdown
+++
list = true
title = "Blog"
+++

# Blog

<!-- BLOG_POSTS_LIST -->
```

**YAML (Legacy):**
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

## Format Comparison

| Feature | TOML (Recommended) | YAML (Legacy) |
|---------|-------------------|---------------|
| **Delimiters** | `+++` | `---` |
| **Syntax** | `key = "value"` | `key: "value"` |
| **Booleans** | `true`, `false` | `true`, `false` |
| **Strings** | `key = "value"` (quotes required) | `key: value` or `key: "value"` |
| **Comments** | `# comment` | `# comment` |
| **Readability** | Simple key-value pairs | More verbose, requires proper indentation |

### Why TOML is Recommended

- **Simpler syntax**: No complex nesting rules or indentation requirements
- **More explicit**: Clearer distinction between strings and other types
- **Less error-prone**: No indentation-based parsing issues
- **Standard for configuration**: Widely used in Rust ecosystem

## Field Priority

1. **Title Resolution:**
   - `title` from frontmatter (highest priority)
   - First H1 heading in content
   - Filename (lowest priority)

2. **Content Types:**
   - Pages without `list: true` → rendered as individual pages
   - Pages with `list: true` → automatically generate listings of sibling content

3. **Template Selection:**
   - `page_template` from frontmatter (if specified and exists)
   - Default template (`default.stpl`) as fallback

## Best Practices

- **Use TOML format** for new content (`+++` delimiters)
- Use ISO date format (`YYYY-MM-DD`) for dates
- Keep frontmatter simple - complex nesting isn't currently supported
- Use meaningful filenames that reflect your content
- When using `page_template`, ensure the template file exists

## Migration from YAML to TOML

Converting existing YAML frontmatter to TOML is straightforward:

**YAML:**
```yaml
---
title: "My Post"
date: "2024-01-15"
list: true
---
```

**TOML:**
```toml
+++
title = "My Post"
date = "2024-01-15"
list = true
+++
```

**Key changes:**
- Replace `---` with `+++`
- Replace `key: value` with `key = "value"`
- Keep all string values quoted

## Future Extensions

Sherwood may support additional frontmatter fields in future versions:

- `description` for SEO meta tags
- `tags` for categorization
- `author` for bylines
- `draft` for unpublished content