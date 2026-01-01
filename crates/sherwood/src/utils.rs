use anyhow::Result;
use std::fs;
use std::path::Path;

/// Creates a directory and all necessary parent directories with better error handling
pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            Ok(())
        } else {
            anyhow::bail!("Path '{}' exists but is not a directory", path.display());
        }
    } else {
        fs::create_dir_all(path)
            .map_err(|e| anyhow::anyhow!("Failed to create directory '{}': {}", path.display(), e))
    }
}

/// Ensures the parent directory of a file path exists
pub fn ensure_parent_exists(file_path: &Path) -> Result<()> {
    if let Some(parent) = file_path.parent() {
        ensure_directory_exists(parent)?;
    }
    Ok(())
}

/// Utility function to create multiple directories at once
pub fn ensure_directories_exist(paths: &[&Path]) -> Result<()> {
    for path in paths {
        ensure_directory_exists(path)?;
    }
    Ok(())
}
