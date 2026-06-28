#![forbid(unsafe_code)]

//! # sand-build
//!
//! Build pipeline for the [Sand](https://github.com/ThatOneToast/sand)
//! Minecraft datapack toolkit.
//!
//! This crate handles:
//!
//! 1. Fetching Mojang's version manifest and resolving version strings
//! 2. Downloading and caching the Minecraft server jar (with SHA1 verification)
//! 3. Running Minecraft's built-in data generator
//! 4. Parsing the generated reports and producing Rust source files:
//!    - `registries.rs` — enums for `Item`, `Block`, `EntityType`, `Biome`,
//!      `Enchantment`, `SoundEvent`
//!    - `block_states.rs` — typed per-block property structs and shared enums
//!    - `commands.rs` — typed command builders from `commands.json`
//!
//! # Usage
//!
//! Typically called from a `build.rs` script:
//!
//! ```rust,ignore
//! fn main() {
//!     let mc_version = std::env::var("SAND_MC_VERSION")
//!         .unwrap_or_else(|_| "1.21.4".to_string());
//!     sand_build::generate(&mc_version).expect("sand-build codegen failed");
//! }
//! ```
//!
//! Requires Java 21+ on `PATH` for the data generator.

mod cache;
mod codegen;
mod download;
mod error;
mod manifest;
mod report;

pub use error::{Error, Result};

struct VersionCacheLock {
    path: std::path::PathBuf,
}

impl VersionCacheLock {
    fn acquire(version_id: &str) -> Result<Self> {
        use std::io::ErrorKind;
        use std::time::{Duration, Instant};

        let dir = cache::version_dir(version_id)?;
        cache::ensure_dir(&dir)?;
        let path = dir.join(".sand-codegen.lock");
        let start = Instant::now();

        loop {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(_) => return Ok(Self { path }),
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                    if start.elapsed() > Duration::from_secs(300) {
                        return Err(std::io::Error::new(
                            ErrorKind::TimedOut,
                            format!(
                                "timed out waiting for Sand codegen cache lock '{}'",
                                path.display()
                            ),
                        )
                        .into());
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

impl Drop for VersionCacheLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Download and cache the vanilla server jar for `mc_version`, returning its path.
///
/// Resolves `"latest"` to the current release via Mojang's version manifest.
/// The jar is cached in `~/.sand/cache/<version>/server.jar` and SHA1-verified
/// on every call; it is only re-downloaded when the checksum does not match.
pub fn ensure_server_jar(mc_version: &str) -> Result<std::path::PathBuf> {
    let (version_id, version_json_url) = resolve_version(mc_version)?;
    download::ensure_server_jar(&version_id, &version_json_url)
}

/// Returns the latest stable Minecraft release version string by fetching
/// Mojang's version manifest.
///
/// Falls back to Sand's bundled latest-known version if the manifest cannot be
/// fetched (e.g. offline), so `"latest"` remains aligned with the verified
/// version table used by `sand-core`.
pub fn latest_release_version() -> String {
    latest_release_version_with(manifest::VersionManifest::fetch_fresh)
}

fn latest_release_version_with<F>(fetch_manifest: F) -> String
where
    F: FnOnce() -> Result<manifest::VersionManifest>,
{
    fetch_manifest()
        .and_then(|m| m.resolve("latest").map(|e| e.id.clone()))
        .unwrap_or_else(|_| latest_known_version().to_string())
}

fn latest_known_version() -> &'static str {
    sand_version::LATEST_KNOWN
}

/// Resolve `mc_version` to a `(version_id, version_json_url)` pair.
///
/// For `"latest"`, attempts a fresh manifest fetch and falls back to the
/// bundled `sand_version::LATEST_KNOWN` anchor when the network is
/// unavailable — so stale cached manifest data can never silently override
/// the verified bundled anchor on refresh failure.
///
/// For pinned versions (e.g. `"1.21.4"`), uses the normal `PreferCache`
/// policy (network only when the version is absent from cache).
fn resolve_version(mc_version: &str) -> Result<(String, String)> {
    resolve_version_with(mc_version, manifest::VersionManifest::fetch_fresh, |v| {
        manifest::VersionManifest::fetch_or_cached(v)
    })
}

/// Testable core of [`resolve_version`].
///
/// `fetch_fresh` is called for `"latest"` resolution and falls back to
/// `LATEST_KNOWN` on error. `fetch_cached` is called for pinned versions.
fn resolve_version_with<FF, FC>(
    mc_version: &str,
    fetch_fresh: FF,
    fetch_cached: FC,
) -> Result<(String, String)>
where
    FF: FnOnce() -> Result<manifest::VersionManifest>,
    FC: FnOnce(&str) -> Result<manifest::VersionManifest>,
{
    if mc_version == "latest" {
        // For "latest", try a fresh manifest; if that fails, use the bundled
        // anchor so stale cached manifests cannot bypass LATEST_KNOWN.
        let version_id = latest_release_version_with(fetch_fresh);
        // Now fetch the manifest entry for the resolved version id via cache.
        let manifest = fetch_cached(&version_id)?;
        let entry = manifest.resolve(&version_id)?;
        Ok((entry.id.clone(), entry.url.clone()))
    } else {
        let manifest = fetch_cached(mc_version)?;
        let entry = manifest.resolve(mc_version)?;
        Ok((entry.id.clone(), entry.url.clone()))
    }
}

/// Entry point for user `build.rs` scripts.
///
/// Given a Minecraft version string (e.g. `"1.21.4"` or `"latest"`), this
/// function:
///
/// 1. Resolves the version via Mojang's version manifest.
/// 2. Downloads and caches the server jar to `~/.sand/cache/<version>/`.
/// 3. Runs the Minecraft data generator to produce `generated/reports/`.
/// 4. Parses the reports and writes Rust source files to `$OUT_DIR`:
///    - `registries.rs` — enums for `Item`, `Block`, `EntityType`, etc.
///    - `block_states.rs` — typed block property structs and enums.
///
/// Requires Java 21+ on `PATH`.
///
/// # Panics
/// Panics if `OUT_DIR` is not set (i.e. called outside a Cargo build script).
pub fn generate(mc_version: &str) -> Result<()> {
    // Tell Cargo to re-run the build script when the version changes.
    println!("cargo:rerun-if-env-changed=SAND_MC_VERSION");

    let out_dir = std::path::PathBuf::from(
        std::env::var("OUT_DIR").expect("OUT_DIR must be set (called from a build.rs)"),
    );
    generate_to_dir(mc_version, &out_dir)
}

/// Same as [`generate`] but writes output to an explicit directory instead of
/// `$OUT_DIR`. Useful for integration tests and tooling.
pub fn generate_to_dir(mc_version: &str, out_dir: &std::path::Path) -> Result<()> {
    // 1. Resolve version. For "latest", fresh manifest fetch falls back to the
    //    bundled LATEST_KNOWN anchor — stale cached manifests are never used.
    let (version_id, version_json_url) = resolve_version(mc_version)?;
    let _lock = VersionCacheLock::acquire(&version_id)?;

    // 2. Download server jar.
    let jar_path = download::ensure_server_jar(&version_id, &version_json_url)?;

    // 3. Run data generator.
    let reports_dir = report::ensure_reports(&version_id, &jar_path)?;

    // 4. Codegen.
    codegen::generate_all(&reports_dir, out_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::manifest::{LatestVersions, VersionEntry, VersionManifest};
    use crate::{Error, Result};

    #[test]
    fn latest_known_fallback_uses_shared_version_anchor() {
        assert_eq!(super::latest_known_version(), sand_version::LATEST_KNOWN);
    }

    #[test]
    fn latest_release_falls_back_to_shared_anchor_on_refresh_error() {
        let release = super::latest_release_version_with(|| {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "simulated refresh failure",
            )))
        });

        assert_eq!(release, sand_version::LATEST_KNOWN);
    }

    #[test]
    fn latest_release_uses_fresh_manifest_when_available() {
        let release = super::latest_release_version_with(|| -> Result<VersionManifest> {
            Ok(make_manifest("26.3"))
        });

        assert_eq!(release, "26.3");
    }

    fn make_manifest(latest_release: &str) -> VersionManifest {
        VersionManifest {
            latest: LatestVersions {
                release: latest_release.to_string(),
                snapshot: latest_release.to_string(),
            },
            versions: vec![VersionEntry {
                id: latest_release.to_string(),
                version_type: "release".to_string(),
                url: format!("https://example.com/{latest_release}.json"),
                sha1: "fake-sha1".to_string(),
                time: "2026-01-01T00:00:00Z".to_string(),
                release_time: "2026-01-01T00:00:00Z".to_string(),
            }],
        }
    }

    // -------------------------------------------------------------------------
    // resolve_version_with() regression tests
    // -------------------------------------------------------------------------

    /// Regression: when manifest refresh fails for "latest", resolve_version_with
    /// must use LATEST_KNOWN, NOT a stale cached manifest's latest.release.
    ///
    /// Under the old RefreshLatest path, a stale cache like `{"latest":
    /// {"release": "1.19.0"}}` would be returned on network failure, and the
    /// caller would silently use "1.19.0". With the new path, LATEST_KNOWN is
    /// always used instead.
    #[test]
    fn resolve_version_latest_uses_bundled_anchor_on_refresh_failure() {
        let stale_release = "1.19.0"; // what the old code would have returned

        // The stale manifest says the latest is "1.19.0"; the "cache" is able to
        // look up LATEST_KNOWN if asked (new code path), but must NOT be asked
        // for the stale latest.release.
        let stale_manifest = {
            let mut m = make_manifest(stale_release);
            // Inject LATEST_KNOWN so fetch_cached can resolve it after fallback.
            m.versions.push(crate::manifest::VersionEntry {
                id: sand_version::LATEST_KNOWN.to_string(),
                version_type: "release".to_string(),
                url: format!("https://example.com/{}.json", sand_version::LATEST_KNOWN),
                sha1: "fake".to_string(),
                time: String::new(),
                release_time: String::new(),
            });
            m
        };

        let (version_id, _url) = super::resolve_version_with(
            "latest",
            // fetch_fresh fails (offline / network error)
            || -> Result<VersionManifest> {
                Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "simulated network error",
                )))
            },
            // fetch_cached returns the stale manifest (contains both releases)
            move |_v| Ok(stale_manifest.clone()),
        )
        .unwrap();

        assert_ne!(
            version_id, stale_release,
            "stale cached release '{stale_release}' must not be returned on refresh failure"
        );
        assert_eq!(
            version_id,
            sand_version::LATEST_KNOWN,
            "bundled LATEST_KNOWN must be used instead of stale cached release"
        );
    }

    /// Regression: stale cached latest.release does not override LATEST_KNOWN
    /// even when the stale cached manifest is valid/parseable.
    #[test]
    fn stale_cached_latest_does_not_override_bundled_anchor() {
        let stale_release = "1.19.0"; // older than LATEST_KNOWN
        // Manifest containing the stale release AND LATEST_KNOWN (so lookup succeeds).
        let mut stale_manifest = make_manifest(stale_release);
        stale_manifest.versions.push(crate::manifest::VersionEntry {
            id: sand_version::LATEST_KNOWN.to_string(),
            version_type: "release".to_string(),
            url: format!("https://example.com/{}.json", sand_version::LATEST_KNOWN),
            sha1: "fake".to_string(),
            time: String::new(),
            release_time: String::new(),
        });

        let (version_id, _url) = super::resolve_version_with(
            "latest",
            || -> Result<VersionManifest> {
                Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "simulated network error",
                )))
            },
            move |_v| Ok(stale_manifest.clone()),
        )
        .unwrap();

        assert_ne!(
            version_id, stale_release,
            "stale cached release '{stale_release}' must not be returned"
        );
        assert_eq!(
            version_id,
            sand_version::LATEST_KNOWN,
            "bundled LATEST_KNOWN must be used instead of stale cached release"
        );
    }

    /// Fresh manifest lookup still uses the fresh manifest release.
    #[test]
    fn resolve_version_latest_uses_fresh_manifest_on_success() {
        let fresh_release = "26.9";
        let fresh_manifest = make_manifest(fresh_release);
        let cached_manifest = make_manifest(fresh_release);

        let (version_id, url) = super::resolve_version_with(
            "latest",
            move || -> Result<VersionManifest> { Ok(fresh_manifest.clone()) },
            move |_v| Ok(cached_manifest.clone()),
        )
        .unwrap();

        assert_eq!(version_id, fresh_release);
        assert!(url.contains(fresh_release));
    }

    /// Explicit versions still resolve normally (via PreferCache path).
    #[test]
    fn resolve_version_explicit_resolves_normally() {
        let pinned = "1.21.4";
        let manifest = make_manifest(pinned);

        let (version_id, url) = super::resolve_version_with(
            pinned,
            || -> Result<VersionManifest> {
                panic!("fetch_fresh must not be called for pinned versions");
            },
            move |_v| Ok(manifest.clone()),
        )
        .unwrap();

        assert_eq!(version_id, pinned);
        assert!(url.contains(pinned));
    }
}
