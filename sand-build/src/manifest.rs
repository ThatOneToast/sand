// Fields on Mojang API structs are part of the public API shape; not all are
// consumed within this crate itself yet.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::{
    cache::{cache_dir, ensure_dir},
    error::{Error, Result},
};

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// Controls how `fetch_or_cached_with_policy` resolves the version manifest.
pub enum ManifestCachePolicy {
    /// Use the cached manifest if it contains the requested version; fetch only
    /// when the version is absent from cache. Safe for pinned versions.
    PreferCache,
    /// Always attempt a network refresh first. Falls back to the cached manifest
    /// with a warning if the network request fails. Use for `"latest"` so that
    /// stale cached releases are not returned indefinitely.
    RefreshLatest,
    /// Never make network requests. Returns an error if no cached manifest exists.
    OfflineOnly,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    /// Load the version manifest, choosing a cache policy based on `version_str`.
    ///
    /// - Pinned versions (e.g. `"1.21.4"`) use `PreferCache` for deterministic
    ///   builds: the cached manifest is returned as-is if it contains the version.
    /// - `"latest"` uses `RefreshLatest`: a network refresh is attempted first so
    ///   the cached `latest.release` is never returned stale indefinitely. If the
    ///   refresh fails, the cached manifest is used with a warning.
    pub fn fetch_or_cached(version_str: &str) -> Result<Self> {
        let policy = if version_str == "latest" {
            ManifestCachePolicy::RefreshLatest
        } else {
            ManifestCachePolicy::PreferCache
        };
        Self::fetch_or_cached_with_policy(version_str, policy)
    }

    /// Load the version manifest with an explicit cache policy.
    pub fn fetch_or_cached_with_policy(
        version_str: &str,
        policy: ManifestCachePolicy,
    ) -> Result<Self> {
        let cache_path = cache_dir()?.join("version_manifest_v2.json");
        fetch_or_cached_impl(version_str, policy, &cache_path, fetch_and_cache)
    }

    /// Fetch Mojang's version manifest and update the cache.
    ///
    /// Unlike [`VersionManifest::fetch_or_cached`], this does not return a
    /// cached manifest when the network refresh fails.
    pub fn fetch_fresh() -> Result<Self> {
        let cache_path = cache_dir()?.join("version_manifest_v2.json");
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

/// Core manifest resolution logic, parameterised over the fetch function so
/// tests can inject a mock without touching the network.
fn fetch_or_cached_impl<F>(
    version_str: &str,
    policy: ManifestCachePolicy,
    cache_path: &std::path::Path,
    fetcher: F,
) -> Result<VersionManifest>
where
    F: Fn(&std::path::Path) -> Result<VersionManifest>,
{
    match policy {
        ManifestCachePolicy::PreferCache => {
            if cache_path.exists()
                && let Ok(content) = std::fs::read_to_string(cache_path)
                && let Ok(manifest) = serde_json::from_str::<VersionManifest>(&content)
                && manifest.versions.iter().any(|v| v.id == version_str)
            {
                return Ok(manifest);
            }
            // Cache absent, unparseable, or version not found — fetch from network.
            fetcher(cache_path)
        }

        ManifestCachePolicy::RefreshLatest => {
            // Pre-load a valid cached manifest *before* calling the fetcher.
            // If the fetcher corrupts the cache file before failing (e.g. by
            // writing a bad HTTP error page), the in-memory copy is still valid.
            let cached_fallback = if cache_path.exists() {
                std::fs::read_to_string(cache_path)
                    .ok()
                    .and_then(|c| serde_json::from_str::<VersionManifest>(&c).ok())
            } else {
                None
            };

            match fetcher(cache_path) {
                Ok(manifest) => Ok(manifest),
                Err(fetch_err) => {
                    if let Some(manifest) = cached_fallback {
                        eprintln!(
                            "warning: failed to refresh Minecraft version manifest \
                             ({fetch_err}); falling back to cached manifest — \
                             `latest` may be stale"
                        );
                        Ok(manifest)
                    } else {
                        Err(fetch_err)
                    }
                }
            }
        }

        ManifestCachePolicy::OfflineOnly => {
            if cache_path.exists() {
                let content = std::fs::read_to_string(cache_path)?;
                Ok(serde_json::from_str(&content)?)
            } else {
                Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "no cached manifest at '{}'; OfflineOnly policy disallows network access",
                        cache_path.display()
                    ),
                )))
            }
        }
    }
}

fn fetch_and_cache(cache_path: &std::path::Path) -> Result<VersionManifest> {
    let response = reqwest::blocking::get(MANIFEST_URL)?;
    let content = response.text()?;

    // Parse first — only write to cache after confirming the response is valid.
    // This prevents a truncated or non-JSON HTTP error page from overwriting a
    // previously valid cached manifest.
    let manifest = serde_json::from_str::<VersionManifest>(&content)?;

    if let Some(parent) = cache_path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }
    // Atomic write via temp-file + rename so the cache is never left in a
    // partially-written state if the process is killed mid-write.
    let tmp_path = cache_path.with_extension("json.tmp");
    std::fs::write(&tmp_path, &content)?;
    std::fs::rename(&tmp_path, cache_path)?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manifest(latest_release: &str, versions: &[(&str, &str)]) -> VersionManifest {
        VersionManifest {
            latest: LatestVersions {
                release: latest_release.to_string(),
                snapshot: latest_release.to_string(),
            },
            versions: versions
                .iter()
                .map(|(id, url)| VersionEntry {
                    id: id.to_string(),
                    version_type: "release".to_string(),
                    url: url.to_string(),
                    sha1: "deadbeef".to_string(),
                    time: String::new(),
                    release_time: String::new(),
                })
                .collect(),
        }
    }

    fn write_manifest_to(dir: &std::path::Path, manifest: &VersionManifest) {
        let content = serde_json::to_string(manifest).unwrap();
        std::fs::write(dir.join("version_manifest_v2.json"), content).unwrap();
    }

    // -------------------------------------------------------------------------
    // resolve() tests (no I/O)
    // -------------------------------------------------------------------------

    #[test]
    fn resolve_explicit_version() {
        let m = make_manifest("1.21.4", &[("1.21.4", "http://example.com/1.21.4.json")]);
        let entry = m.resolve("1.21.4").unwrap();
        assert_eq!(entry.id, "1.21.4");
    }

    #[test]
    fn resolve_latest() {
        let m = make_manifest("1.21.11", &[("1.21.11", "http://example.com/1.21.11.json")]);
        let entry = m.resolve("latest").unwrap();
        assert_eq!(entry.id, "1.21.11");
    }

    #[test]
    fn resolve_unknown_errors() {
        let m = make_manifest("1.21.4", &[("1.21.4", "http://example.com/1.21.4.json")]);
        let err = m.resolve("9.99.99").unwrap_err();
        assert!(matches!(err, Error::UnknownVersion(_)));
    }

    // -------------------------------------------------------------------------
    // fetch_or_cached_impl() tests (mock fetcher, no network)
    // -------------------------------------------------------------------------

    /// PreferCache: cached manifest contains the requested version → use cache,
    /// do not call the fetcher.
    #[test]
    fn prefer_cache_uses_cache_when_version_present() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("version_manifest_v2.json");

        let cached = make_manifest("1.21.4", &[("1.21.4", "http://example.com/1.21.4.json")]);
        write_manifest_to(dir.path(), &cached);

        let fetcher_called = std::sync::atomic::AtomicBool::new(false);
        let fetcher = |_: &std::path::Path| -> Result<VersionManifest> {
            fetcher_called.store(true, std::sync::atomic::Ordering::SeqCst);
            panic!("fetcher must not be called for a cached pinned version");
        };

        let manifest = fetch_or_cached_impl(
            "1.21.4",
            ManifestCachePolicy::PreferCache,
            &cache_path,
            fetcher,
        )
        .unwrap();
        assert_eq!(manifest.latest.release, "1.21.4");
        assert!(!fetcher_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    /// RefreshLatest: fetcher returns a newer manifest → `latest` resolves to
    /// the newer release, not the stale cached one.
    #[test]
    fn refresh_latest_uses_fetched_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("version_manifest_v2.json");

        let stale = make_manifest("1.20.0", &[("1.20.0", "http://example.com/1.20.0.json")]);
        write_manifest_to(dir.path(), &stale);

        let fresh = make_manifest(
            "1.21.4",
            &[
                ("1.21.4", "http://example.com/1.21.4.json"),
                ("1.20.0", "http://example.com/1.20.0.json"),
            ],
        );
        let fetcher = move |_: &std::path::Path| -> Result<VersionManifest> { Ok(fresh.clone()) };

        let manifest = fetch_or_cached_impl(
            "latest",
            ManifestCachePolicy::RefreshLatest,
            &cache_path,
            fetcher,
        )
        .unwrap();
        assert_eq!(manifest.latest.release, "1.21.4");
        let entry = manifest.resolve("latest").unwrap();
        assert_eq!(entry.id, "1.21.4");
    }

    /// RefreshLatest fallback: fetcher fails but cache exists → use cache with
    /// a warning (observable as Ok result containing the cached manifest).
    #[test]
    fn refresh_latest_falls_back_to_cache_on_network_error() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("version_manifest_v2.json");

        let cached = make_manifest("1.20.0", &[("1.20.0", "http://example.com/1.20.0.json")]);
        write_manifest_to(dir.path(), &cached);

        let fetcher = |_: &std::path::Path| -> Result<VersionManifest> {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "simulated network error",
            )))
        };

        let manifest = fetch_or_cached_impl(
            "latest",
            ManifestCachePolicy::RefreshLatest,
            &cache_path,
            fetcher,
        )
        .unwrap();
        // Fell back to cached manifest.
        assert_eq!(manifest.latest.release, "1.20.0");
    }

    /// Regression: if the fetcher writes invalid content to the cache file before
    /// failing (simulating a truncated or non-JSON HTTP error response), the
    /// pre-loaded in-memory copy of the valid manifest is returned and the caller
    /// receives a correct result even though the on-disk file is now corrupted.
    #[test]
    fn refresh_latest_bad_response_preserves_existing_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("version_manifest_v2.json");

        let valid = make_manifest("1.20.0", &[("1.20.0", "http://example.com/1.20.0.json")]);
        write_manifest_to(dir.path(), &valid);

        // Simulate a fetcher that corrupts the cache file (e.g. writes an HTTP
        // error page) before returning an error — the classic parse-after-write bug.
        let cache_path_clone = cache_path.clone();
        let fetcher = move |_: &std::path::Path| -> Result<VersionManifest> {
            std::fs::write(&cache_path_clone, b"<html>502 Bad Gateway</html>").unwrap();
            Err(Error::Json(
                serde_json::from_str::<VersionManifest>("<bad>").unwrap_err(),
            ))
        };

        let result = fetch_or_cached_impl(
            "latest",
            ManifestCachePolicy::RefreshLatest,
            &cache_path,
            fetcher,
        )
        .unwrap();

        // The pre-loaded in-memory manifest is returned despite the corrupted file.
        assert_eq!(result.latest.release, "1.20.0");

        // Confirm the disk file was actually corrupted — this proves the test
        // exercises the in-memory fallback, not a re-read of the (now bad) file.
        let disk_content = std::fs::read_to_string(&cache_path).unwrap();
        assert!(
            serde_json::from_str::<VersionManifest>(&disk_content).is_err(),
            "on-disk cache should be corrupted by the bad fetcher"
        );
    }

    /// RefreshLatest with no cache and network error → returns Err.
    #[test]
    fn refresh_latest_errors_when_no_cache_and_fetch_fails() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("version_manifest_v2.json");
        // No cached manifest written.

        let fetcher = |_: &std::path::Path| -> Result<VersionManifest> {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "simulated network error",
            )))
        };

        let result = fetch_or_cached_impl(
            "latest",
            ManifestCachePolicy::RefreshLatest,
            &cache_path,
            fetcher,
        );
        assert!(result.is_err());
    }

    /// OfflineOnly: uses cache when present.
    #[test]
    fn offline_only_uses_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("version_manifest_v2.json");

        let cached = make_manifest("1.21.4", &[("1.21.4", "http://example.com/1.21.4.json")]);
        write_manifest_to(dir.path(), &cached);

        let fetcher = |_: &std::path::Path| -> Result<VersionManifest> {
            panic!("fetcher must not be called under OfflineOnly");
        };

        let manifest = fetch_or_cached_impl(
            "1.21.4",
            ManifestCachePolicy::OfflineOnly,
            &cache_path,
            fetcher,
        )
        .unwrap();
        assert_eq!(manifest.latest.release, "1.21.4");
    }

    /// OfflineOnly: errors clearly when no cached manifest exists.
    #[test]
    fn offline_only_errors_without_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("version_manifest_v2.json");
        // No cached manifest written.

        let fetcher = |_: &std::path::Path| -> Result<VersionManifest> {
            panic!("fetcher must not be called under OfflineOnly");
        };

        let result = fetch_or_cached_impl(
            "1.21.4",
            ManifestCachePolicy::OfflineOnly,
            &cache_path,
            fetcher,
        );
        assert!(matches!(result, Err(Error::Io(_))));
    }

    /// latest_release_version() equivalent: RefreshLatest should not return the
    /// stale cached release when a fresh manifest is available.
    #[test]
    fn refresh_latest_does_not_return_stale_cached_release() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("version_manifest_v2.json");

        let stale = make_manifest("1.19.0", &[("1.19.0", "http://example.com/1.19.0.json")]);
        write_manifest_to(dir.path(), &stale);

        let fresh = make_manifest(
            "1.21.4",
            &[
                ("1.21.4", "http://example.com/1.21.4.json"),
                ("1.19.0", "http://example.com/1.19.0.json"),
            ],
        );
        let fetcher = move |_: &std::path::Path| -> Result<VersionManifest> { Ok(fresh.clone()) };

        let manifest = fetch_or_cached_impl(
            "latest",
            ManifestCachePolicy::RefreshLatest,
            &cache_path,
            fetcher,
        )
        .unwrap();
        let release = manifest.resolve("latest").unwrap().id.clone();
        assert_eq!(
            release, "1.21.4",
            "stale cached release 1.19.0 should not be returned"
        );
    }
}
