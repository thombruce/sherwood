//! Shared markdown parsing configuration
//!
//! This module provides centralized ParseOptions for consistent markdown parsing
//! across the Sherwood codebase, eliminating duplication.

use markdown::ParseOptions;

/// Creates ParseOptions with frontmatter support enabled
///
/// This is the most common configuration used throughout the codebase
/// for parsing markdown content that contains frontmatter.
pub fn with_frontmatter() -> ParseOptions {
    ParseOptions {
        constructs: markdown::Constructs {
            frontmatter: true,
            ..Default::default()
        },
        ..ParseOptions::default()
    }
}

/// Creates ParseOptions with frontmatter and GFM strikethrough support
///
/// This configuration is used when parsing markdown that may contain
/// GitHub-flavored markdown strikethrough syntax.
pub fn with_frontmatter_and_gfm() -> ParseOptions {
    ParseOptions {
        constructs: markdown::Constructs {
            frontmatter: true,
            gfm_strikethrough: true,
            ..Default::default()
        },
        ..ParseOptions::default()
    }
}

/// Creates default ParseOptions for basic markdown parsing
///
/// Use this when you need standard markdown parsing without frontmatter
/// or other specific constructs.
pub fn default() -> ParseOptions {
    ParseOptions::default()
}
