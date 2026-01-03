use crate::presentation::styles::CssProcessor;
use anyhow::Result;
use lightningcss::stylesheet::{MinifyOptions, PrinterOptions, StyleSheet};
use std::collections::HashSet;

/// Apply minification to a stylesheet if enabled
pub fn apply_minification(stylesheet: &mut StyleSheet, processor: &CssProcessor) -> Result<()> {
    if processor.minify {
        let minify_options = MinifyOptions {
            targets: processor.targets,
            #[allow(clippy::if_same_then_else)]
            unused_symbols: if processor.remove_unused {
                HashSet::new() // Remove all unused symbols
            } else {
                HashSet::new() // Default empty set
            },
        };
        stylesheet
            .minify(minify_options)
            .map_err(|e| anyhow::anyhow!("Failed to minify CSS: {}", e))?;
    }
    Ok(())
}

/// Serialize a stylesheet to CSS string
pub fn serialize_stylesheet(
    stylesheet: &StyleSheet,
    processor: &CssProcessor,
    filename: &str,
) -> Result<String> {
    let result = stylesheet
        .to_css(PrinterOptions {
            minify: processor.minify,
            source_map: None, // Will handle source maps separately
            targets: processor.targets,
            ..PrinterOptions::default()
        })
        .map_err(|e| anyhow::anyhow!("Failed to serialize CSS from {}: {}", filename, e))?;

    // TODO: Implement proper source map generation when Lightning CSS API supports it
    // For now, source maps are not generated due to API limitations
    if processor.source_maps {
        eprintln!("⚠️  Source maps requested but not yet implemented in Lightning CSS integration");
    }

    Ok(result.code)
}
