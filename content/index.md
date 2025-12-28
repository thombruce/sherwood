# Welcome to Sherwood

This is a **static site generator** written in Rust that converts Markdown files to semantic HTML.

- [About](/about)
- [Blog](/blog/)

## Features

- Converts Markdown to HTML5 with semantic structure
- Command-line interface with `generate` and `dev` commands
- Development server for local testing
- Semantic HTML with proper accessibility

## Usage

### Generate Site

```bash
sherwood generate
```

### Development Server

```bash
sherwood dev
```

## Markdown Support

Sherwood supports standard Markdown syntax including:

- **Bold text** and *italic text*
- `Code snippets` and code blocks
- Lists (ordered and unordered)
- Links and images
- Tables
- Blockquotes

> This is a blockquote to demonstrate the styling.

### Code Example

```rust
fn main() {
    println!("Hello, Sherwood!");
}
```

### Table Example

| Feature | Status |
|---------|--------|
| Markdown parsing | ✅ Complete |
| HTML generation | ✅ Complete |
| Dev server | ✅ Complete |
| Hot reload | 🚧 Future |

The generator creates semantic HTML with proper structure and basic styling for readability.
