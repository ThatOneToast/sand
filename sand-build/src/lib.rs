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
//!         .unwrap_or_else(|_| "26.1".to_string());
//!     sand_build::generate(&mc_version).expect("sand-build codegen failed");
//! }
//! ```
//!
//! Requires a Java runtime new enough for the selected server on `PATH`.
//! Sand's current Minecraft 26.2 validation target uses Java 25.

mod api_provider;
mod cache;
mod codegen;
mod download;
mod error;
pub mod fingerprint;
mod manifest;
mod registry_fixture;
mod registry_id_contracts;
mod report;
pub mod source_tree_fingerprint;

pub use api_provider::{
    ApiProviderCatalog, GeneratedProviderEntry, PROVIDER_SCHEMA_VERSION, read_api_provider,
    validate_api_provider_source, write_placeholder_codegen,
};
pub use error::{Error, Result};
pub use registry_fixture::refresh_registry_coverage_fixture;
pub use registry_id_contracts::{
    install_registry_id_contract_provider, registry_id_contract_provider,
};

/// Download and cache the vanilla server jar for `mc_version`, returning its path.
///
/// Resolves `"latest"` to Sand's bundled latest-known verified version.
/// Returns an immutable digest-addressed path under `~/.sand/cache/<version>/artifacts/`.
/// The JAR is SHA1-verified on every call against cached package bytes verified
/// against the selected Mojang manifest entry. A fully verified warm cache needs
/// no HTTP. A legacy JAR alone may still need a metadata fetch before import.
/// Use the returned path; do not construct a version-only cache filename.
/// Calls serialize with other Sand artifact writers; a live owner can delay a
/// call for up to five minutes before a lock timeout. OS locks release on exit.
pub fn ensure_server_jar(mc_version: &str) -> Result<std::path::PathBuf> {
    ensure_server_jar_in(&cache::cache_dir()?, mc_version, &download::Http)
}

fn ensure_server_jar_in(
    root: &std::path::Path,
    version: &str,
    http: &impl download::Transport,
) -> Result<std::path::PathBuf> {
    let entry = resolve_version(root, version, http)?;
    let cache = cache::VersionCache::acquire(root, &entry.id)?;
    download::ensure_server_jar(&cache, &entry, http)
}

/// Returns Sand's bundled latest-known verified Minecraft version.
///
/// This keeps `"latest"` aligned with the verified version table used by
/// `sand-core`, pack metadata, and version-sensitive feature flags. Pinned
/// versions still resolve through Mojang's version manifest.
pub fn latest_release_version() -> String {
    latest_known_version().to_string()
}

fn latest_known_version() -> &'static str {
    sand_version::LATEST_KNOWN
}

/// Resolve once, keeping the selected entry intact across lock waits.
fn resolve_version(
    root: &std::path::Path,
    mc_version: &str,
    http: &impl download::Transport,
) -> Result<manifest::VersionEntry> {
    let concrete = if mc_version == "latest" {
        latest_known_version()
    } else {
        mc_version
    };
    manifest::resolve(root, concrete, http)
}

/// Entry point for user `build.rs` scripts.
///
/// Given a Minecraft version string (e.g. `"26.1"` or `"latest"`), this
/// function:
///
/// 1. Resolves the version via Mojang's version manifest.
/// 2. Obtains the verified immutable JAR via the same cache as [`ensure_server_jar`].
/// 3. Generates or reuses reports with matching JAR provenance.
/// 4. Parses the reports and writes Rust source files to `$OUT_DIR`:
///    - `registries.rs` — enums for `Item`, `Block`, `EntityType`, etc.
///    - `block_states.rs` — typed block property structs and enums.
///
/// Requires a Java runtime new enough for the selected Minecraft server when
/// reports are absent or invalid. Legacy version-only reports are not reused.
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
    // 1. Resolve version. For "latest", use the bundled LATEST_KNOWN anchor so
    //    generated APIs and runtime version metadata stay aligned.
    let root = cache::cache_dir()?;
    let entry = resolve_version(&root, mc_version, &download::Http)?;
    let cache = cache::VersionCache::acquire(&root, &entry.id)?;

    // 2. Download server jar.
    let jar_path = download::ensure_server_jar(&cache, &entry, &download::Http)?;

    // 3. Run data generator.
    let reports_dir = report::ensure_reports(&cache, &jar_path)?;

    // 4. Codegen (fingerprint-cached -- issue #347 Phase 3).
    codegen::generate_all_cached(&reports_dir, out_dir, &entry.id)?;

    Ok(())
}
