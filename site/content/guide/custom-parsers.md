---
title: Custom Parsers
---

# Custom Parsers

Sherwood parses content through a small trait, so you can teach it formats
beyond the built-in Markdown. A parser turns one file's raw source into a
`Parsed` payload; it never deals with paths or URLs.

```rust
use std::path::Path;
use sherwood::{ContentParser, FrontMatter, Parsed, ParserError, Pod};

struct ShoutParser;

impl ContentParser for ShoutParser {
    fn extensions(&self) -> &[&str] {
        &["shout"]
    }

    fn parse(&self, source: &str, _path: &Path) -> Result<Parsed, ParserError> {
        let mut lines = source.lines();
        let title = lines.next().unwrap_or("").to_string();
        let body = lines.collect::<Vec<_>>().join(" ").to_uppercase();
        Ok(Parsed {
            frontmatter: FrontMatter { title, data: Pod::Null },
            content_html: format!("<p>{body}</p>"),
            excerpt_html: None,
        })
    }
}
```

Register it on a `ParserRegistry` and pass that to the build:

```rust
use std::sync::Arc;
use sherwood::ParserRegistry;

let mut registry = ParserRegistry::default(); // markdown built in
registry.register(Arc::new(ShoutParser));
```

Now any `.shout` file becomes a page. Files whose extension no parser claims
are copied verbatim to the mirrored output path, so assets (images, downloads)
can sit in the content tree next to the pages that use them.

## Reusing frontmatter

If your format uses the same `---` / `+++` frontmatter convention as Markdown,
call the public `split_frontmatter` helper instead of reinventing it:

```rust
let (frontmatter, body) = sherwood::split_frontmatter(source)?;
```

Formats with their own metadata convention — like `ShoutParser` above, which
takes the title from the first line — simply ignore it.
