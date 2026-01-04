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

Marks an index page as a list page that automatically displays all other content in the same directory. The list will be rendered after the page content.

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

### `sort_by` (string, optional)

Specifies the field to sort content by when this page is a list page (`list = true`). Only affects the ordering of content in the automatically generated list.

**Available values:**
- `"date"` - Sort by frontmatter date (default for list pages)
- `"title"` - Sort by page title  
- `"filename"` - Sort by filename

**TOML:**
```toml
+++
list = true
title = "Blog"
sort_by = "date"
+++
```

**YAML:**
```yaml
---
list: true
title: "Blog"
sort_by: "date"
---
```

### `sort_order` (string, optional)

Specifies the sort direction when `list = true`. Defaults to `"desc"` for date sorting and `"asc"` for other fields.

**Available values:**
- `"asc"` - Ascending order (A-Z, oldest first)
- `"desc"` - Descending order (Z-A, newest first)

**TOML:**
```toml
+++
list = true
title = "Blog"
sort_by = "date"
sort_order = "desc"
+++
```

**YAML:**
```yaml
---
list: true
title: "Blog"
sort_by: "date"
sort_order: "desc"
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

**Basic list (default date sorting, newest first):**
```markdown
+++
list = true
title = "Blog"
+++

# Blog

Welcome to my blog! Here you'll find articles about various topics.
```

**Blog with custom sorting (newest first):**
```markdown
+++
list = true
title = "Blog"
sort_by = "date"
sort_order = "desc"
+++

# Blog

Welcome to my blog! Here you'll find articles about various topics.
```

**Blog sorted by title alphabetically:**
```markdown
+++
list = true
title = "Blog"
sort_by = "title"
sort_order = "asc"
+++

# Blog

Welcome to my blog! Here you'll find articles about various topics.
```

**YAML (Legacy):**
```markdown
---
list: true
title: "Blog"
sort_by: "date"
sort_order: "desc"
---

# Blog

Welcome to my blog! Here you'll find articles about various topics.
```

## List Rendering and Sorting

For list pages, the content list is automatically rendered after all page content when `list = true` is set in the frontmatter. The list can be sorted using the `sort_by` and `sort_order` fields.

### Default Sorting Behavior

- When `list = true` but no sorting options specified: defaults to date descending (newest first)
- When `sort_by` is specified but no `sort_order`: defaults to ascending for all fields except date
- Date sorting: files with valid dates come before files without dates
- Invalid dates fall back to filename sorting within their group

### Supported Date Formats

The date field supports multiple formats for flexible input:

- `2024-01-15` (ISO 8601 format - recommended)
- `January 15, 2024` (full month name)
- `Jan 15, 2024` (abbreviated month name)
- `15/01/2024` (DD/MM/YYYY format)
- `01/15/2024` (MM/DD/YYYY format)

### Sorting Examples

**Blog with newest posts first:**
```toml
+++
list = true
title = "Blog"
sort_by = "date"
sort_order = "desc"
+++
```

**Documentation alphabetical by title:**
```toml
+++
list = true
title = "Documentation"
sort_by = "title"
sort_order = "asc"
+++
```

**Simple list with default sorting (date, newest first):**
```toml
+++
list = true
title = "Updates"
+++
```

### Error Handling

- Invalid `sort_by` values: falls back to date sorting with warning
- Invalid `sort_order` values: falls back to "asc" with warning
- Unparseable dates: falls back to filename sorting
- All sorting errors are logged with helpful messages

## Format Comparison

| Feature | TOML (Recommended) | YAML (Legacy) |
|---------|-------------------|---------------|
| **Delimiters** | `+++` | `---` |
| **Syntax** | `key = "value"` | `key: "value"` |
| **Booleans** | `true`, `false` | `true`, `false` |
| **Strings** | `key = "value"` (quotes required) | `key: value` or `key: "value"` |
| **Comments** | `# comment` | `# comment` |
| **Readability** | Simple key-value pairs | More verbose, requires proper indentation |

## Available Fields Summary

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `title` | string | no | Auto-extracted | Page title |
| `date` | string | no | none | Publication date (multiple formats) |
| `list` | boolean | no | `false` | Mark as list page |
| `page_template` | string | no | `default.stpl` | Custom template file |
| `sort_by` | string | no | `date` (for lists) | Sort field (`date`, `title`, `filename`) |
| `sort_order` | string | no | `desc` (for date), `asc` (others) | Sort direction (`asc`, `desc`) |

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