// Fields on Mojang API structs are part of the public API shape; not all are
// consumed within this crate itself yet.
#![allow(dead_code)]

use serde::Deserialize;

use crate::{
    cache::{cache_dir, ensure_dir},
    error::{Error, Result},
};

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    /// URL to the version-specific package JSON (contains server jar URL).
    pub url: String,
    pub sha1: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

impl VersionManifest {
    /// Load from `~/.sand/cache/version_manifest_v2.json`, fetching from Mojang
    /// if not cached. If the requested version is not found in the cache, the
    /// manifest is re-fetched once to pick up new releases.
    pub fn fetch_or_cached(version_str: &str) -> Result<Self> {
        let cache_path = cache_dir()?.join("version_manifest_v2.json");

        if cache_path.exists() {
            let content = std::fs::read_to_string(&cache_path)?;
            if let Ok(manifest) = serde_json::from_str::<VersionManifest>(&content) {
                // If the version is present (or "latest"), return without re-fetching.
                let target = if version_str == "latest" {
                    &manifest.latest.release
                } else {
                    version_str
                };
                if manifest.versions.iter().any(|v| v.id == *target) {
                    return Ok(manifest);
                }
                // Version not found in cached manifest — fall through to re-fetch.
            }
        }

        fetch_and_cache(&cache_path)
    }

    /// Resolve a version string to a `VersionEntry`.
    ///
    /// Accepts `"latest"` (maps to latest release) or an explicit version id
    /// such as `"1.21.4"`. Returns an error for unknown versions.
    pub fn resolve(&self, version_str: &str) -> Result<&VersionEntry> {
        let target = if version_str == "latest" {
            self.latest.release.as_str()
        } else {
            version_str
        };

        self.versions
            .iter()
            .find(|v| v.id == target)
            .ok_or_else(|| Error::UnknownVersion(version_str.to_string()))
    }
}

fn fetch_and_cache(cache_path: &std::path::Path) -> Result<VersionManifest> {
    let response = reqwest::blocking::get(MANIFEST_URL)?;
    let content = response.text()?;

    if let Some(parent) = cache_path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }
    std::fs::write(cache_path, &content)?;

    Ok(serde_json::from_str(&content)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manifest(versions: &[(&str, &str)]) -> VersionManifest {
        VersionManifest {
            latest: LatestVersions {
                release: versions[0].0.to_string(),
                snapshot: versions[0].0.to_string(),
            },
            versions: versions
                .iter()
                .map(|(id, _url)| VersionEntry {
                    id: id.to_string(),
                    version_type: "release".to_string(),
                    url: _url.to_string(),
                    sha1: "deadbeef".to_string(),
                    time: String::new(),
                    release_time: String::new(),
                })
                .collect(),
        }
    }

    #[test]
    fn resolve_explicit_version() {
        let m = make_manifest(&[("1.21.4", "http://example.com/1.21.4.json")]);
        let entry = m.resolve("1.21.4").unwrap();
        assert_eq!(entry.id, "1.21.4");
    }

    #[test]
    fn resolve_latest() {
        let m = make_manifest(&[("1.21.11", "http://example.com/1.21.11.json")]);
        let entry = m.resolve("latest").unwrap();
        assert_eq!(entry.id, "1.21.11");
    }

    #[test]
    fn resolve_unknown_errors() {
        let m = make_manifest(&[("1.21.4", "http://example.com/1.21.4.json")]);
        let err = m.resolve("9.99.99").unwrap_err();
        assert!(matches!(err, Error::UnknownVersion(_)));
    }
}
