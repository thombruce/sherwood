+++
title = "Parsers"
date = "2024-01-10"
page_template = "docs.stpl"
+++

# Plugin System: Custom Content Parsers

Sherwood's plugin system allows extending content parsing beyond Markdown by creating custom parsers and registering them with file extensions.

## Overview

The plugin system uses **compile-time registration** for **zero runtime overhead** while enabling complete flexibility in content formats.

## Creating Custom Parsers

### Parser Structure

All parsers must implement the `ContentParser` trait:

```rust
use sherwood::plugins::{ContentParser, ParsedContent};
use sherwood::content::parser::Frontmatter;
use anyhow::Result;
use std::path::Path;
use std::collections::HashMap;

pub struct MyParser;

impl MyParser {
    pub fn new() -> Box<dyn ContentParser> {
        Box::new(Self)
    }
}

impl ContentParser for MyParser {
    fn name(&self) -> &'static str {
        "myformat"  // Unique identifier for error messages
    }

    fn parse(&self, content: &str, _path: &Path) -> Result<ParsedContent> {
        // 1. Parse your content format
        // 2. Extract title (optional - will fallback to filename)
        // 3. Map to Sherwood frontmatter
        // 4. Return structured result
        
        let frontmatter = Frontmatter {
            title: extract_title_from_content(content),
            date: extract_date_from_content(content),
            list: None,
            page_template: None,
            sort_by: None,
            sort_order: None,
            tags: extract_tags_from_content(content),
        };

        Ok(ParsedContent {
            title: String::new(), // Let filename fallback work
            frontmatter,
            content: process_content(content), // Your content transformation
            metadata: HashMap::new(),
        })
    }
}
```

### The `Box<dyn ContentParser>` Pattern

**Purpose**: Type erasure for heterogeneous storage

Each parser has different memory layout, but `Box<dyn ContentParser>` creates a uniform "handle":

```rust
// Different parser types:
TomlParser  // 24 bytes
JsonParser  // 32 bytes  
TextParser  // 16 bytes

// All become same type after boxing:
Box<dyn ContentParser>  // Same size for all, uniform interface
```

**Benefits**:
- **Runtime polymorphism**: Same interface, different implementations
- **Memory efficiency**: One allocation per parser, reused many times
- **Type safety**: Rust guarantees all boxed types implement `ContentParser`
- **Extensibility**: Unlimited parser types without registry changes

### Return Types Explained

#### `ParsedContent`
Standardized structure for all parsers:

```rust
pub struct ParsedContent {
    pub title: String,           // Document title (filename fallback)
    pub frontmatter: Frontmatter, // Sherwood frontmatter fields
    pub content: String,         // Processed content body
    pub metadata: HashMap<String, String>, // Custom parser data
}
```

#### `Frontmatter`
Common metadata fields across all parsers:

```rust
pub struct Frontmatter {
    pub title: Option<String>,      // Document title
    pub date: Option<String>,       // Publication date
    pub list: Option<bool>,        // Is this a list page?
    pub page_template: Option<String>, // Custom template
    pub sort_by: Option<String>,    // Sorting field
    pub sort_order: Option<String>,  // Sort direction
    pub tags: Option<Vec<String>>,   // Content tags
}
```

## Parser Registration

### Registration in `src/main.rs`

```rust
mod parsers;
use parsers::{TomlContentParser, JsonContentParser, TextContentParser, MyParser};
use sherwood::plugins::PluginRegistry;

#[tokio::main]
async fn main() {
    let plugin_registry = PluginRegistry::new()
        // Register parsers with primary extensions
        .register("toml", TomlContentParser::new(), "toml")
        .register("json", JsonContentParser::new(), "json")
        .register("text", TextContentParser::new(), "txt")
        .register("myformat", MyParser::new(), "myext")
        
        // Map additional extensions to existing parsers
        .map_extensions(&[
            ("conf", "toml"),     // .conf files use TOML parser
            ("config", "toml"),   // .config files use TOML parser
            ("schema", "json"),     // .schema files use JSON parser
            ("note", "text"),      // .note files use text parser
            ("myext2", "myformat"), // Additional extension for custom parser
        ]);

    let cli = sherwood::SherwoodCli::new("myapp", "My static site generator")
        .with_plugins(plugin_registry);

    if let Err(e) = cli.run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

### Module Exports in `src/parsers/mod.rs`

```rust
pub mod toml;
pub mod json;
pub mod txt;
pub mod my_parser;

// Re-export for convenience
pub use toml::TomlContentParser;
pub use json::JsonContentParser;
pub use txt::TextContentParser;
pub use my_parser::MyParser;
```

## File Organization

### Directory Structure

```
docs/
├── src/
│   ├── main.rs              # Plugin registration
│   └── parsers/
│       ├── mod.rs          # Module declarations & re-exports
│       ├── toml.rs         # TOML parser example
│       ├── json.rs         # JSON parser example
│       ├── txt.rs          # Text parser example
│       └── my_parser.rs    # Your custom parser
├── content/
│   ├── example.toml      # Will be processed by TOML parser
│   ├── data.json        # Will be processed by JSON parser
│   ├── notes.txt        # Will be processed by text parser
│   └── data.myext      # Will be processed by custom parser
└── Sherwood.toml           # Site configuration
```

### Registration Patterns

#### Single Extension
```rust
.register("toml", TomlContentParser::new(), "toml")
```

#### Multiple Extensions
```rust
.register("text", TextContentParser::new(), "txt")
    .map_extensions(&[
        ("note", "text"),
        ("readme", "text"),
        ("md", "text"),  // Override built-in parser
    ])
```

#### Conditional Registration
```rust
let mut registry = PluginRegistry::new();

if cfg!(feature = "yaml-support") {
    registry = registry.register("yaml", YamlParser::new(), "yaml");
}

if cfg!(feature = "csv-support") {
    registry = registry.register("csv", CsvParser::new(), "csv");
}
```

## Example Parsers

### TOML Parser (`toml.rs`)

Handles structured data with frontmatter fields + content body:

```toml
title = "TOML Content Page"
date = "2024-01-05"
description = "This is a TOML content file example"
content = "This is the main content of TOML file.\n\n## Subheading\n\nMore content here in markdown format."
```

### JSON Parser (`json.rs`)

Alternative structured data format:

```json
{
  "title": "JSON Content Page",
  "date": "2024-01-05",
  "description": "This is a JSON content file example",
  "content": "# JSON Content\n\nThis is the main content of JSON file.\n\n## Subheading\n\nMore content here in markdown format."
}
```

### Text Parser (`txt.rs`)

Pure document content with no frontmatter processing:

```text
API Documentation: Authentication Endpoints

This document describes the authentication API endpoints for our web service.

## Login Endpoint

POST /api/auth/login

### Request Body
```json
{
  "username": "string",
  "password": "string"
}
```

### Response
```json
{
  "token": "jwt_token_string",
  "expires_in": 3600
}
```
```

## Parser Implementation Strategies

### Data-First Parsers (TOML/JSON)

**Characteristics**:
- Structured data format
- Separate frontmatter and content
- Field validation and type safety

**Implementation Pattern**:
```rust
fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
    let parsed: serde_json::Value = serde_json::from_str(content)?;
    
    let frontmatter = Frontmatter {
        title: parsed.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
        date: parsed.get("date").and_then(|v| v.as_str()).map(|s| s.to_string()),
        // ... map other fields
    };

    Ok(ParsedContent {
        title: String::new(), // Filename fallback
        frontmatter,
        content: parsed
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        metadata: HashMap::new(),
    })
}
```

### Document Parsers (TXT/Markdown)

**Characteristics**:
- Unstructured text content
- Optional frontmatter markers
- Content preservation focus

**Implementation Pattern**:
```rust
fn parse(&self, content: &str, _path: &Path) -> Result<ParsedContent> {
    // No frontmatter parsing - entire file is content
    let frontmatter = Frontmatter::default(); // Empty frontmatter

    Ok(ParsedContent {
        title: String::new(), // Will be overridden by filename fallback
        frontmatter,
        content: content.to_string(), // Preserve exactly
        metadata: HashMap::new(),
    })
}
```

### Custom Format Parsers

**Characteristics**:
- Proprietary data format
- Custom transformation logic
- Domain-specific requirements

**Implementation Pattern**:
```rust
fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
    // Custom parsing logic for your format
    let parsed = parse_my_custom_format(content)?;
    
    // Map your format's fields to Sherwood frontmatter
    let frontmatter = Frontmatter {
        title: parsed.title,
        date: parsed.date,
        list: parsed.is_list_page,
        // ... map your format's metadata
    };

    Ok(ParsedContent {
        title: String::new(),
        frontmatter,
        content: transform_to_html(parsed.content),
        metadata: parsed.custom_metadata,
    })
}
```

## Testing Your Parser

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_parsing() {
        let parser = MyParser::new();
        let content = "your format content";
        let path = Path::new("test.myext");
        
        let result = parser.parse(content, path);
        assert!(result.is_ok());
        
        let parsed = result.unwrap();
        assert_eq!(parsed.title, "test");
        // More assertions for content, frontmatter...
    }

    #[test]
    fn test_error_handling() {
        let parser = MyParser::new();
        let invalid_content = "malformed content";
        let path = Path::new("test.myext");
        
        let result = parser.parse(invalid_content, path);
        assert!(result.is_err());
    }
}
```

### Integration Tests

```bash
# Create test file
echo "title: Test Content" > content/test.myext

# Run Sherwood
cargo run -- generate

# Check output
ls dist/test.html
cat dist/test.html
```

## Best Practices

### Design Principles

1. **Keep parsers focused**: One format per parser
2. **Handle errors gracefully**: Use `anyhow` for context
3. **Preserve content**: Don't over-process unless necessary
4. **Document format**: Include example files in `content/`
5. **Test thoroughly**: Unit + integration tests

### Error Handling

```rust
fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
    let parsed = parse_my_format(content)
        .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.display(), e))?;
    
    Ok(transform_to_parsed_content(parsed))
}
```

### Performance Considerations

1. **One allocation**: `Box::new(Self)` in `new()` method
2. **Efficient parsing**: Use streaming for large files if needed
3. **Memory reuse**: Parser instance reused for all files
4. **Error early**: Validate format before expensive operations

### Configuration Integration

```toml
# Sherwood.toml
[plugins]
# Future: plugin-specific configuration
[plugins.myformat]
enable_transforms = true
output_format = "html"
```

## Advanced Features

### Content Transformation

```rust
fn transform_content(content: &str) -> String {
    // Apply custom transformations:
    // - Auto-link URLs
    // - Syntax highlighting markers
    // - Smart paragraph detection
    // - Custom HTML injection
    content.to_string()
}
```

### Custom Metadata

```rust
fn create_metadata(parsed: &MyParsedFormat) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    metadata.insert("word_count".to_string(), parsed.word_count.to_string());
    metadata.insert("reading_time".to_string(), parsed.reading_time.to_string());
    metadata.insert("difficulty".to_string(), parsed.difficulty.to_string());
    metadata
}
```

### Template Integration

```rust
impl ContentParser for MyParser {
    fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
        let mut parsed = base_parse(content, path)?;
        
        // Specify custom template for this format
        parsed.frontmatter.page_template = Some("my_template.stpl".to_string());
        
        Ok(parsed)
    }
}
```

This plugin system provides **maximum flexibility** while maintaining **zero overhead** and **type safety**!
