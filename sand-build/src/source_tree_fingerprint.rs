//! Recursive, deterministic, path-normalized fingerprint of a source
//! directory tree.
//!
//! Used to compute automatic implementation-identity fingerprints (e.g.
//! `sand-build/build.rs`'s `CODEGEN_IMPL_FINGERPRINT`) without a
//! manually-maintained file list, which a developer could forget to update
//! when adding a new generator source module (issue #347 PR #348 review
//! item 2).
//!
//! This file is included two ways:
//!
//! - As a normal module of the `sand-build` library (`mod
//!   source_tree_fingerprint;` in `lib.rs`), so it's unit-tested the same
//!   way as any other module.
//! - Standalone-compiled into `build.rs` via `#[path =
//!   "src/source_tree_fingerprint.rs"] mod source_tree_fingerprint;`,
//!   because `build.rs` cannot link against `sand-build`'s own not-yet-built
//!   library artifact (the classic build-script chicken-and-egg problem) --
//!   but it *can* pull in an extra sibling source file as its own separate
//!   compilation, as long as that file only depends on `std` plus `build.rs`'s
//!   own `[build-dependencies]` (here: `sha1`, `hex`, already present for
//!   exactly this purpose).
//!
//! Both inclusions compile the exact same code, so `cargo test`ing this
//! module *is* testing what `build.rs` actually runs -- not a parallel
//! reimplementation that could silently drift from it.

use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

/// Recursively discovers every `.rs` file under `root`, returning
/// `(root-relative path using `/` separators, absolute path)` pairs sorted
/// deterministically by the relative path string.
///
/// Sorting by the *relative* path (not the OS-dependent absolute
/// `PathBuf`) is what makes [`hash_source_tree`] produce the same result
/// regardless of where the repository/worktree is checked out, and using
/// `/` unconditionally (rather than `std::path::MAIN_SEPARATOR`) keeps it
/// consistent across platforms too.
pub fn discover_rust_files(root: &Path) -> std::io::Result<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();
    collect_rust_files(root, root, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

fn collect_rust_files(
    root: &Path,
    dir: &Path,
    files: &mut Vec<(String, PathBuf)>,
) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    // Deterministic traversal order regardless of the OS's directory
    // iteration order -- doesn't affect the final sort (relative path wins),
    // but keeps intermediate behavior deterministic and easy to reason
    // about too.
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(root, &path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let relative = path
                .strip_prefix(root)
                .expect("child of root has root as a prefix")
                .to_string_lossy()
                .replace('\\', "/");
            files.push((relative, path));
        }
    }
    Ok(())
}

/// Computes a deterministic fingerprint of every `.rs` file under `root`:
/// hex-encoded SHA-1 over the sorted `(relative path, content hash)` pairs,
/// each length-prefixed so path/content boundaries can never collide.
///
/// Including the relative path (not just file content) means adding,
/// removing, or renaming a source file changes the fingerprint even if
/// every existing file's *content* is untouched -- required so a new
/// generator module can't silently escape invalidating cached output built
/// from the old file set. Never includes `root`'s absolute path, so two
/// worktrees with identical source produce identical fingerprints
/// regardless of where each is checked out.
pub fn hash_source_tree(root: &Path) -> std::io::Result<String> {
    let files = discover_rust_files(root)?;
    let mut hasher = Sha1::new();
    for (relative, path) in &files {
        let bytes = std::fs::read(path)?;
        let content_hash = {
            let mut file_hasher = Sha1::new();
            file_hasher.update(&bytes);
            hex::encode(file_hasher.finalize())
        };
        hasher.update(relative.len().to_le_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(content_hash.len().to_le_bytes());
        hasher.update(content_hash.as_bytes());
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, rel: &str, content: &[u8]) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn deterministic_for_identical_trees() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        write(a.path(), "mod.rs", b"a");
        write(a.path(), "sub/x.rs", b"b");
        write(b.path(), "mod.rs", b"a");
        write(b.path(), "sub/x.rs", b"b");
        assert_eq!(
            hash_source_tree(a.path()).unwrap(),
            hash_source_tree(b.path()).unwrap()
        );
    }

    #[test]
    fn adding_a_source_file_changes_the_fingerprint() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "mod.rs", b"a");
        let before = hash_source_tree(dir.path()).unwrap();
        write(dir.path(), "new_module.rs", b"c");
        let after = hash_source_tree(dir.path()).unwrap();
        assert_ne!(
            before, after,
            "adding a new generator source file must change the fingerprint"
        );
    }

    #[test]
    fn removing_a_source_file_changes_the_fingerprint() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "mod.rs", b"a");
        write(dir.path(), "doomed.rs", b"b");
        let before = hash_source_tree(dir.path()).unwrap();
        std::fs::remove_file(dir.path().join("doomed.rs")).unwrap();
        let after = hash_source_tree(dir.path()).unwrap();
        assert_ne!(before, after);
    }

    #[test]
    fn content_change_changes_the_fingerprint() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "mod.rs", b"a");
        let before = hash_source_tree(dir.path()).unwrap();
        write(dir.path(), "mod.rs", b"a-but-different");
        let after = hash_source_tree(dir.path()).unwrap();
        assert_ne!(before, after);
    }

    #[test]
    fn renaming_a_file_changes_the_fingerprint_even_with_identical_total_content() {
        // Same bytes, different file layout -- must not collide, since the
        // relative path participates in the hash.
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        write(a.path(), "foo.rs", b"shared content");
        write(b.path(), "bar.rs", b"shared content");
        assert_ne!(
            hash_source_tree(a.path()).unwrap(),
            hash_source_tree(b.path()).unwrap()
        );
    }

    #[test]
    fn absolute_worktree_location_does_not_affect_the_fingerprint() {
        // Two independent temp directories (necessarily different absolute
        // paths -- tempdir() never reuses a path) with identical relative
        // structure and content must fingerprint identically. This is the
        // "different worktree checkout path" guarantee.
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        assert_ne!(
            a.path(),
            b.path(),
            "test requires genuinely different paths"
        );
        write(a.path(), "nested/deep/module.rs", b"content");
        write(b.path(), "nested/deep/module.rs", b"content");
        assert_eq!(
            hash_source_tree(a.path()).unwrap(),
            hash_source_tree(b.path()).unwrap()
        );
    }

    #[test]
    fn discovery_ignores_non_rust_files() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "mod.rs", b"a");
        write(dir.path(), "README.md", b"not rust");
        write(dir.path(), "data.json", b"{}");
        let files = discover_rust_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "mod.rs");
    }

    #[test]
    fn discovery_recurses_into_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "mod.rs", b"a");
        write(dir.path(), "sub/one.rs", b"b");
        write(dir.path(), "sub/deeper/two.rs", b"c");
        let files = discover_rust_files(dir.path()).unwrap();
        let names: Vec<&str> = files.iter().map(|(rel, _)| rel.as_str()).collect();
        assert_eq!(names, vec!["mod.rs", "sub/deeper/two.rs", "sub/one.rs"]);
    }

    #[test]
    fn discovery_order_is_deterministic_regardless_of_filesystem_iteration_order() {
        // Run discovery twice; the result must be identical both times
        // (sorted by relative path), which is what makes hash_source_tree
        // deterministic even though std::fs::read_dir's own order is not
        // guaranteed by the OS.
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "z.rs", b"1");
        write(dir.path(), "a.rs", b"2");
        write(dir.path(), "m/b.rs", b"3");
        let first = discover_rust_files(dir.path()).unwrap();
        let second = discover_rust_files(dir.path()).unwrap();
        assert_eq!(first, second);
        let names: Vec<&str> = first.iter().map(|(rel, _)| rel.as_str()).collect();
        assert_eq!(names, vec!["a.rs", "m/b.rs", "z.rs"]);
    }
}
