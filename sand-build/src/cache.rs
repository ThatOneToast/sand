use std::path::PathBuf;

use crate::error::{Error, Result};

/// Returns `~/.sand/cache/`.
pub fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or(Error::NoHomeDir)?;
    Ok(home.join(".sand").join("cache"))
}

/// Returns `~/.sand/cache/<version>/`.
pub fn version_dir(version_id: &str) -> Result<PathBuf> {
    Ok(cache_dir()?.join(version_id))
}

/// Ensures the directory exists, creating it if necessary.
pub fn ensure_dir(path: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}
