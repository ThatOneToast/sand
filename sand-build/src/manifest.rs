use crate::cache::{CacheLock, invalid, read_optional, validate_version, write_atomic};
use crate::download::{Transport, digest};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct VersionManifest {
    pub(crate) versions: Vec<VersionEntry>,
}

/// Canonical Mojang descriptor; preserve ID, URL and expected package digest
/// together from resolution through package verification.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct VersionEntry {
    pub(crate) id: String,
    pub(crate) url: String,
    pub(crate) sha1: String,
}

impl VersionManifest {
    pub(crate) fn resolve(&self, concrete: &str) -> Result<VersionEntry> {
        let mut entry = self
            .versions
            .iter()
            .find(|v| v.id == concrete)
            .cloned()
            .ok_or_else(|| Error::UnknownVersion(concrete.to_owned()))?;
        validate_version(&entry.id)?;
        entry.sha1 = digest(&entry.sha1)?;
        Ok(entry)
    }
}

/// Pinned cache preference only. Public `latest` is resolved to the bundled
/// anchor before this function. There is no moving-latest or offline policy API.
/// New writers never mutate the legacy manifest or its shared .tmp pathname.
pub(crate) fn resolve(root: &Path, concrete: &str, http: &impl Transport) -> Result<VersionEntry> {
    validate_version(concrete)?;
    let _lock = CacheLock::acquire(&root.join(".sand-manifest-v1.lock"))?;
    let path = root.join("download-manifest-v1.json");
    let current = read_optional(&path)?;
    if let Some(bytes) = &current
        && let Ok(manifest) = serde_json::from_slice::<VersionManifest>(bytes)
        && let Ok(entry) = manifest.resolve(concrete)
    {
        return Ok(entry);
    }
    // Import once, only when our manifest is absent. A newer manifest must not
    // fall back to a different legacy identity when it becomes corrupt.
    if current.is_none()
        && let Some(bytes) = read_optional(&root.join("version_manifest_v2.json"))?
        && let Ok(manifest) = serde_json::from_slice::<VersionManifest>(&bytes)
        && let Ok(entry) = manifest.resolve(concrete)
    {
        write_atomic(&path, &bytes)?;
        return Ok(entry);
    }
    let bytes = http
        .get(MANIFEST_URL)?
        .checked(MANIFEST_URL)?
        .bytes(MANIFEST_URL)?;
    let manifest: VersionManifest = serde_json::from_slice(&bytes)?;
    if manifest.versions.is_empty() {
        return Err(invalid("empty Minecraft version manifest"));
    }
    let entry = manifest.resolve(concrete)?;
    write_atomic(&path, &bytes)?;
    Ok(entry)
}

#[cfg(test)]
mod tests;
