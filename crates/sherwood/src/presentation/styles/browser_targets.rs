use crate::config::CssTargets;
use lightningcss::targets::{Browsers, Targets};

pub fn parse_css_targets(css_targets: &CssTargets) -> Targets {
    let mut browsers = Browsers::default();

    // Parse individual browser versions
    if let Some(chrome) = &css_targets.chrome
        && let Ok(version) = parse_browser_version(chrome)
    {
        browsers.chrome = Some(version);
    }

    if let Some(firefox) = &css_targets.firefox
        && let Ok(version) = parse_browser_version(firefox)
    {
        browsers.firefox = Some(version);
    }

    if let Some(safari) = &css_targets.safari
        && let Ok(version) = parse_browser_version(safari)
    {
        browsers.safari = Some(version);
    }

    if let Some(edge) = &css_targets.edge
        && let Ok(version) = parse_browser_version(edge)
    {
        browsers.edge = Some(version);
    }

    // TODO: Parse browserslist string if provided
    // For now, fall back to defaults if browserslist is provided
    if css_targets.browserslist.is_some() {
        return get_default_browser_targets();
    }

    Targets {
        browsers: Some(browsers),
        ..Targets::default()
    }
}

fn parse_browser_version(version_str: &str) -> Result<u32, std::num::ParseIntError> {
    // Parse version like "103" or "103.0" to Lightning CSS format (version << 16)
    let parts: Vec<&str> = version_str.split('.').collect();
    let major: u32 = parts[0].parse()?;

    // Lightning CSS uses version in format: (major << 16) | (minor << 8) | patch
    let minor = if parts.len() > 1 {
        parts[1].parse().unwrap_or(0)
    } else {
        0
    };
    let patch = if parts.len() > 2 {
        parts[2].parse().unwrap_or(0)
    } else {
        0
    };

    Ok((major << 16) | (minor << 8) | patch)
}

pub fn get_default_browser_targets() -> Targets {
    // Target modern browsers for better CSS support
    let browsers = Browsers {
        chrome: Some(103 << 16),  // Chrome 103+
        firefox: Some(115 << 16), // Firefox 115+
        safari: Some(15 << 16),   // Safari 15+
        edge: Some(127 << 16),    // Edge 127+
        ..Browsers::default()
    };

    Targets {
        browsers: Some(browsers),
        ..Targets::default()
    }
}
