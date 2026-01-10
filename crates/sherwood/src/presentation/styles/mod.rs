//! CSS processing and style management for Sherwood
//!
//! This module provides functionality for processing, bundling, and managing CSS files
//! with support for modern browser targets, minification, and embedded styles.

pub mod browser_targets;
pub mod embedded_files;
pub mod manager;
pub mod processor;
pub mod validation;

// Re-export public types for backward compatibility
pub use browser_targets::{get_default_browser_targets, parse_css_targets};
pub use embedded_files::STYLES;
pub use manager::StyleManager;
pub use processor::CssProcessor;
pub use validation::{
    EntryPointValidationError, resolve_and_validate_entry_point, validate_css_entry_point,
};
