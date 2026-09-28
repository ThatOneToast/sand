//! Content-hash-addressed output manifest for `dist/` (issue #347 Phase 7).
//!
//! Tracks every file a build writes under one pack root (a datapack's
//! `dist/<namespace>/`)
//! so that:
//!
//! - an unchanged file is never rewritten (byte-identical content -> the
//!   file on disk keeps its old mtime, only the manifest is refreshed);
//! - a changed file is rewritten atomically (temp file + rename, so a crash
//!   mid-write can never leave a torn file at its final path);
//! - a file that stopped being generated (its record disappeared, e.g. a
//!   component was deleted from the project) is removed instead of being
//!   silently left behind as stale output.
//!
//! Correctness never depends on file mtimes — only on content hashes
//! ([`sand_build::fingerprint::hash_bytes`]), and a missing or corrupt
//! manifest falls back to "treat everything as new," which can only cost
//! extra writes. Stale deletion additionally requires a matching on-disk hash.
//! Transaction publication uses strict manifest loading and conflict checks.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use sand_build::fingerprint::hash_bytes;
use serde::{Deserialize, Serialize};

/// Bumped whenever the manifest's on-disk shape changes incompatibly. A
/// manifest with a different (or missing/unparsable) schema version is
/// treated as absent — see [`OutputManifest::load`].
const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Lives at the root of the pack directory the manifest describes, e.g.
/// `dist/<namespace>/.sand-build-manifest.json`.
pub const MANIFEST_FILE_NAME: &str = ".sand-build-manifest.json";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct ManifestFile {
    schema_version: u32,
    /// Pack-root-relative path (forward-slash separated) -> content hash.
    entries: BTreeMap<String, String>,
}

/// Write/unchanged/removed counts for one build, surfaced to `--timings`
/// and future `sand run` reload logic.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ChangeSummary {
    pub written: usize,
    pub unchanged: usize,
    pub removed: usize,
}

/// Accumulates one build's writes against the previous build's manifest for
/// a single pack root.
pub struct OutputManifest {
    root: PathBuf,
    previous: BTreeMap<String, String>,
    current: BTreeMap<String, String>,
    /// Paths this invocation actually wrote to disk, recorded at the moment
    /// [`OutputManifest::write_if_changed`] decides to write — not
    /// recomputed afterward from `previous`/`current` hash comparison,
    /// which cannot distinguish "hash unchanged, file also still on disk"
    /// from "hash unchanged, but the file had been deleted out-of-band and
    /// was just restored." Both are real writes; only the former is
    /// `unchanged`.
    written_paths: std::collections::BTreeSet<String>,
}

impl OutputManifest {
    /// Loads the manifest previously published at `root`, if any.
    ///
    /// A missing file, unreadable file, unparsable JSON, or mismatched
    /// schema version are all treated identically: "no prior knowledge."
    /// That's the required safe fallback for a corrupt/missing manifest —
    /// it degrades to a full rewrite of every output this build produces,
    /// never to incorrect output or an incorrect deletion.
    pub fn load(root: &Path) -> Self {
        let previous = std::fs::read(root.join(MANIFEST_FILE_NAME))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<ManifestFile>(&bytes).ok())
            .filter(|manifest| {
                manifest.schema_version == MANIFEST_SCHEMA_VERSION
                    && manifest
                        .entries
                        .keys()
                        .all(|path| validate_resource_path(path).is_ok())
            })
            .map(|manifest| manifest.entries)
            .unwrap_or_default();
        Self {
            root: root.to_path_buf(),
            previous,
            current: BTreeMap::new(),
            written_paths: std::collections::BTreeSet::new(),
        }
    }

    /// Writes `bytes` to `root.join(rel_path)`, skipping the write entirely
    /// when the previous manifest already recorded the same content hash
    /// for `rel_path` *and* the file still exists on disk (so a file
    /// deleted out-of-band by the user is still restored, and correctly
    /// counted as written — see [`ChangeSummary`]).
    ///
    /// `rel_path` must use `/` separators; it is the manifest key and is
    /// joined onto `root` verbatim via [`Path::join`], which accepts `/` on
    /// every platform Sand supports.
    pub fn write_if_changed(&mut self, rel_path: &str, bytes: &[u8]) -> Result<bool> {
        validate_resource_path(rel_path)?;
        reject_symlink_ancestors(&self.root.join(rel_path))?;
        let hash = hash_bytes(bytes);
        let dest = self.root.join(rel_path);
        let unchanged = self
            .previous
            .get(rel_path)
            .is_some_and(|prev| prev == &hash)
            && std::fs::read(&dest).is_ok_and(|actual| hash_bytes(&actual) == hash);
        self.current.insert(rel_path.to_string(), hash);
        if unchanged {
            return Ok(false);
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create dir for '{}'", dest.display()))?;
        }
        atomic_write(&dest, bytes)?;
        // Recorded here, at the moment of the actual write, rather than
        // reconstructed later from previous-vs-current hash comparison --
        // see the `written_paths` field docs for why that reconstruction is
        // wrong for the "hash unchanged but file was missing" case.
        self.written_paths.insert(rel_path.to_string());
        Ok(true)
    }

    /// Removes verified stale files before new paths can reuse them as directories.
    pub(super) fn prune_stale(&mut self, retained: &BTreeSet<String>) -> Result<usize> {
        let mut removed = 0usize;
        for rel_path in self.previous.keys() {
            if retained.contains(rel_path) {
                continue;
            }
            let path = self.root.join(rel_path);
            reject_symlink_ancestors(&path)?;
            if path.exists() {
                ensure!(
                    hash_bytes(&std::fs::read(&path)?) == self.previous[rel_path],
                    "stale managed output was modified: {rel_path}"
                );
                std::fs::remove_file(&path).with_context(|| {
                    format!("failed to remove stale output '{}'", path.display())
                })?;
            }
            removed += 1;
        }

        self.previous.retain(|path, _| retained.contains(path));
        Ok(removed)
    }

    /// Removes files the previous manifest tracked that were not written
    /// this build (no longer generated), then atomically publishes the new
    /// manifest. Returns a summary for diagnostics.
    ///
    /// Files this manifest never knew about (e.g. hand-placed by the user
    /// next to generated output, or written before manifests existed) are
    /// left alone — only previously-tracked, now-untracked entries are
    /// removed. That keeps a first build after introducing manifests safe:
    /// it cannot delete anything it didn't itself write and later drop.
    pub fn finish(mut self) -> Result<ChangeSummary> {
        let removed = self.prune_stale(&self.current.keys().cloned().collect())?;

        // `written` is exactly what write_if_changed recorded as actually
        // written this invocation -- never recomputed from hash comparison,
        // which would misclassify a restored-after-out-of-band-deletion
        // file (same hash, was missing, got rewritten) as unchanged.
        let written = self.written_paths.len();
        let unchanged = self.current.len() - written;

        let manifest = ManifestFile {
            schema_version: MANIFEST_SCHEMA_VERSION,
            entries: self.current,
        };
        let bytes =
            serde_json::to_vec_pretty(&manifest).context("failed to serialize output manifest")?;
        atomic_write(&self.root.join(MANIFEST_FILE_NAME), &bytes)?;

        Ok(ChangeSummary {
            written,
            unchanged,
            removed,
        })
    }
}

/// Validates a portable pack-relative path before it is joined to a root.
/// The ownership manifest is reserved and cannot itself be a generated resource.
pub(super) fn validate_resource_path(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty() && !path.contains(['\\', ':', '\0']),
        "unsafe output path: {path:?}"
    );
    ensure!(
        path.split('/')
            .all(|part| !part.is_empty() && part != "." && part != ".."),
        "unsafe output path: {path:?}"
    );
    ensure!(
        path.split('/').next() != Some(MANIFEST_FILE_NAME),
        "reserved output path: {path}"
    );
    Ok(())
}

/// Rejects symlinks even when they occur in an existing ancestor of a missing
/// destination. This is a preflight policy, not protection against concurrent
/// hostile filesystem mutation.
pub(crate) fn reject_symlink_ancestors(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(metadata) => ensure!(
                !metadata.file_type().is_symlink(),
                "symlink output path: {}",
                ancestor.display()
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("inspect output path '{}'", ancestor.display()));
            }
        }
    }
    Ok(())
}

/// Reads ownership strictly for transaction publication. Unlike legacy builds,
/// an existing corrupt manifest is an error: it cannot authorize replacement.
pub(super) fn read_ownership(root: &Path) -> Result<BTreeMap<String, String>> {
    let path = root.join(MANIFEST_FILE_NAME);
    reject_symlink_ancestors(&path)?;
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error).context("read output ownership manifest"),
    };
    let manifest: ManifestFile =
        serde_json::from_slice(&bytes).context("invalid output ownership manifest")?;
    ensure!(
        manifest.schema_version == MANIFEST_SCHEMA_VERSION,
        "unsupported output ownership manifest revision"
    );
    for (path, hash) in &manifest.entries {
        validate_resource_path(path)?;
        if hash.len() != hash_bytes(b"").len() || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            bail!("invalid ownership hash for {path}");
        }
    }
    Ok(manifest.entries)
}

/// Writes `bytes` to `dest` via a same-directory temp file followed by a
/// rename, so concurrent readers (or a process crash) never observe a
/// partially written file at `dest`. The temp file lives next to `dest` so
/// the rename stays within one filesystem (required for it to be atomic).
fn atomic_write(dest: &Path, bytes: &[u8]) -> Result<()> {
    reject_symlink_ancestors(dest)?;
    let parent = dest
        .parent()
        .with_context(|| format!("output path '{}' has no parent", dest.display()))?;
    let file_name = dest
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("output path '{}' has no file name", dest.display()))?;
    let (tmp_path, mut file) = loop {
        let path = parent.join(format!(
            ".{file_name}.sand-tmp-{}-{}",
            std::process::id(),
            tmp_nonce()
        ));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => break (path, file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("create temporary output for '{}'", dest.display()));
            }
        }
    };
    if let Err(error) = std::io::Write::write_all(&mut file, bytes) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(error)
            .with_context(|| format!("write temporary output for '{}'", dest.display()));
    }
    drop(file);
    std::fs::rename(&tmp_path, dest).with_context(|| {
        // Best-effort cleanup so a rename failure doesn't leave the temp
        // file behind forever; ignore the cleanup's own result since the
        // rename error is what actually matters to the caller.
        let _ = std::fs::remove_file(&tmp_path);
        format!("failed to publish '{}'", dest.display())
    })?;
    Ok(())
}

/// A small per-process counter, not a cryptographic nonce: it only needs to
/// keep same-process concurrent temp file names from colliding, alongside
/// the process id which already separates concurrent processes/worktrees.
fn tmp_nonce() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_build_writes_every_file_and_reports_none_unchanged() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let mut manifest = OutputManifest::load(dir.path());

        assert!(manifest.write_if_changed("a.txt", b"one").unwrap());
        assert!(manifest.write_if_changed("nested/b.txt", b"two").unwrap());

        let summary = manifest.finish().unwrap();
        assert_eq!(summary.written, 2);
        assert_eq!(summary.unchanged, 0);
        assert_eq!(summary.removed, 0);
        assert_eq!(std::fs::read(dir.path().join("a.txt")).unwrap(), b"one");
        assert_eq!(
            std::fs::read(dir.path().join("nested/b.txt")).unwrap(),
            b"two"
        );
    }

    #[test]
    fn identical_rebuild_touches_no_files() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let mut first = OutputManifest::load(dir.path());
        first.write_if_changed("a.txt", b"one").unwrap();
        first.finish().unwrap();

        let mtime_before = std::fs::metadata(dir.path().join("a.txt"))
            .unwrap()
            .modified()
            .unwrap();
        // Ensure the filesystem's mtime resolution can't hide a rewrite.
        std::thread::sleep(std::time::Duration::from_millis(20));

        let mut second = OutputManifest::load(dir.path());
        let wrote = second.write_if_changed("a.txt", b"one").unwrap();
        let summary = second.finish().unwrap();

        assert!(!wrote, "identical content must not be rewritten");
        assert_eq!(summary.written, 0);
        assert_eq!(summary.unchanged, 1);
        let mtime_after = std::fs::metadata(dir.path().join("a.txt"))
            .unwrap()
            .modified()
            .unwrap();
        assert_eq!(
            mtime_before, mtime_after,
            "unchanged file must keep its original mtime"
        );
    }

    /// Regression test (issue #347 PR #348 review item 3): an output file
    /// deleted out-of-band (not through this manifest -- e.g. the user ran
    /// `rm` or a cleanup script) must be restored on the next build *and*
    /// counted as `written`, never as `unchanged`, even though its content
    /// hash is identical to what the manifest last recorded.
    #[test]
    fn restoring_an_out_of_band_deleted_file_counts_as_written_not_unchanged() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let mut first = OutputManifest::load(dir.path());
        first.write_if_changed("a.txt", b"one").unwrap();
        first.write_if_changed("b.txt", b"two").unwrap();
        first.finish().unwrap();

        // Out-of-band deletion: not through the manifest API.
        std::fs::remove_file(dir.path().join("a.txt")).unwrap();
        assert!(!dir.path().join("a.txt").exists());

        let mut second = OutputManifest::load(dir.path());
        let wrote_a = second.write_if_changed("a.txt", b"one").unwrap();
        let wrote_b = second.write_if_changed("b.txt", b"two").unwrap();
        let summary = second.finish().unwrap();

        assert!(wrote_a, "a missing file must be restored (a real write)");
        assert!(
            !wrote_b,
            "b.txt was never touched, so it truly is unchanged"
        );
        assert!(dir.path().join("a.txt").exists());
        assert_eq!(
            std::fs::read(dir.path().join("a.txt")).unwrap(),
            b"one",
            "restored content must match"
        );
        assert_eq!(
            summary.written, 1,
            "the restored file must be counted as written, not unchanged"
        );
        assert_eq!(
            summary.unchanged, 1,
            "the untouched file must still be counted as unchanged"
        );
        assert_eq!(summary.removed, 0);
    }

    #[test]
    fn matching_manifest_hash_does_not_hide_modified_file_bytes() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let mut first = OutputManifest::load(dir.path());
        first.write_if_changed("a.txt", b"generated").unwrap();
        first.finish().unwrap();
        std::fs::write(dir.path().join("a.txt"), b"modified").unwrap();

        let mut next = OutputManifest::load(dir.path());
        assert!(next.write_if_changed("a.txt", b"generated").unwrap());
        assert_eq!(next.finish().unwrap().written, 1);
        assert_eq!(
            std::fs::read(dir.path().join("a.txt")).unwrap(),
            b"generated"
        );
    }

    #[test]
    fn changed_content_is_rewritten() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let mut first = OutputManifest::load(dir.path());
        first.write_if_changed("a.txt", b"one").unwrap();
        first.finish().unwrap();

        let mut second = OutputManifest::load(dir.path());
        let wrote = second.write_if_changed("a.txt", b"two").unwrap();
        let summary = second.finish().unwrap();

        assert!(wrote);
        assert_eq!(summary.written, 1);
        assert_eq!(summary.unchanged, 0);
        assert_eq!(std::fs::read(dir.path().join("a.txt")).unwrap(), b"two");
    }

    #[test]
    fn no_longer_generated_file_is_removed() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let mut first = OutputManifest::load(dir.path());
        first.write_if_changed("a.txt", b"one").unwrap();
        first.write_if_changed("b.txt", b"two").unwrap();
        first.finish().unwrap();
        assert!(dir.path().join("b.txt").exists());

        // Second build only produces a.txt -- b.txt's component was removed.
        let mut second = OutputManifest::load(dir.path());
        second.write_if_changed("a.txt", b"one").unwrap();
        let summary = second.finish().unwrap();

        assert_eq!(summary.removed, 1);
        assert!(!dir.path().join("b.txt").exists());
        assert!(dir.path().join("a.txt").exists());
    }

    #[test]
    fn missing_manifest_falls_back_to_full_write_without_deleting_anything() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        // A file already exists on disk that the (nonexistent) manifest
        // never tracked -- e.g. a hand-placed file, or output from before
        // manifests existed.
        std::fs::write(dir.path().join("untracked.txt"), b"keep me").unwrap();

        let mut manifest = OutputManifest::load(dir.path());
        manifest.write_if_changed("a.txt", b"one").unwrap();
        let summary = manifest.finish().unwrap();

        assert_eq!(summary.removed, 0, "untracked files must not be deleted");
        assert!(dir.path().join("untracked.txt").exists());
        assert_eq!(
            std::fs::read(dir.path().join("untracked.txt")).unwrap(),
            b"keep me"
        );
    }

    #[test]
    fn corrupt_manifest_falls_back_safely_instead_of_failing() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        std::fs::write(dir.path().join(MANIFEST_FILE_NAME), b"not json at all{{{").unwrap();

        let mut manifest = OutputManifest::load(dir.path());
        let wrote = manifest.write_if_changed("a.txt", b"one").unwrap();
        let summary = manifest.finish().unwrap();

        assert!(wrote);
        assert_eq!(summary.written, 1);
    }

    #[test]
    fn mismatched_schema_version_is_treated_as_absent() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let stale = serde_json::json!({
            "schema_version": MANIFEST_SCHEMA_VERSION + 1,
            "entries": { "a.txt": "deadbeef" },
        });
        std::fs::write(
            dir.path().join(MANIFEST_FILE_NAME),
            serde_json::to_vec(&stale).unwrap(),
        )
        .unwrap();
        // Content on disk differs from what a real build with that hash
        // would have produced, proving the stale entry was not trusted.
        std::fs::write(dir.path().join("a.txt"), b"one").unwrap();

        let mut manifest = OutputManifest::load(dir.path());
        let wrote = manifest.write_if_changed("a.txt", b"one").unwrap();

        assert!(
            wrote,
            "a schema-mismatched manifest must not be trusted for skip decisions"
        );
    }

    #[test]
    fn manifest_round_trips_through_disk() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let mut first = OutputManifest::load(dir.path());
        first.write_if_changed("a.txt", b"one").unwrap();
        first.finish().unwrap();

        let raw = std::fs::read_to_string(dir.path().join(MANIFEST_FILE_NAME)).unwrap();
        let parsed: ManifestFile = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed.schema_version, MANIFEST_SCHEMA_VERSION);
        assert_eq!(parsed.entries.get("a.txt").unwrap(), &hash_bytes(b"one"));
    }
}
