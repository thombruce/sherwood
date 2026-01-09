+++
title = "CLI Commands"
date = "2024-01-19"
page_template = "docs.stpl"
+++

# CLI Commands

Sherwood provides a command-line interface for site generation, development, and validation. All commands use consistent patterns and provide helpful feedback.

## Available Commands

### `generate`

Generates a static site from Markdown content.

```bash
cargo run --bin generate -- [OPTIONS]
```

**Options:**
- `--input <DIR>`: Source directory containing Markdown files (default: `content`)
- `--output <DIR>`: Output directory for generated HTML (default: `dist`)

**Examples:**
```bash
# Use default directories (content → dist)
cargo run --bin generate

# Custom input/output directories
cargo run --bin generate -- --input src --output build

# Using binary directly if installed
sherwood generate --input content --output dist
```

**Process:**
1. Scans input directory for `.md` files
2. Parses Markdown with frontmatter
3. Extracts titles and metadata
4. Renders HTML using templates
5. Bundles and processes CSS
6. Generates file structure in output directory

### `dev`

Starts development server with live reload functionality.

```bash
cargo run --bin dev -- [OPTIONS]
```

**Options:**
- `--input <DIR>`: Source directory containing Markdown files (default: `content`)
- `--output <DIR>`: Output directory for generated HTML (default: `dist`)
- `--port <PORT>`: Server port (default: `3000`)
- `--host <HOST>`: Server host (default: `localhost`)

**Features:**
- **Live Reload**: Automatically regenerates site when content changes
- **File Watching**: Monitors input directory for changes
- **Development Server**: Serves generated files with proper headers
- **Error Display**: Shows build errors in browser overlay

**Examples:**
```bash
# Default development server (localhost:3000)
cargo run --bin dev

# Custom port and directories
cargo run --bin dev -- --port 8080 --input src --output build

# External access (for mobile testing)
cargo run --bin dev -- --host 0.0.0.0 --port 3000
```

### `validate`

Validates configuration and content without generating files.

```bash
cargo run --bin validate -- [OPTIONS]
```

**Options:**
- `--input <DIR>`: Source directory to validate (default: `content`)
- `--output <DIR>`: Output directory for validation checks (default: `dist`)

**Validates:**
- ✅ Frontmatter syntax and fields
- ✅ Markdown parsing errors
- ✅ Template file existence
- ✅ CSS configuration validity
- ✅ Directory structure consistency

**Examples:**
```bash
# Validate default content directory
cargo run --bin validate

# Validate custom source
cargo run --bin validate -- --input src/content
```

## Common Usage Patterns

### Development Workflow

```bash
# 1. Start development server
cargo run --bin dev

# 2. In another terminal, validate changes
cargo run --bin validate

# 3. When ready, generate production build
cargo run --bin generate
```

### Custom Project Structure

```bash
# Project with custom directories
cargo run --bin dev -- --input docs --output public

# CI/CD pipeline validation
cargo run --bin validate -- --input docs --output public
cargo run --bin generate -- --input docs --output public
```

### Production Build

```bash
# Clean build directory
rm -rf dist/

# Generate production site
cargo run --bin generate

# Optional: Optimize output
find dist -name "*.html" -exec gzip -k {} \;
find dist -name "*.css" -exec gzip -k {} \;
```

## Command Aliases

You can create shell aliases for convenience:

**Bash/Zsh:**
```bash
# Add to ~/.bashrc or ~/.zshrc
alias sherwood='cargo run --bin'
alias sherwood-dev='cargo run --bin dev'
alias sherwood-build='cargo run --bin generate'
alias sherwood-validate='cargo run --bin validate'
```

**Fish Shell:**
```fish
# Add to ~/.config/fish/config.fish
alias sherwood 'cargo run --bin'
alias sherwood-dev 'cargo run --bin dev'
alias sherwood-build 'cargo run --bin generate'
alias sherwood-validate 'cargo run --bin validate'
```

## Exit Codes

Sherwood returns specific exit codes for automation:

- `0`: Success
- `1`: General error (parse error, file not found)
- `2`: Validation error (invalid configuration)
- `3`: Build error (template missing, syntax error)

**Using in Scripts:**
```bash
#!/bin/bash
cargo run --bin validate
case $? in
  0) echo "✅ Validation passed"
     cargo run --bin generate
     ;;
  1) echo "❌ Error occurred"
     exit 1
     ;;
  2) echo "❌ Validation failed"
     exit 1
     ;;
esac
```

## Environment Variables

Sherwood respects several environment variables:

- `RUST_LOG`: Set logging level (`debug`, `info`, `warn`, `error`)
- `SHERWOOD_INPUT`: Default input directory
- `SHERWOOD_OUTPUT`: Default output directory
- `SHERWOOD_PORT`: Default server port
- `SHERWOOD_HOST`: Default server host

**Examples:**
```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin dev

# Set custom defaults
export SHERWOOD_INPUT=src/docs
export SHERWOOD_OUTPUT=public
cargo run --bin generate
```

## Performance Considerations

### Build Performance

```bash
# Enable release mode for production builds
cargo build --release

# Use binary directly after initial build
./target/release/sherwood generate

# Parallel processing (future feature)
# cargo run --bin generate -- --parallel
```

### Development Performance

```bash
# Limit file watching to specific directories (future feature)
# cargo run --bin dev -- --watch-only content

# Exclude certain file patterns (future feature)
# cargo run --bin dev -- --exclude "*.tmp" --exclude "node_modules/*"
```

## Troubleshooting

### Common Issues

**"No such file or directory"**
```bash
# Check current directory and files
ls -la
ls content/

# Use explicit paths
cargo run --bin generate -- --input ./content --output ./dist
```

**"Permission denied"**
```bash
# Check directory permissions
ls -ld content/ dist/

# Fix permissions
chmod 755 content/ dist/
chmod 644 content/*.md
```

**"Frontmatter parse error"**
```bash
# Validate specific files
cargo run --bin validate -- --input content/problem-file.md

# Check YAML/TOML syntax
# YAML:
python -c "import yaml; yaml.safe_load(open('content/file.md', 'r').read())"

# TOML:
python -c "import toml; toml.load(open('content/file.md', 'r').read())"
```

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin dev 2>&1 | tee sherwood.log

# Parse-specific file
RUST_LOG=debug cargo run --bin validate -- --input content/specific-file.md
```

## Integration Examples

### Makefile Integration

```makefile
.PHONY: dev build validate clean

dev:
	cargo run --bin dev

build:
	cargo run --bin generate

validate:
	cargo run --bin validate

clean:
	rm -rf dist/

deploy: validate build
	# Deployment commands here
```

### npm Scripts Integration

```json
{
  "scripts": {
    "dev": "cargo run --bin dev",
    "build": "cargo run --bin generate",
    "validate": "cargo run --bin validate",
    "clean": "rm -rf dist/",
    "deploy": "npm run validate && npm run build"
  }
}
```

### GitHub Actions

```yaml
name: Build Site
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Validate
        run: cargo run --bin validate
      - name: Build
        run: cargo run --bin generate
```

This CLI reference provides everything needed to effectively use Sherwood's command-line interface for development and production workflows.