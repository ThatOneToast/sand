//! Shared pack publication transaction for Rust builds and portable programs.
//!
//! Resources are validated before touching output. A sibling staging copy keeps
//! unrelated files and uses the ordinary output manifest writer. Directory
//! replacement rolls back on ordinary publication errors. Crash atomicity and
//! concurrent writers are deliberately not promised; callers must serialize
//! publication to a destination.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result, ensure};
use sand_build::fingerprint::hash_bytes;

use super::output_manifest::{
    ChangeSummary, OutputManifest, read_ownership, reject_symlink_ancestors, validate_resource_path,
};

/// Publishes a complete, already compiled pack and preserves unrelated files.
///
/// Existing generated files must still match their ownership hashes. An
/// unmanaged file at a generated path is a conflict, even if bytes match.
/// Symlinks, non-regular files, unsafe relative paths and corrupt manifests are
/// rejected. On an ordinary staging or installation error the prior pack is
/// retained (or restored); if restoration itself fails, the error names the
/// backup that must be recovered. No crash or concurrent-writer atomicity is
/// claimed. The returned counts describe generated resources only.
pub fn publish_pack(
    destination: &Path,
    resources: &BTreeMap<String, Vec<u8>>,
) -> Result<ChangeSummary> {
    publish_with_install(destination, resources, |stage, destination| {
        std::fs::rename(stage, destination).context("install staged pack")
    })
}

fn publish_with_install(
    destination: &Path,
    resources: &BTreeMap<String, Vec<u8>>,
    install: impl FnOnce(&Path, &Path) -> Result<()>,
) -> Result<ChangeSummary> {
    // Resolve relative paths explicitly so that a bare directory name has a
    // usable parent, and reject lexical traversal before any filesystem work.
    ensure!(
        !destination
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir)),
        "output destination cannot contain '..'"
    );
    let destination = if destination.is_absolute() {
        destination.to_path_buf()
    } else {
        std::env::current_dir()?.join(destination)
    };
    ensure!(
        destination.file_name().is_some(),
        "output destination must name a directory"
    );
    reject_symlink_ancestors(&destination)?;
    let parent = destination
        .parent()
        .context("output destination has no parent")?;
    for path in resources.keys() {
        validate_resource_path(path)?;
        for ancestor in Path::new(path).ancestors().skip(1) {
            if let Some(ancestor) = ancestor.to_str() {
                ensure!(
                    !resources.contains_key(ancestor),
                    "output file/directory collision: {ancestor}"
                );
            }
        }
    }
    let previous = read_ownership(&destination)?;
    if destination.exists() {
        ensure!(
            destination.is_dir(),
            "output destination is not a directory"
        );
        inspect_tree(&destination)?;
    }
    // Verify all managed files, including stale ones, before copying or writing.
    for (path, hash) in &previous {
        let actual = destination.join(path);
        if actual.exists() {
            ensure!(actual.is_file(), "managed output is not a file: {path}");
            ensure!(
                hash_bytes(&std::fs::read(&actual)?) == *hash,
                "managed output was modified: {path}"
            );
        }
    }
    for path in resources.keys() {
        let actual = destination.join(path);
        if actual.is_dir() {
            // Only directories implied by verified stale files are owned.
            // Unmanaged files and even unrelated empty directories must survive.
            for entry in walkdir::WalkDir::new(&actual).follow_links(false) {
                let entry = entry.context("inspect directory-to-file transition")?;
                let relative = entry
                    .path()
                    .strip_prefix(&destination)?
                    .to_str()
                    .context("output path is not UTF-8")?;
                let owned = if entry.file_type().is_dir() {
                    let prefix = format!("{relative}/");
                    previous
                        .keys()
                        .any(|old| old.starts_with(&prefix) && !resources.contains_key(old))
                } else {
                    previous.contains_key(relative) && !resources.contains_key(relative)
                };
                ensure!(owned, "unmanaged output conflict: {relative}");
            }
        }
        ensure!(
            !actual.exists() || actual.is_dir() || previous.contains_key(path),
            "unmanaged output conflict: {path}"
        );
        for ancestor in actual.ancestors().skip(1) {
            if ancestor == destination {
                break;
            }
            ensure!(
                !ancestor.exists()
                    || ancestor.is_dir()
                    || ancestor
                        .strip_prefix(&destination)
                        .ok()
                        .and_then(Path::to_str)
                        .is_some_and(
                            |path| previous.contains_key(path) && !resources.contains_key(path)
                        ),
                "output parent is not a directory: {}",
                ancestor.display()
            );
        }
    }
    std::fs::create_dir_all(parent).context("create output parent")?;
    let transaction = create_transaction(parent)?;
    let stage = transaction.join("stage");
    let backup = transaction.join("previous");
    let mut rollback_failed = false;
    let result = (|| {
        std::fs::create_dir(&stage)?;
        if destination.exists() {
            copy_tree(&destination, &stage)?;
        }
        let mut manifest = OutputManifest::load(&stage);
        let removed = manifest.prune_stale(&resources.keys().cloned().collect())?;
        for (path, bytes) in resources {
            let actual = stage.join(path);
            if actual.is_dir() {
                // Stale files were hash-checked and removed above. Remove only
                // empty directories; never recursively erase remaining files.
                for entry in walkdir::WalkDir::new(&actual).contents_first(true) {
                    let entry = entry?;
                    std::fs::remove_dir(entry.path())?;
                }
            }
            manifest.write_if_changed(path, bytes)?;
        }
        let mut summary = manifest.finish()?;
        summary.removed += removed;
        let had_previous = destination.exists();
        if had_previous {
            std::fs::rename(&destination, &backup).context("back up previous pack")?;
        }
        if let Err(error) = install(&stage, &destination) {
            if had_previous {
                // Installation only renames a directory and must not partially
                // create a destination on error.
                if let Err(rollback) = std::fs::rename(&backup, &destination) {
                    rollback_failed = true;
                    return Err(error).context(format!(
                        "rollback failed ({rollback}); recover previous pack from '{}'",
                        backup.display()
                    ));
                }
            }
            return Err(error);
        }
        Ok(summary)
    })();
    // Never erase the sole surviving good pack after a rollback failure.
    if !rollback_failed {
        let _ = std::fs::remove_dir_all(&transaction);
    }
    result
}

fn create_transaction(parent: &Path) -> Result<std::path::PathBuf> {
    static NONCE: AtomicU64 = AtomicU64::new(0);
    loop {
        let candidate = parent.join(format!(
            ".sand-publish-{}-{}",
            std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error).context("create sibling output staging directory"),
        }
    }
}

fn inspect_tree(root: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.context("inspect existing pack")?;
        ensure!(
            entry.file_type().is_dir() || entry.file_type().is_file(),
            "output contains symlink or special file: {}",
            entry.path().display()
        );
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(source)
        .min_depth(1)
        .follow_links(false)
    {
        let entry = entry.context("copy existing pack")?;
        let target = destination.join(entry.path().strip_prefix(source)?);
        if entry.file_type().is_dir() {
            std::fs::create_dir(&target)?;
        } else {
            ensure!(entry.file_type().is_file(), "output changed during staging");
            let mut source_file = std::fs::File::open(entry.path())?;
            let metadata = source_file.metadata()?;
            let mut target_file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)?;
            std::io::copy(&mut source_file, &mut target_file)
                .context("copy existing output file")?;
            // Apply metadata while the writable creation handle is still open,
            // before restoring permissions that may make the copy read-only.
            target_file.set_times(std::fs::FileTimes::new().set_modified(metadata.modified()?))?;
            target_file.set_permissions(metadata.permissions())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::output_manifest::MANIFEST_FILE_NAME;
    use super::*;

    fn resources(entries: &[(&str, &str)]) -> BTreeMap<String, Vec<u8>> {
        entries
            .iter()
            .map(|(path, body)| (path.to_string(), body.as_bytes().to_vec()))
            .collect()
    }

    #[test]
    fn rebuild_preserves_unrelated_files_and_prunes_owned_stale_files() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let pack = temp.path().join("pack");
        publish_pack(&pack, &resources(&[("data/a", "one"), ("stale", "old")])).unwrap();
        let modified = std::fs::metadata(pack.join("data/a"))
            .unwrap()
            .modified()
            .unwrap();
        std::fs::write(pack.join("notes"), b"mine").unwrap();
        let summary =
            publish_pack(&pack, &resources(&[("data/a", "one"), ("new", "two")])).unwrap();
        assert_eq!(
            summary,
            ChangeSummary {
                written: 1,
                unchanged: 1,
                removed: 1
            }
        );
        assert!(!pack.join("stale").exists());
        assert_eq!(std::fs::read(pack.join("notes")).unwrap(), b"mine");
        assert_eq!(
            modified,
            std::fs::metadata(pack.join("data/a"))
                .unwrap()
                .modified()
                .unwrap()
        );
    }

    #[test]
    fn managed_file_can_become_a_directory_without_changing_pack_on_failure() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let pack = temp.path().join("pack");
        let old = resources(&[("data/demo/function/foo.mcfunction", "say old")]);
        let new = resources(&[(
            "data/demo/function/foo.mcfunction/bar.mcfunction",
            "say new",
        )]);
        publish_pack(&pack, &old).unwrap();
        let manifest = std::fs::read(pack.join(MANIFEST_FILE_NAME)).unwrap();
        assert!(
            publish_with_install(&pack, &new, |stage, _| {
                assert_eq!(
                    std::fs::read(stage.join(new.keys().next().unwrap()))?,
                    b"say new"
                );
                anyhow::bail!("injected failure")
            })
            .is_err()
        );
        assert_eq!(
            std::fs::read(pack.join(old.keys().next().unwrap())).unwrap(),
            b"say old"
        );
        assert_eq!(
            std::fs::read(pack.join(MANIFEST_FILE_NAME)).unwrap(),
            manifest
        );
        let summary = publish_pack(&pack, &new).unwrap();
        assert_eq!(summary.removed, 1);
        assert_eq!(summary.written, 1);
        assert_eq!(
            std::fs::read(pack.join(new.keys().next().unwrap())).unwrap(),
            b"say new"
        );
    }

    #[test]
    fn managed_directory_can_become_a_file_with_rollback_and_conflict_protection() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let pack = temp.path().join("pack");
        let parent = "data/demo/function/foo.mcfunction";
        let child = "data/demo/function/foo.mcfunction/nested/bar.mcfunction";
        let old = resources(&[(child, "say old")]);
        let new = resources(&[(parent, "say new")]);
        publish_pack(&pack, &old).unwrap();
        let manifest = std::fs::read(pack.join(MANIFEST_FILE_NAME)).unwrap();
        for directory in [false, true] {
            let unrelated = pack.join(parent).join("unmanaged");
            if directory {
                std::fs::create_dir(&unrelated).unwrap();
            } else {
                std::fs::write(&unrelated, "mine").unwrap();
            }
            assert!(
                publish_pack(&pack, &new)
                    .unwrap_err()
                    .to_string()
                    .contains("unmanaged")
            );
            assert!(unrelated.exists());
            if directory {
                std::fs::remove_dir(unrelated).unwrap();
            } else {
                std::fs::remove_file(unrelated).unwrap();
            }
        }
        assert!(
            publish_with_install(&pack, &new, |stage, _| {
                assert_eq!(std::fs::read(stage.join(parent))?, b"say new");
                anyhow::bail!("injected failure")
            })
            .is_err()
        );
        assert_eq!(std::fs::read(pack.join(child)).unwrap(), b"say old");
        assert_eq!(
            std::fs::read(pack.join(MANIFEST_FILE_NAME)).unwrap(),
            manifest
        );
        let summary = publish_pack(&pack, &new).unwrap();
        assert_eq!(summary.removed, 1);
        assert_eq!(summary.written, 1);
        assert_eq!(std::fs::read(pack.join(parent)).unwrap(), b"say new");
    }

    #[test]
    fn rebuild_preserves_read_only_unrelated_files_and_their_metadata() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let pack = temp.path().join("pack");
        publish_pack(&pack, &resources(&[("owned", "one")])).unwrap();
        let note = pack.join("notes");
        std::fs::write(&note, b"keep").unwrap();
        let mut permissions = std::fs::metadata(&note).unwrap().permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&note, permissions).unwrap();
        let before = std::fs::metadata(&note).unwrap();
        publish_pack(&pack, &resources(&[("owned", "two")])).unwrap();
        assert_eq!(std::fs::read(&note).unwrap(), b"keep");
        let after = std::fs::metadata(&note).unwrap();
        assert_eq!(after.modified().unwrap(), before.modified().unwrap());
        assert_eq!(after.permissions(), before.permissions());
    }

    #[test]
    fn unmanaged_and_modified_files_are_never_overwritten_or_deleted() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let pack = temp.path().join("pack");
        publish_pack(&pack, &resources(&[("owned", "one")])).unwrap();
        std::fs::write(pack.join("notes"), b"mine").unwrap();
        assert!(publish_pack(&pack, &resources(&[("notes", "mine")])).is_err());
        std::fs::write(pack.join("owned"), b"edited").unwrap();
        assert!(publish_pack(&pack, &resources(&[("owned", "two")])).is_err());
        assert!(publish_pack(&pack, &BTreeMap::new()).is_err());
        assert_eq!(std::fs::read(pack.join("owned")).unwrap(), b"edited");
    }

    #[test]
    fn failed_install_restores_previous_pack_and_manifest() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let pack = temp.path().join("pack");
        publish_pack(&pack, &resources(&[("owned", "one")])).unwrap();
        let manifest = std::fs::read(pack.join(MANIFEST_FILE_NAME)).unwrap();
        let error = publish_with_install(
            &pack,
            &resources(&[("owned", "two")]),
            |stage, destination| {
                assert_eq!(std::fs::read(stage.join("owned")).unwrap(), b"two");
                assert!(
                    !destination.exists(),
                    "failure occurs after old pack has been backed up"
                );
                anyhow::bail!("injected installation rename failure")
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("injected"));
        assert_eq!(std::fs::read(pack.join("owned")).unwrap(), b"one");
        assert_eq!(
            std::fs::read(pack.join(MANIFEST_FILE_NAME)).unwrap(),
            manifest
        );
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn failed_rollback_keeps_recoverable_backup_even_if_destination_exists() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let pack = temp.path().join("pack");
        publish_pack(&pack, &resources(&[("owned", "one")])).unwrap();
        let error =
            publish_with_install(&pack, &resources(&[("owned", "two")]), |_, destination| {
                // Simulate another writer blocking rollback after the failed rename.
                std::fs::create_dir(destination)?;
                std::fs::write(destination.join("obstruction"), b"other writer")?;
                anyhow::bail!("injected installation failure")
            })
            .unwrap_err();
        assert!(error.to_string().contains("rollback failed"));
        let transaction = std::fs::read_dir(temp.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".sand-publish-")
            })
            .unwrap();
        assert_eq!(
            std::fs::read(transaction.join("previous/owned")).unwrap(),
            b"one"
        );
    }

    #[test]
    fn unsafe_resources_and_manifest_entries_cannot_escape_root() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let pack = temp.path().join("pack");
        for path in [
            "../victim",
            "/absolute",
            "a//b",
            "a/./b",
            "a\\b",
            "C:/file",
            MANIFEST_FILE_NAME,
        ] {
            assert!(
                publish_pack(&pack, &resources(&[(path, "bad")])).is_err(),
                "{path}"
            );
        }
        publish_pack(&pack, &resources(&[("owned", "one")])).unwrap();
        std::fs::write(temp.path().join("victim"), b"safe").unwrap();
        std::fs::write(
            pack.join(MANIFEST_FILE_NAME),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1, "entries": {"../victim": hash_bytes(b"safe")}
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(publish_pack(&pack, &BTreeMap::new()).is_err());
        assert_eq!(std::fs::read(temp.path().join("victim")).unwrap(), b"safe");
    }

    #[test]
    fn file_directory_collisions_fail_without_changing_existing_pack() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let pack = temp.path().join("pack");
        publish_pack(&pack, &resources(&[("owned", "one")])).unwrap();
        assert!(publish_pack(&pack, &resources(&[("a", "one"), ("a/b", "two")])).is_err());
        std::fs::write(pack.join("unmanaged"), b"mine").unwrap();
        assert!(publish_pack(&pack, &resources(&[("unmanaged/child", "two")])).is_err());
        assert_eq!(std::fs::read(pack.join("owned")).unwrap(), b"one");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_destinations_ancestors_and_descendants_are_rejected() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let pack = temp.path().join("pack");
        publish_pack(&pack, &resources(&[("owned", "one")])).unwrap();
        symlink(&pack, temp.path().join("link")).unwrap();
        assert!(publish_pack(&temp.path().join("link"), &BTreeMap::new()).is_err());
        assert!(publish_pack(&temp.path().join("link/child"), &BTreeMap::new()).is_err());
        symlink(temp.path().join("absent"), pack.join("dangling")).unwrap();
        assert!(publish_pack(&pack, &BTreeMap::new()).is_err());
        assert_eq!(std::fs::read(pack.join("owned")).unwrap(), b"one");
    }
}
