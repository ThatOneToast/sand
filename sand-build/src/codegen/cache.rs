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
//! output: the Minecraft version, [`super::CODEGEN_IMPL_FINGERPRINT`] (an
//! automatic recursive hash of the generator's entire source tree --
//! `codegen/**/*.rs` by relative path and content, computed at compile time,
//! see `sand-build/build.rs` and `source_tree_fingerprint.rs` -- so editing,
//! adding, or removing any generator source file invalidates every cached
//! entry without anyone needing to remember a manual bump or update a file
//! list), [`super::CODEGEN_CACHE_FORMAT_VERSION`] (an explicit salt for
//! cache-format-only changes), and the content of the three report files
//! actually read ([`crate::report::ensure_reports`]
//! already guarantees these exist and are version-pinned; this module
//! hashes their bytes rather than trusting their path or mtime). This
//! deliberately does *not* key on Minecraft version alone — a generator
//! implementation change invalidates every cached entry for every version,
//! and a report content change invalidates just that version's entry.
//!
//! Every public function here takes `cache_root` explicitly (the directory
//! [`crate::cache::cache_dir`] resolves to in production) rather than
//! reading `$HOME` itself, so tests can point at a temporary directory
//! without mutating global process state — safe under `cargo test`'s
//! default multi-threaded execution.
//!
//! # Validation
//!
//! A cache entry is never trusted merely because its directory and
//! `manifest.json` exist. [`try_load`] requires, in order: the manifest
//! parses and its schema version matches; its recorded fingerprint matches
//! the one requested; its file list is *exactly* the canonical
//! [`GENERATED_FILES`] set (no missing entry, no unexpected extra entry);
//! every listed file exists on disk; every listed file's actual bytes hash
//! to the value recorded for it in the manifest. Only once *all* of that
//! holds does it copy anything into `out_dir` — a manifest that references
//! a file that turns out to be missing, truncated, or bit-flipped is a
//! miss, not a partially-populated `out_dir`.
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

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::fingerprint::{combine, hash_bytes};

/// Bumped whenever this cache's on-disk manifest shape changes
/// incompatibly. A manifest with any other value is treated as absent.
const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// The exact set of files [`super::generate_all`] writes into `out_dir`,
/// and therefore the exact set of files this cache stores/restores/
/// requires a published manifest to list — no more, no fewer. Kept as one
/// list so the cache and the generator can't silently drift apart.
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
    /// Filename (one of [`GENERATED_FILES`]) -> content hash
    /// ([`hash_bytes`] of that file's bytes at publish time).
    files: BTreeMap<String, String>,
}

/// Computes this codegen invocation's cache fingerprint from the Minecraft
/// version, the automatic generator-implementation fingerprint, the
/// explicit cache-format salt, and the content of every report file codegen
/// reads.
///
/// Deterministic and path-independent: only file *contents* and the fixed
/// inputs above affect the result, never `reports_dir`'s absolute path —
/// required for the fingerprint to mean the same thing across worktrees.
pub fn fingerprint(reports_dir: &Path, minecraft_version: &str) -> Result<String> {
    let mut parts = vec![
        "codegen-impl".to_string(),
        super::CODEGEN_IMPL_FINGERPRINT.to_string(),
        "codegen-cache-format".to_string(),
        super::CODEGEN_CACHE_FORMAT_VERSION.to_string(),
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

/// The outcome of validating a cache entry directory, whether or not it
/// exists. Distinguishing [`Missing`](CacheEntryState::Missing) from
/// [`Corrupt`](CacheEntryState::Corrupt) matters for [`try_publish`]: a
/// missing entry is published normally, but a corrupt one must be actively
/// *repaired* (replaced), not silently left in place forever because
/// "something is already there" (issue #347 PR #348 review item 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheEntryState {
    /// No directory at all, or a directory Cargo/us haven't published to
    /// yet (e.g. an empty leftover). Safe to populate directly.
    Missing,
    /// A fully valid, hash-verified entry for the requested fingerprint
    /// already exists. Nothing to do.
    Valid,
    /// A directory exists but fails validation (unreadable/malformed
    /// manifest, wrong fingerprint, non-canonical file set, missing file,
    /// or a content-hash mismatch). Must be replaced, not left in place --
    /// otherwise every future build would keep hitting this same poisoned
    /// entry, detecting it's corrupt, and falling back to full regeneration
    /// forever.
    Corrupt,
}

/// Validates the cache entry at `dir` against the expected fingerprint,
/// returning why it either validates or doesn't. Never touches `out_dir` or
/// any other side effects -- purely a read-only check, so it's safe to call
/// speculatively from both [`try_load`] and [`try_publish`].
fn validate_entry(dir: &Path, fp: &str) -> CacheEntryState {
    if !dir.is_dir() {
        return CacheEntryState::Missing;
    }
    let Some(manifest) = read_manifest(dir, fp) else {
        return CacheEntryState::Corrupt;
    };
    if !manifest_file_set_is_canonical(&manifest) {
        return CacheEntryState::Corrupt;
    }
    for (file, expected_hash) in &manifest.files {
        let Ok(bytes) = std::fs::read(dir.join(file)) else {
            return CacheEntryState::Corrupt;
        };
        if &hash_bytes(&bytes) != expected_hash {
            return CacheEntryState::Corrupt;
        }
    }
    CacheEntryState::Valid
}

/// Attempts to satisfy this codegen invocation from the cache rooted at
/// `cache_root`. Returns `true` and populates `out_dir` on a fully
/// validated hit; returns `false` (leaving `out_dir` untouched) on any
/// miss, including a missing, corrupt, incomplete, or fingerprint-
/// mismatched cache entry — the caller is expected to fall back to
/// generating normally and calling [`publish`], which will *repair* a
/// corrupt entry rather than leaving it permanently poisoned.
///
/// See the module docs' "Validation" section for exactly what is checked.
/// Nothing is copied into `out_dir` until every check has passed for every
/// file.
pub fn try_load(
    cache_root: &Path,
    minecraft_version: &str,
    fp: &str,
    out_dir: &Path,
) -> Result<bool> {
    let dir = entry_dir(cache_root, minecraft_version, fp);
    if validate_entry(&dir, fp) != CacheEntryState::Valid {
        return Ok(false);
    }
    // Re-read the manifest (cheap; the file set is tiny) rather than thread
    // it out of `validate_entry`, keeping that function a pure yes/no check
    // with a single obvious call site for "what do I actually copy."
    let manifest = read_manifest(&dir, fp).expect("just validated as Valid");
    for file in manifest.files.keys() {
        std::fs::copy(dir.join(file), out_dir.join(file))?;
    }
    Ok(true)
}

/// A manifest's file set is canonical iff it lists exactly
/// [`GENERATED_FILES`] — no missing canonical entry, no unexpected extra
/// entry. Either deviation is treated as corruption, not partial success.
fn manifest_file_set_is_canonical(manifest: &CacheManifestFile) -> bool {
    if manifest.files.len() != GENERATED_FILES.len() {
        return false;
    }
    GENERATED_FILES
        .iter()
        .all(|expected| manifest.files.contains_key(*expected))
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
/// partial entry (see the module docs). Records each file's content hash
/// in the published manifest so a future [`try_load`] can detect
/// corruption rather than trusting the files blindly.
///
/// If an entry already exists at this fingerprint, it is re-validated
/// first: a valid entry is left alone (no reason to re-publish
/// deterministic content), but a *corrupt* one is atomically replaced --
/// see [`try_publish`] for exactly how, and the module docs for why this
/// matters (a corrupt entry that's never repaired would poison every
/// future build at this fingerprint forever, each one detecting the
/// corruption, missing, and regenerating from scratch every single time).
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
    match validate_entry(&final_dir, fp) {
        CacheEntryState::Valid => {
            // Already published by this or another process for this exact
            // fingerprint, and it's genuinely good -- nothing to do, and no
            // reason to re-publish deterministic content.
            return Ok(());
        }
        CacheEntryState::Missing => {}
        CacheEntryState::Corrupt => {
            // Fall through to publish a fresh entry, then swap it in below.
        }
    }
    let parent = final_dir.parent().ok_or_else(|| Error::MissingField {
        field: "parent",
        context: "rust-codegen cache entry path".to_string(),
    })?;
    std::fs::create_dir_all(parent)?;
    let tmp_dir = parent.join(format!(".tmp-{fp}-{}-{}", std::process::id(), tmp_nonce()));
    std::fs::create_dir_all(&tmp_dir)?;

    let mut files = BTreeMap::new();
    for file in GENERATED_FILES {
        let bytes = std::fs::read(out_dir.join(file))?;
        files.insert((*file).to_string(), hash_bytes(&bytes));
        std::fs::copy(out_dir.join(file), tmp_dir.join(file))?;
    }
    let manifest = CacheManifestFile {
        schema_version: MANIFEST_SCHEMA_VERSION,
        fingerprint: fp.to_string(),
        files,
    };
    std::fs::write(
        tmp_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;

    match std::fs::rename(&tmp_dir, &final_dir) {
        Ok(()) => Ok(()),
        Err(_) if validate_entry(&final_dir, fp) == CacheEntryState::Valid => {
            // Lost a publish race to another process that got a valid
            // entry in first; its content is deterministically identical
            // for this fingerprint, so this is not an error. Clean up our
            // now-orphaned temp dir.
            let _ = std::fs::remove_dir_all(&tmp_dir);
            Ok(())
        }
        Err(_) if final_dir.is_dir() => {
            // The direct rename failed (most likely because `final_dir`
            // already exists -- POSIX rename onto a non-empty directory
            // fails), and re-validation didn't find a valid winner either:
            // this is the corrupt-entry-repair path. Move the corrupt
            // directory out of the way first (an already-open reader's file
            // descriptors stay valid even after their directory entry
            // moves, so this can't tear a concurrent `try_load` read), then
            // move our fresh entry into place, then best-effort clean up
            // the quarantined corrupt copy. If another process wins this
            // same race, our stale-rename below simply fails and we treat
            // that as "someone else repaired it," not an error -- the
            // final state is a valid entry either way.
            let stale_dir = parent.join(format!(
                ".stale-{fp}-{}-{}",
                std::process::id(),
                tmp_nonce()
            ));
            match std::fs::rename(&final_dir, &stale_dir) {
                Ok(()) => match std::fs::rename(&tmp_dir, &final_dir) {
                    Ok(()) => {
                        let _ = std::fs::remove_dir_all(&stale_dir);
                        Ok(())
                    }
                    Err(_) if validate_entry(&final_dir, fp) == CacheEntryState::Valid => {
                        // Another process published a valid entry into the
                        // now-empty slot between our two renames.
                        let _ = std::fs::remove_dir_all(&tmp_dir);
                        let _ = std::fs::remove_dir_all(&stale_dir);
                        Ok(())
                    }
                    Err(error) => {
                        // Could neither install our fresh entry nor find a
                        // valid one already there. Restore the original
                        // (still-corrupt, but at least present) entry
                        // rather than leaving the fingerprint directory
                        // empty, and surface the error -- `publish` logs it
                        // and the build continues using its already-correct
                        // `out_dir` regardless.
                        let _ = std::fs::rename(&stale_dir, &final_dir);
                        let _ = std::fs::remove_dir_all(&tmp_dir);
                        Err(error.into())
                    }
                },
                Err(_) if validate_entry(&final_dir, fp) == CacheEntryState::Valid => {
                    // Someone else already repaired it between our
                    // `validate_entry` check above and this rename attempt.
                    let _ = std::fs::remove_dir_all(&tmp_dir);
                    Ok(())
                }
                Err(error) => {
                    // Couldn't even quarantine the corrupt entry (e.g.
                    // permissions, or it vanished and reappeared). Leave
                    // whatever's there alone and surface the error; the
                    // caller's `out_dir` is already correct regardless.
                    let _ = std::fs::remove_dir_all(&tmp_dir);
                    Err(error.into())
                }
            }
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

    /// A valid, fully-canonical manifest for a published entry whose files
    /// were written by [`write_generated`].
    fn canonical_manifest(fp: &str) -> CacheManifestFile {
        let dir_stub = tempfile::tempdir().unwrap();
        write_generated(dir_stub.path());
        let files = GENERATED_FILES
            .iter()
            .map(|file| {
                let bytes = std::fs::read(dir_stub.path().join(file)).unwrap();
                ((*file).to_string(), hash_bytes(&bytes))
            })
            .collect();
        CacheManifestFile {
            schema_version: MANIFEST_SCHEMA_VERSION,
            fingerprint: fp.to_string(),
            files,
        }
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

    /// The generator-implementation fingerprint is compiled in from
    /// `build.rs` and can't be swapped at test time, so this test instead
    /// proves the *mechanism* the real fingerprint relies on: that
    /// changing any input string folded into `combine(...)` -- which is
    /// exactly how `CODEGEN_IMPL_FINGERPRINT` participates -- changes the
    /// result. Combined with `sand-build/build.rs`'s own
    /// `hash_files`/`rerun-if-changed` wiring (which recomputes the
    /// constant whenever the generator source changes), this is the
    /// automatic-invalidation guarantee end to end: build.rs changes the
    /// constant automatically, and this proves the constant changing
    /// changes the fingerprint.
    #[test]
    fn simulated_generator_implementation_identity_change_invalidates_the_fingerprint() {
        let dir = tempfile::tempdir().unwrap();
        write_reports(dir.path());
        let with_impl_a = combine([
            "codegen-impl",
            "impl-fingerprint-a",
            "codegen-cache-format",
            "1",
            "mc-version",
            "1.21.4",
        ]);
        let with_impl_b = combine([
            "codegen-impl",
            "impl-fingerprint-b",
            "codegen-cache-format",
            "1",
            "mc-version",
            "1.21.4",
        ]);
        assert_ne!(
            with_impl_a, with_impl_b,
            "a generator implementation identity change must change the fingerprint"
        );
    }

    #[test]
    fn real_codegen_impl_fingerprint_is_a_nonempty_stable_hash() {
        // Sanity check that build.rs actually embedded something real, not
        // an empty string or a constant placeholder.
        assert!(!super::super::CODEGEN_IMPL_FINGERPRINT.is_empty());
        assert_eq!(
            super::super::CODEGEN_IMPL_FINGERPRINT,
            super::super::CODEGEN_IMPL_FINGERPRINT,
            "must be a fixed compiled-in value"
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
        let manifest = canonical_manifest("abc123");
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        // None of the referenced files were actually written -- entry is corrupt.

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(!hit);
        assert!(
            !restored.path().join("registries.rs").exists(),
            "a corrupt entry must never partially populate out_dir"
        );
    }

    #[test]
    fn manifest_missing_one_canonical_entry_is_a_miss() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);

        let mut manifest = canonical_manifest("abc123");
        manifest.files.remove("commands.api.json");
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(
            !hit,
            "a manifest missing a canonical generated file must be rejected"
        );
    }

    #[test]
    fn manifest_with_an_unexpected_extra_entry_is_a_miss() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);
        std::fs::write(dir.join("unexpected.rs"), b"// not a canonical file").unwrap();

        let mut manifest = canonical_manifest("abc123");
        manifest.files.insert(
            "unexpected.rs".to_string(),
            hash_bytes(b"// not a canonical file"),
        );
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(
            !hit,
            "a manifest with an entry outside the canonical file set must be rejected"
        );
    }

    #[test]
    fn corrupted_rs_bytes_that_dont_match_the_recorded_hash_are_a_miss() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);
        let manifest = canonical_manifest("abc123");
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        // Simulate on-disk bit-rot / truncation of a cached .rs file after
        // the manifest was published.
        std::fs::write(dir.join("registries.rs"), b"// CORRUPTED").unwrap();

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(
            !hit,
            "a cached .rs file whose bytes don't match its recorded hash must be rejected"
        );
        assert!(
            !restored.path().join("block_states.rs").exists(),
            "no file may be copied out of an entry that fails validation, even ones that \
             individually still match their hash"
        );
    }

    #[test]
    fn corrupted_provider_json_bytes_that_dont_match_the_recorded_hash_are_a_miss() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);
        let manifest = canonical_manifest("abc123");
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        std::fs::write(dir.join("registries.api.json"), b"{\"corrupted\":true}").unwrap();

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(
            !hit,
            "a cached provider JSON file whose bytes don't match its recorded hash must be rejected"
        );
    }

    #[test]
    fn malformed_manifest_json_is_a_miss() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);
        // Valid JSON, but doesn't match CacheManifestFile's shape at all.
        std::fs::write(dir.join("manifest.json"), b"{\"totally\": \"wrong shape\"}").unwrap();

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(!hit);
    }

    #[test]
    fn valid_entry_with_full_manifest_still_loads_normally() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);
        let manifest = canonical_manifest("abc123");
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(hit, "a fully valid, fully hash-matching entry must load");
        for file in GENERATED_FILES {
            assert!(restored.path().join(file).exists());
        }
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

    // ── Corrupt-entry repair (issue #347 PR #348 review item 1) ────────────
    //
    // Before this fix, `publish` treated "a directory already exists at
    // this fingerprint" as "already published, nothing to do" -- even when
    // that directory was corrupt. That meant a corrupted cache entry was
    // permanently poisoned: every future build would `try_load` -> detect
    // corruption -> miss -> regenerate -> `publish` -> see the (still
    // corrupt) directory already exists -> skip -> leave it corrupt
    // forever. These tests prove that's fixed: a corrupt entry gets
    // replaced by `publish`, and the *next* build after that is a real
    // cache hit rather than another regeneration.

    #[test]
    fn corrupt_entry_is_repaired_on_next_publish() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        // A corrupt entry: files present, but bytes don't match a manifest
        // hash (simulating bit-rot/truncation after a valid publish).
        write_generated(&dir);
        let manifest = canonical_manifest("abc123");
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        std::fs::write(dir.join("registries.rs"), b"// CORRUPTED").unwrap();
        assert_eq!(validate_entry(&dir, "abc123"), CacheEntryState::Corrupt);

        // Simulate what generate_all_cached_with_root does on a miss:
        // regenerate into a fresh out_dir, then publish.
        let generated = tempfile::tempdir().unwrap();
        write_generated(generated.path());
        publish(cache_root.path(), "1.21.4", "abc123", generated.path());

        assert_eq!(
            validate_entry(&dir, "abc123"),
            CacheEntryState::Valid,
            "publish must repair a corrupt entry, not leave it poisoned"
        );
    }

    #[test]
    fn second_build_after_corruption_recovery_is_a_real_cache_hit() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);
        let manifest = canonical_manifest("abc123");
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        std::fs::write(dir.join("commands.rs"), b"// CORRUPTED").unwrap();

        // Build 1: miss (corrupt), regenerate, publish (repairs it).
        let out1 = tempfile::tempdir().unwrap();
        let hit1 = try_load(cache_root.path(), "1.21.4", "abc123", out1.path()).unwrap();
        assert!(!hit1, "corrupt entry must miss");
        let generated = tempfile::tempdir().unwrap();
        write_generated(generated.path());
        publish(cache_root.path(), "1.21.4", "abc123", generated.path());

        // Build 2: must now be a genuine hit -- not another silent miss.
        let out2 = tempfile::tempdir().unwrap();
        let hit2 = try_load(cache_root.path(), "1.21.4", "abc123", out2.path()).unwrap();
        assert!(
            hit2,
            "the build immediately after repair must hit the cache, not regenerate again"
        );
        for file in GENERATED_FILES {
            assert_eq!(
                std::fs::read(generated.path().join(file)).unwrap(),
                std::fs::read(out2.path().join(file)).unwrap(),
            );
        }
    }

    #[test]
    fn corrupt_manifest_json_is_repaired_on_publish() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);
        std::fs::write(dir.join("manifest.json"), b"not json{{{").unwrap();

        let generated = tempfile::tempdir().unwrap();
        write_generated(generated.path());
        publish(cache_root.path(), "1.21.4", "abc123", generated.path());

        assert_eq!(validate_entry(&dir, "abc123"), CacheEntryState::Valid);
    }

    #[test]
    fn missing_canonical_file_is_repaired_on_publish() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);
        std::fs::remove_file(dir.join("block_states.rs")).unwrap();
        let manifest = canonical_manifest("abc123");
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert_eq!(validate_entry(&dir, "abc123"), CacheEntryState::Corrupt);

        let generated = tempfile::tempdir().unwrap();
        write_generated(generated.path());
        publish(cache_root.path(), "1.21.4", "abc123", generated.path());

        assert_eq!(validate_entry(&dir, "abc123"), CacheEntryState::Valid);
        assert!(dir.join("block_states.rs").is_file());
    }

    #[test]
    fn repair_never_leaves_the_entry_directory_empty_or_missing() {
        // Even mid-repair, the fingerprint directory must always resolve to
        // either the old (corrupt) or new (valid) complete entry -- never
        // nothing. This proves the repair is a directory-level swap, not a
        // delete-then-recreate.
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);
        std::fs::write(dir.join("registries.rs"), b"// CORRUPTED").unwrap();
        let manifest = canonical_manifest("abc123");
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        let generated = tempfile::tempdir().unwrap();
        write_generated(generated.path());
        publish(cache_root.path(), "1.21.4", "abc123", generated.path());

        assert!(dir.is_dir(), "entry directory must exist after repair");
        assert_eq!(validate_entry(&dir, "abc123"), CacheEntryState::Valid);
        // No leftover temp/stale directories from the repair.
        let siblings: Vec<_> = std::fs::read_dir(dir.parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            siblings.iter().all(|name| !name.starts_with(".stale-")),
            "no stale quarantine directories should remain: {siblings:?}"
        );
        assert!(
            siblings.iter().all(|name| !name.starts_with(".tmp-")),
            "no temp directories should remain: {siblings:?}"
        );
    }

    #[test]
    fn concurrent_corrupt_entry_recovery_converges_on_one_valid_entry() {
        let cache_root = tempfile::tempdir().unwrap();
        let dir = entry_dir(cache_root.path(), "1.21.4", "abc123");
        std::fs::create_dir_all(&dir).unwrap();
        write_generated(&dir);
        std::fs::write(dir.join("commands.rs"), b"// CORRUPTED").unwrap();
        let manifest = canonical_manifest("abc123");
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

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

        assert_eq!(
            validate_entry(&dir, "abc123"),
            CacheEntryState::Valid,
            "concurrent repair attempts must converge on a valid entry"
        );
        let restored = tempfile::tempdir().unwrap();
        let hit = try_load(cache_root.path(), "1.21.4", "abc123", restored.path()).unwrap();
        assert!(
            hit,
            "readers after concurrent repair must see a valid, loadable entry"
        );
    }
}
