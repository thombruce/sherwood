use crate::config::CssSection;

#[derive(Debug)]
pub enum EntryPointValidationError {
    Empty,
    ContainsPathSeparators,
    MissingExtension,
    InvalidExtension,
}

impl std::fmt::Display for EntryPointValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryPointValidationError::Empty => write!(f, "cannot be empty"),
            EntryPointValidationError::ContainsPathSeparators => {
                write!(f, "must be a simple filename, not a path")
            }
            EntryPointValidationError::MissingExtension => write!(f, "must have a file extension"),
            EntryPointValidationError::InvalidExtension => write!(f, "must end with .css"),
        }
    }
}

pub fn validate_css_entry_point(entry_point: &str) -> Result<String, EntryPointValidationError> {
    if entry_point.is_empty() {
        return Err(EntryPointValidationError::Empty);
    }

    if entry_point.contains('/') || entry_point.contains('\\') {
        return Err(EntryPointValidationError::ContainsPathSeparators);
    }

    if !entry_point.contains('.') {
        return Err(EntryPointValidationError::MissingExtension);
    }

    if !entry_point.ends_with(".css") {
        return Err(EntryPointValidationError::InvalidExtension);
    }

    Ok(entry_point.to_string())
}

pub fn resolve_and_validate_entry_point(css_config: Option<&CssSection>) -> String {
    if let Some(css_config) = css_config
        && let Some(entry_point) = &css_config.entry_point
    {
        match validate_css_entry_point(entry_point) {
            Ok(validated) => validated,
            Err(error) => {
                eprintln!(
                    "⚠️  Warning: Invalid CSS entry point '{}': {}. Using default 'main.css'.",
                    entry_point, error
                );
                "main.css".to_string()
            }
        }
    } else {
        "main.css".to_string()
    }
}
