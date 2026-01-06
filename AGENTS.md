# Sherwood Development Guidelines for AI Agents

This document provides comprehensive guidelines for AI agents working on the Sherwood static site generator codebase. Following these guidelines ensures consistency, security, and maintainability across all contributions.

## Quick Reference Commands

### Build & Test Commands
```bash
# Build the project
cargo build

# Check formatting
cargo fmt --check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests (when available)
cargo test

# Run single integration test
cargo run --bin generate

# Run development server
cargo run --bin dev --input content --output dist

# Validate configuration
cargo run --bin validate --input content --output dist
```

### Project Structure
```
sherwood/
├── crates/sherwood/
│   ├── src/
│   │   ├── bin/           # CLI applications
│   │   ├── config/         # Configuration structures
│   │   ├── content/        # Content processing
│   │   ├── core/           # Core utilities
│   │   ├── presentation/    # CSS and templates
│   │   └── generator.rs     # Site generation orchestration
│   ├── styles/             # Embedded CSS files
│   ├── templates/           # Embedded templates
│   └── Cargo.toml
└── docs/                  # Working Sherwood site (documentation + testing)
```

## Code Style Guidelines

### Imports & Dependencies

**Import Organization:**
```rust
// Standard library imports first
use std::fs;
use std::path::{Path, PathBuf};

// External crates second
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};

// Local modules last
use crate::config::CssSection;
use super::css_processing;
```

**Dependency Management:**
- Add new dependencies to `crates/sherwood/Cargo.toml`
- Use specific features only when needed
- Prefer minimal, well-maintained crates
- Security-focused dependencies for file operations (e.g., `tempfile`)

### Naming Conventions

**Functions:**
```rust
// Public: snake_case with descriptive names
pub fn generate_site(input_dir: &Path, output_dir: &Path) -> Result<()>

// Private: snake_case, prefixed with purpose
fn apply_minification(stylesheet: &mut StyleSheet, processor: &CssProcessor) -> Result<()>
fn resolve_and_validate_entry_point(css_config: Option<&CssSection>) -> String
```

**Structs & Enums:**
```rust
// Public: PascalCase, descriptive
#[derive(Debug, Clone)]
pub struct CssProcessor {
    pub minify: bool,
    pub targets: Targets,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum EntryPointValidationError {
    Empty,
    ContainsPathSeparators,
    MissingExtension,
    InvalidExtension,
}
```

**Constants:**
```rust
// SCREAMING_SNAKE_CASE
const CONFIG_PATH_RELATIVE: &str = "../Sherwood.toml";
const DEFAULT_PAGE_TEMPLATE: &str = "default.stpl";

static STYLES: Dir = include_dir!("$CARGO_MANIFEST_DIR/styles");
```

### Type System & Error Handling

**Result Types:**
```rust
// Use anyhow::Result for application-level errors
use anyhow::Result;

pub fn process_css(content: &str) -> Result<String>

// Provide context for errors
.map_err(|e| anyhow::anyhow!("Failed to process CSS: {}", e))
```

**Error Messages:**
```rust
// User-friendly, actionable error messages
anyhow::anyhow!(
    "CSS entry point '{}' not found in styles directory '{}'.\n\
     Available files: {}\n\
     Fix: either create the file or remove the 'entry_point' configuration to use defaults.",
    entry_point,
    self.styles_dir.display(),
    self.list_available_css_files()?
);
```

**Validation Errors:**
```rust
// Use custom error types for domain-specific validation
#[derive(Debug)]
pub enum EntryPointValidationError {
    Empty,                    // Clear, one-word description
    ContainsPathSeparators,     // Descriptive of the issue
    MissingExtension,          // Specific about what's missing
    InvalidExtension,          // Clear about what's wrong
}

impl std::fmt::Display for EntryPointValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryPointValidationError::Empty => write!(f, "cannot be empty"),
            EntryPointValidationError::ContainsPathSeparators => write!(f, "must be a simple filename, not a path"),
            EntryPointValidationError::MissingExtension => write!(f, "must have a file extension"),
            EntryPointValidationError::InvalidExtension => write!(f, "must end with .css"),
        }
    }
}
```

### Security Guidelines

**File Operations:**
```rust
// Use secure temporary directories
use tempfile::tempdir;

let temp_dir = tempfile::tempdir()?;  // Automatic cleanup, secure creation
let temp_path = temp_dir.path();     // temp_dir cleans up when out of scope

// Path traversal prevention
if entry_point.contains('/') || entry_point.contains('\\') {
    return Err(EntryPointValidationError::ContainsPathSeparators);
}
```

**Input Validation:**
```rust
// Always validate user input
fn validate_css_entry_point(entry_point: &str) -> Result<String, EntryPointValidationError> {
    if entry_point.is_empty() {
        return Err(EntryPointValidationError::Empty);
    }
    
    if !entry_point.ends_with(".css") {
        return Err(EntryPointValidationError::InvalidExtension);
    }
    
    Ok(entry_point.to_string())
}
```

### Code Organization & Architecture

**Module Structure:**
```rust
// mod.rs - Module declarations
pub mod css_processing;
pub mod styles;
pub mod templates;

// Use super imports for sibling modules
use super::css_processing::{apply_minification, serialize_stylesheet};
```

**Shared Functions:**
```rust
// Create shared modules for common functionality
pub mod css_processing {
    pub fn apply_minification(stylesheet: &mut StyleSheet, processor: &CssProcessor) -> Result<()> {
        // Shared implementation
    }
    
    pub fn serialize_stylesheet(stylesheet: &StyleSheet, processor: &CssProcessor, filename: &str) -> Result<String> {
        // Shared implementation
    }
}
```

**Configuration Patterns:**
```rust
// Use Option<T> for optional configuration fields
#[derive(Debug, Deserialize, Serialize)]
pub struct CssSection {
    pub minify: Option<bool>,           // User can override
    pub targets: Option<CssTargets>,     // Optional advanced config
    pub source_maps: Option<bool>,        // Development vs production
}

// Provide sensible defaults
impl CssProcessor {
    pub fn from_config(css_config: &CssSection, is_development: bool) -> Self {
        Self {
            minify: css_config.minify.unwrap_or(!is_development),  // Default: !is_development
            targets: css_config.targets.as_ref()
                .map(parse_css_targets)
                .unwrap_or_else(get_default_browser_targets),  // Fallback to defaults
        }
    }
}
```

### Testing & Quality Assurance

**Development Workflow:**
```bash
# Always run before committing
cargo fmt --check    # Verify formatting
cargo clippy         # Check for linting issues
cargo build           # Ensure compilation
cargo test           # Run tests (when available)
```

**Functional Testing:**
```bash
# Test core functionality
cargo run --bin generate -- --input test_content --output test_dist

# Verify no temporary files left behind
find /tmp -name "*sherwood*" 2>/dev/null || echo "No temp files found"
```

**Error Testing:**
- Test with invalid configuration files
- Test with missing directories/files
- Test with malformed input
- Verify error messages are helpful

### Performance & Resource Management

**RAII Pattern:**
```rust
// Use types that automatically clean up
use tempfile::tempdir;

{
    let temp_dir = tempfile::tempdir()?;  // Automatically cleaned up
    // Do work with temp directory
} // temp_dir cleaned up here
```

**Memory Efficiency:**
```rust
// Prefer borrowing over cloning when possible
pub fn process_css_string(&self, content: &str, filename: &str) -> Result<String>
//                              ^ slice reference, not String

// Use references for large data
pub fn bundle_css_files(&self, entry_point: &Path, output_dir: &Path) -> Result<PathBuf>
```

**Async Patterns:**
```rust
// Use tokio for I/O operations
#[tokio::main]
async fn main() {
    if let Err(e) = generate_site(&cli.input, &cli.output).await {
        eprintln!("Error generating site: {}", e);
        std::process::exit(1);
    }
}
```

### CLI & Configuration

**Command Structure:**
```rust
// Use clap for consistent CLI
use clap::Parser;

#[derive(Parser)]
#[command(name = "generate")]
#[command(about = "Generate a static site from Markdown content")]
struct Cli {
    #[arg(short, long, default_value = "content")]
    input: PathBuf,
    
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,
}
```

**Configuration Files:**
```toml
# Use descriptive section names
[site]
# Site-level configuration

[css]
# CSS processing configuration
entry_point = "main.css"
minify = true
source_maps = false

[css.targets]
# Browser targeting for CSS optimization
chrome = "103"
firefox = "115"
```

### Documentation & Comments

**Inline Documentation:**
```rust
/// Process CSS content from a string and return the processed CSS string
/// 
/// # Arguments
/// * `content` - Raw CSS content to process
/// * `filename` - Source filename for error reporting
/// 
/// # Returns
/// Processed CSS string with minification and optimizations applied
/// 
/// # Errors
/// Returns error if CSS parsing or processing fails
pub fn process_css_string(&self, content: &str, filename: &str) -> Result<String>
```

**TODO Comments:**
```rust
// TODO: Implement proper source map generation when Lightning CSS API supports it
// For now, source maps are not generated due to API limitations
if processor.source_maps {
    eprintln!("⚠️  Source maps requested but not yet implemented");
}
```

### Git & Version Control

**Commit Messages:**
```
feat: Add configurable CSS entry point
fix: CSS bundling yields single file
refactor: Common functions to reduce code duplication
chore: Cleanup unused imports
docs: Update README for new CSS configuration
```

**Branch Naming:**
- `feature/css-processing` - New CSS features
- `fix/temp-file-security` - Bug fixes
- `refactor/code-deduplication` - Code quality improvements

## Important Notes

### CSS Processing Pipeline
- Uses Lightning CSS for modern CSS bundling and minification
- Secure temporary file handling with `tempfile` crate
- Shared processing functions in `css_processing.rs` module
- Entry point validation prevents path traversal attacks

### Configuration System
- TOML-based configuration with sensible defaults
- Development vs production modes
- Optional browser targeting for CSS optimization
- Graceful fallbacks for missing configuration

### Security Considerations
- Always validate user input for path traversal
- Use `tempfile` for secure temporary directories
- Provide clear error messages without exposing system paths
- Automatic resource cleanup to prevent leaks

### Performance Characteristics
- Lightning CSS provides fast bundling and minification
- Embedded CSS files for no external dependencies
- Efficient async I/O with tokio
- RAII patterns for resource management

### 📁 Testing Directory: `docs/`
The `docs/` folder contains a **working version of a Sherwood site** that uses the Sherwood library and binaries for testing purposes.

**Key Points:**
- **Working Site**: `docs/` demonstrates actual Sherwood usage and functionality
- **Local Development**: Uses `../crates/sherwood` relative path in its `Cargo.toml`
- **Test Commands**: Run `cargo run --bin dev` or `cargo run --bin generate` from within `docs/` directory
- **Configuration**: Uses local development crate, not published version
- **Purpose**: Serves as both documentation and functional test environment

**Usage for Agents:**
```bash
# Navigate to docs directory for testing
cd docs/

# Run development server
cargo run --bin dev --input content --output dist

# Generate static site
cargo run --bin generate --input content --output dist

# These commands use the local sherwood crate from ../crates/sherwood
```

**Note**: This makes `docs/` an ideal location for testing changes, examples, and reproducing issues in a live Sherwood environment.

### 🚨 CRITICAL RULE: Agents Must NEVER Commit Code
- **ABSOLUTELY NO COMMITS**: AI agents should only make suggestions, write files, or run commands for the user
- **NO EXCEPTIONS**: Even for "obvious" fixes or "urgent" changes
- **HUMAN-ONLY COMMITS**: Only human developers should commit changes to version control
- **REVIEW FIRST**: Always let the user review and approve changes before any consideration of commits
- **EXPLAIN ONLY**: Provide explanations, suggestions, and code proposals - let human decide and execute

**This rule exists to maintain version control integrity and ensure human oversight of all code changes.**

## Development Checklist

Before submitting changes:
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` passes without warnings
- [ ] `cargo build` succeeds
- [ ] Core functionality tested
- [ ] Error conditions tested
- [ ] Security validation performed
- [ ] Documentation updated if needed
- [ ] Breaking changes documented if applicable
- [ ] **NEVER COMMIT CODE** - Agents should only make suggestions, not commit changes

Follow these guidelines to maintain code quality, security, and consistency across the Sherwood codebase.

## Excerpt Support (v0.6.0)

Sherwood now supports excerpt extraction through frontmatter, providing both manual and automatic excerpt generation:

#### Frontmatter Field
The `excerpt` field can be specified in frontmatter:
```toml
+++
title = "My Article"
excerpt = "Custom summary for blog listings"
date = "2024-01-15"
+++
```

#### Automatic Excerpt Extraction
When no excerpt is provided, Sherwood automatically extracts the first paragraph from content:
- Strips all formatting (bold, italic, links, code, etc.)
- Uses AST for accurate paragraph detection
- Falls back gracefully when no paragraphs exist

#### Parser Support
All parsers support excerpt handling:
- **Markdown**: AST-based extraction with formatting stripping
- **JSON**: Manual excerpt + auto-extraction from content field
- **TOML**: Manual excerpt + auto-extraction from content field

#### Implementation Priority
1. **Frontmatter excerpt** (highest priority)
2. **Parser-extracted excerpt** (fallback)
3. **No excerpt** (if neither available)

#### Usage in Templates
Excerpts are passed to templates as `frontmatter.excerpt`:
- Available in blog listings and content summaries
- Rendered as plain text (no HTML formatting)
- None when no excerpt available

#### Excerpt Extraction Logic

**Markdown Parser:**
```rust
// AST-based extraction from first paragraph
fn extract_excerpt_from_markdown(&self, markdown: &str) -> Option<String> {
    let root = to_mdast(markdown, &self.parse_options).ok()?;
    self.extract_first_paragraph_from_ast(&root)
}
```

**Custom Parsers:**
```rust
// For markdown content
frontmatter.excerpt = markdown_parser.extract_excerpt_from_markdown(content);

// For plain text/HTML content  
frontmatter.excerpt = MarkdownParser::extract_excerpt_from_plain_text(content);
```

#### Testing Coverage
- Frontmatter excerpt parsing (TOML/YAML)
- Auto-extraction from markdown content
- Formatting stripping (bold, italic, links, code)
- Plain text extraction from non-markdown content
- Fallback behavior when no content/paragraphs exist
- Priority handling (frontmatter > extracted)

## Markdown Migration Notes

### Migration Completed (v0.5.0)
Sherwood successfully migrated from `gray_matter` + `pulldown-cmark` to the unified `markdown` crate:

**Before:**
```rust
// Dependencies
gray_matter = { version = "0.3", features = ["yaml", "toml"] }
pulldown-cmark = "0.13"

// Separate parsing steps
let toml_result = self.toml_matter.parse::<Frontmatter>(content)?;
let parser = Parser::new_ext(markdown, options);
```

**After:**
```rust
// Dependencies  
markdown = "1.0"

// Unified parsing with frontmatter support
let root = to_mdast(content, &self.parse_options)?;
let html = to_html_with_options(markdown, &options)?;
```

### Benefits
- **Simplified Dependencies**: One crate instead of two
- **Better Performance**: Single-pass parsing with AST
- **Enhanced Features**: Built-in frontmatter, better error messages
- **Future-proof**: More actively maintained, supports MDX extensions

### Future Enhancement Opportunities

The migration to the `markdown` crate with AST access enables several potential enhancements:

#### Table of Contents Generation
```rust
// Future: Auto-generate TOC from heading structure
fn generate_toc_from_mdast(root: &Root) -> String {
    // Extract headings from AST and generate links
}
```

#### Reading Time Estimation  
```rust
// Future: Estimate reading time from word count
fn estimate_reading_time(content: &str) -> String {
    let word_count = content.split_whitespace().count();
    format!("{} min read", (word_count / 200).max(1))
}
```

#### Enhanced Excerpt Generation
```rust
// Now implemented: Smart excerpt extraction using AST
fn extract_smart_excerpt(root: &Root, max_length: usize) -> String {
    // Use paragraph boundaries from AST instead of text parsing
}
```

#### Content Validation
```rust
// Future: Validate internal links and images
fn validate_content_links(root: &Root) -> Vec<ValidationWarning> {
    // Check all internal links reference valid files
}
```

**Note**: These are documented intentions for future development, not current requirements. The AST-based parsing provides the foundation for implementing these features efficiently.
