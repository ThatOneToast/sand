//! Fingerprint-addressed cache for generated Rust codegen (issue #347 Phase
//! 3).
//!
//! [`super::generate_all`] deterministically turns Minecraft data-generator
//! reports into Rust source (`registries.rs`, `block_states.rs`,
//! `commands.rs`) plus two API-provider manifests
//! (`registries.api.json`, `commands.api.json`). Given the same reports and
//! the same codegen logic, it always produces the same bytes — so instead
//! of re-running it on every build-script invocation whose `rerun-if-*`
//! triggers fired for an unrelated reason, this module caches its output
//! under:
//!
//! ```text
//! ~/.sand/cache/<mc-version>/rust-codegen/<fingerprint>/
//!   registries.rs
//!   registries.api.json
//!   block_states.rs
//!   commands.rs
//!   commands.api.json
//!   manifest.json
//! ```
//!
//! `<fingerprint>` is derived from everything capable of changing the
//! output: the Minecraft version, [`super::CODEGEN_SCHEMA_VERSION`] (bump
//! this whenever `codegen/{registries,blocks,commands}.rs` change what they
//! generate), and the content of the three report files actually read
//! ([`crate::report::ensure_reports`] already guarantees these exist and
//! are version-pinned; this module hashes their bytes rather than trusting
//! their path or mtime). This deliberately does *not* key on Minecraft
//! version alone — a codegen schema bump invalidates every cached entry
//! for every version, and a report content change invalidates just that
//! version's entry.
//!
//! Every public function here takes `cache_root` explicitly (the directory
//! [`crate::cache::cache_dir`] resolves to in production) rather than
//! reading `$HOME` itself, so tests can point at a temporary directory
//! without mutating global process state — safe under `cargo test`'s
//! default multi-threaded execution.
//!
//! # Concurrency / atomicity
//!
//! A cache miss generates into a private temporary directory (unique per
//! process+call), then publishes it with a single [`std::fs::rename`] to
//! the final `<fingerprint>/` path. Any reader either sees no directory
//! (miss) or a fully-populated one (hit) — never a partially written one.
//! If two processes race to populate the same fingerprint (e.g. two
//! worktrees building the same Minecraft version concurrently), the loser's
//! `rename` fails harmlessly because the content is deterministically
//! identical either way; that failure is treated as "someone else already
//! published this," not an error. This needs no locking beyond the
//! atomicity `rename` already provides — no global lock is held for cache
//! population itself (the existing coarse `VersionCacheLock` in
//! `crate::lib` still serializes the surrounding jar-download/
//! report-generation steps; narrowing that further is tracked as follow-up,
//! see the issue #347 PR description).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::fingerprint::{combine, hash_bytes};

/// Bumped whenever this cache's on-disk manifest shape changes
/// incompatibly. A manifest with any other value is treated as absent.
const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// The exact set of files [`super::generate_all`] writes into `out_dir`,
/// and therefore the exact set of files this cache stores/restores. Kept as
/// one list so the cache and the generator can't silently drift apart.
const GENERATED_FILES: &[&str] = &[
    "registries.rs",
    "registries.api.json",
    "block_states.rs",
    "commands.rs",
    "commands.api.json",
];

/// The three data-generator report files codegen reads, in the fixed order
/// they're hashed into the fingerprint.
const REPORT_FILES: &[&str] = &["registries.json", "blocks.json", "commands.json"];

#[derive(Debug, Serialize, Deserialize)]
struct CacheManifestFile {
    schema_version: u32,
    fingerprint: String,
    files: Vec<String>,
}

/// Computes this codegen invocation's cache fingerprint from the Minecraft
/// version, the codegen schema version, and the content of every report
/// file codegen reads.
///
/// Deterministic and path-independent: only file *contents* and the fixed
/// inputs above affect the result, never `reports_dir`'s absolute path —
/// required for the fingerprint to mean the same thing across worktrees.
pub fn fingerprint(reports_dir: &Path, minecraft_version: &str) -> Result<String> {
    let mut parts = vec![
        "codegen-schema".to_string(),
        super::CODEGEN_SCHEMA_VERSION.to_string(),
        "mc-version".to_string(),
        minecraft_version.to_string(),
    ];
    for report in REPORT_FILES {
        let bytes = std::fs::read(reports_dir.join(report))?;
        parts.push(format!("report:{report}"));
        parts.push(hash_bytes(&bytes));
    }
    Ok(combine(parts.iter().map(String::as_str)))
}

fn entry_dir(cache_root: &Path, minecraft_version: &str, fp: &str) -> PathBuf {
    cache_root
        .join(minecraft_version)
        .join("rust-codegen")
        .join(fp)
}

/// Attempts to satisfy this codegen invocation from the cache rooted at
/// `cache_root`. Returns `true` and populates `out_dir` on a validated hit;
/// returns `false` (leaving `out_dir` untouched) on any miss, including a
/// missing, corrupt, or fingerprint-mismatched cache entry — the caller is
/// expected to fall back to generating normally and calling [`publish`].
///
/// A corrupt entry (unreadable manifest, mismatched schema version,
/// fingerprint mismatch, or a missing generated file) is treated
/// identically to a plain miss: this function never partially copies files
/// out of a broken entry, and never panics or errors on corruption — it
/// safely reports "not usable" so codegen can regenerate.
pub fn try_load(
    cache_root: &Path,
    minecraft_version: &str,
    fp: &str,
    out_dir: &Path,
) -> Result<bool> {
    let dir = entry_dir(cache_root, minecraft_version, fp);
    let Some(manifest) = read_manifest(&dir, fp) else {
        return Ok(false);
    };
    // Validate every expected file exists before copying any of them, so a
    // partially-corrupt entry (e.g. truncated by an out-of-band `rm`) never
    // leaves `out_dir` half-populated from a bad cache read.
    for file in &manifest.files {
        if !dir.join(file).is_file() {
            return Ok(false);
        }
    }
    for file in &manifest.files {
        std::fs::copy(dir.join(file), out_dir.join(file))?;
    }
    Ok(true)
}

fn read_manifest(dir: &Path, expected_fingerprint: &str) -> Option<CacheManifestFile> {
    let bytes = std::fs::read(dir.join("manifest.json")).ok()?;
    let manifest: CacheManifestFile = serde_json::from_slice(&bytes).ok()?;
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return None;
    }
    if manifest.fingerprint != expected_fingerprint {
        return None;
    }
    Some(manifest)
}

/// Publishes freshly generated files from `out_dir` (which must already
/// contain every file in [`GENERATED_FILES`], i.e. codegen just ran) into
/// the cache rooted at `cache_root` under `fp`, via a
/// temp-directory-then-rename so concurrent readers never observe a
/// partial entry (see the module docs).
///
/// This is best-effort: a failure to publish (e.g. a read-only cache
/// directory) does not fail the build — the caller already has correct
/// output in `out_dir` regardless of whether caching it succeeds, and a
/// missing/broken cache must only cost speed, never correctness.
pub fn publish(cache_root: &Path, minecraft_version: &str, fp: &str, out_dir: &Path) {
    if let Err(error) = try_publish(cache_root, minecraft_version, fp, out_dir) {
        println!(
            "cargo:warning=sand-build: failed to publish generated-code cache entry \
             for MC {minecraft_version} (fingerprint {fp}): {error}. Codegen output is \
             still correct; only caching for future builds was skipped."
        );
    }
}

fn try_publish(cache_root: &Path, minecraft_version: &str, fp: &str, out_dir: &Path) -> Result<()> {
    let final_dir = entry_dir(cache_root, minecraft_version, fp);
    if final_dir.is_dir() {
        // Already published by this or another process for this exact
        // fingerprint -- nothing to do, and no reason to re-publish
        // deterministic content.
        return Ok(());
    }
    let parent = final_dir.parent().ok_or_else(|| Error::MissingField {
        field: "parent",
        context: "rust-codegen cache entry path".to_string(),
    })?;
    std::fs::create_dir_all(parent)?;
    let tmp_dir = parent.join(format!(".tmp-{fp}-{}-{}", std::process::id(), tmp_nonce()));
    std::fs::create_dir_all(&tmp_dir)?;

    for file in GENERATED_FILES {
        std::fs::copy(out_dir.join(file), tmp_dir.join(file))?;
    }
    let manifest = CacheManifestFile {
        schema_version: MANIFEST_SCHEMA_VERSION,
        fingerprint: fp.to_string(),
        files: GENERATED_FILES.iter().map(|s| s.to_string()).collect(),
    };
    std::fs::write(
        tmp_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;

    match std::fs::rename(&tmp_dir, &final_dir) {
        Ok(()) => Ok(()),
        Err(_) if final_dir.is_dir() => {
            // Lost a publish race to another process; its content is
            // deterministically identical for this fingerprint, so this is
            // not an error. Clean up our now-orphaned temp dir.
            let _ = std::fs::remove_dir_all(&tmp_dir);
            Ok(())
        }
        Err(error) => {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            Err(error.into())
        }
    }
}

fn tmp_nonce() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_reports(dir: &Path) {
        std::fs::write(dir.join("registries.json"), b"{\"a\":1}").unwrap();
        std::fs::write(dir.join("blocks.json"), b"{\"b\":2}").unwrap();
        std::fs::write(dir.join("commands.json"), b"{\"c\":3}").unwrap();
    }

    fn write_generated(dir: &Path) {
        std::fs::write(dir.join("registries.rs"), b"// registries").unwrap();
        std::fs::write(dir.join("registries.api.json"), b"{}").unwrap();
        std::fs::write(dir.join("block_states.rs"), b"// blocks").unwrap();
        std::fs::write(dir.join("commands.rs"), b"// commands").unwrap();
        std::fs::write(dir.join("commands.api.json"), b"{}").unwrap();
    }

    #[test]
    fn fingerprint_is_deterministic_for_identical_reports() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        write_reports(a.path());
        write_reports(b.path());
        assert_eq!(
            fingerprint(a.path(), "1.21.4").unwrap(),
            fingerprint(b.path(), "1.21.4").unwrap()
        );
    }

    #[test]
    fn fingerprint_changes_when_a_report_changes() {
        let dir = tempfile::tempdir().unwrap();
        write_reports(dir.path());
        let before = fingerprint(dir.path(), "1.21.4").unwrap();
        std::fs::write(dir.path().join("blocks.json"), b"{\"b\":999}").unwrap();
        let after = fingerprint(dir.path(), "1.21.4").unwrap();
        assert_ne!(before, after);
    }

    #[test]
    fn fingerprint_changes_when_mc_version_changes() {
        let dir = tempfile::tempdir().unwrap();
        write_reports(dir.path());
        assert_ne!(
            fingerprint(dir.path(), "1.21.4").unwrap(),
            fingerprint(dir.path(), "1.21.5").unwrap()
        );
    }

    #[test]
    fn identical_report_bytes_fingerprint_identically_regardless_of_write_order() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        std::fs::write(a.path().join("commands.json"), b"{\"c\":3}").unwrap();
        std::fs::write(a.path().join("registries.json"), b"{\"a\":1}").unwrap();
        std::fs::write(a.path().join("blocks.json"), b"{\"b\":2}").unwrap();
        write_reports(b.path());
        assert_eq!(
            fingerprint(a.path(), "1.21.4").unwrap(),
            fingerprint(b.path(), "1.21.4").unwrap()
        );
    }

    #[test]
    fn miss_when_never_published() {
        let cache_root = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "deadbeef", out.path()).unwrap();
        assert!(!hit);
    }

    #[test]
    fn publish_then_load_round_trips_byte_identical_files() {
        let cache_root = tempfile::tempdir().unwrap();
        let generated = tempfile::tempdir().unwrap();
        write_generated(generated.path());

        publish(cache_root.path(), "1.21.4", "abc123", generated.path());

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(hit);
        for file in GENERATED_FILES {
            assert_eq!(
                std::fs::read(generated.path().join(file)).unwrap(),
                std::fs::read(restored.path().join(file)).unwrap(),
                "cached '{file}' must be byte-identical to the original"
            );
        }
    }

    #[test]
    fn mismatched_fingerprint_is_a_miss_even_though_the_cache_root_has_entries() {
        let cache_root = tempfile::tempdir().unwrap();
        let generated = tempfile::tempdir().unwrap();
        write_generated(generated.path());
        publish(cache_root.path(), "1.21.4", "abc123", generated.path());

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(
            cache_root.path(),
            "1.21.4",
            "different-fingerprint",
            restored.path(),
        )
        .unwrap();
        assert!(!hit);
    }

    #[test]
    fn corrupt_manifest_is_treated_as_a_miss_not_an_error() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("manifest.json"), b"not json{{{").unwrap();

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(!hit);
    }

    #[test]
    fn manifest_referencing_a_missing_file_is_treated_as_a_miss() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = CacheManifestFile {
            schema_version: MANIFEST_SCHEMA_VERSION,
            fingerprint: "abc123".to_string(),
            files: vec!["registries.rs".to_string()],
        };
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        // registries.rs deliberately not written -- entry is corrupt.

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(!hit);
        assert!(
            !restored.path().join("registries.rs").exists(),
            "a corrupt entry must never partially populate out_dir"
        );
    }

    #[test]
    fn republishing_the_same_fingerprint_is_a_harmless_no_op() {
        let cache_root = tempfile::tempdir().unwrap();
        let generated = tempfile::tempdir().unwrap();
        write_generated(generated.path());
        publish(cache_root.path(), "1.21.4", "abc123", generated.path());
        // Publishing again for the same fingerprint must not error or
        // corrupt the existing entry.
        publish(cache_root.path(), "1.21.4", "abc123", generated.path());

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(hit);
    }

    #[test]
    fn concurrent_publish_of_the_same_fingerprint_does_not_error() {
        let cache_root = tempfile::tempdir().unwrap();
        let generated = tempfile::tempdir().unwrap();
        write_generated(generated.path());

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let cache_root = cache_root.path().to_path_buf();
                let generated = generated.path().to_path_buf();
                std::thread::spawn(move || {
                    publish(&cache_root, "1.21.4", "abc123", &generated);
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(hit, "at least one concurrent publisher must succeed");
    }
}
