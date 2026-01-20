# Sherwood

**Sherwood** is a _compile-it-yourself_ static site generator for blazing fast site generation built in Rust!

- [About](/about)
- [Blog](/blog/)
- [Docs](/docs/)

## Usage

> [!WARNING]
> Sherwood is a work in progress and the documentation may be incomplete or incorrect. The following commands for example simply do not exist in the Sherwood CLI _yet_.

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
