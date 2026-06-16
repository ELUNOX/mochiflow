//! MANIFEST.json types.
//!
//! The engine MANIFEST records sha256 hashes for integrity checking (doctor drift).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("MANIFEST.json not found: {0}")]
    NotFound(String),
    #[error("MANIFEST.json parse error: {0}")]
    Parse(String),
}

/// MANIFEST.json structure.
#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub files: BTreeMap<String, String>,
}

/// Load MANIFEST.json from the engine directory.
pub fn load_manifest(engine_dir: &Path) -> Result<Manifest, ManifestError> {
    let path = engine_dir.join("MANIFEST.json");
    if !path.exists() {
        return Err(ManifestError::NotFound(path.display().to_string()));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| ManifestError::Parse(format!("cannot read: {e}")))?;
    serde_json::from_str(&text).map_err(|e| ManifestError::Parse(e.to_string()))
}

/// Read the installed engine VERSION file from an engine directory.
pub fn read_engine_version(engine_dir: &Path) -> Result<String, ManifestError> {
    let path = engine_dir.join("VERSION");
    if !path.exists() {
        return Err(ManifestError::NotFound(path.display().to_string()));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| ManifestError::Parse(format!("cannot read VERSION: {e}")))?;
    let version = text.trim();
    if version.is_empty() {
        return Err(ManifestError::Parse("VERSION is empty".to_string()));
    }
    Ok(version.to_string())
}

/// Best-effort version label for user-facing output when the installed engine is missing.
pub fn engine_version_label(engine_dir: &Path) -> String {
    read_engine_version(engine_dir).unwrap_or_else(|_| "unknown".to_string())
}
