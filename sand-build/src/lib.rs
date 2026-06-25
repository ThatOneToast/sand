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
    let manifest = manifest::VersionManifest::fetch_or_cached(mc_version)?;
    let entry = manifest.resolve(mc_version)?;
    download::ensure_server_jar(&entry.id, &entry.url)
}

/// Returns the latest stable Minecraft release version string by fetching
/// Mojang's version manifest.
///
/// Falls back to `"1.21.11"` if the manifest cannot be fetched (e.g. offline).
pub fn latest_release_version() -> String {
    manifest::VersionManifest::fetch_or_cached("latest")
        .and_then(|m| m.resolve("latest").map(|e| e.id.clone()))
        .unwrap_or_else(|_| "1.21.11".to_string())
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
    // 1. Resolve version.
    let manifest = manifest::VersionManifest::fetch_or_cached(mc_version)?;
    let entry = manifest.resolve(mc_version)?;
    let version_id = entry.id.clone();
    let version_json_url = entry.url.clone();
    let _lock = VersionCacheLock::acquire(&version_id)?;

    // 2. Download server jar.
    let jar_path = download::ensure_server_jar(&version_id, &version_json_url)?;

    // 3. Run data generator.
    let reports_dir = report::ensure_reports(&version_id, &jar_path)?;

    // 4. Codegen.
    codegen::generate_all(&reports_dir, out_dir)?;

    Ok(())
}
