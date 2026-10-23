//! Portable content polling: renames, deletion and equal-mtime edits all count.
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub(super) type Snapshot = BTreeMap<PathBuf, String>;

pub(super) fn snapshot(root: &Path) -> Result<Snapshot> {
    snapshot_excluding(root, &[])
}

pub(super) fn snapshot_excluding(root: &Path, targets: &[PathBuf]) -> Result<Snapshot> {
    let root = root.canonicalize()?;
    let targets: Vec<_> = targets
        .iter()
        .map(|path| path.canonicalize().unwrap_or_else(|_| path.to_path_buf()))
        .collect();
    let mut files = BTreeMap::new();
    for entry in walkdir::WalkDir::new(&root).into_iter().filter_entry(|e| {
        !targets.iter().any(|target| target == e.path())
            && (e.depth() != 1
                || !matches!(e.file_name().to_str(), Some("target" | "dist" | ".git")))
    }) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.insert(
                entry.path().strip_prefix(&root)?.to_owned(),
                sand_build::fingerprint::hash_bytes(&std::fs::read(entry.path())?),
            );
        }
    }
    Ok(files)
}

pub(super) struct Watch {
    previous: Snapshot,
    changed: Option<Instant>,
}
impl Watch {
    pub fn new(previous: Snapshot) -> Self {
        Self {
            previous,
            changed: None,
        }
    }
    pub fn observe(&mut self, current: Snapshot, now: Instant) {
        if self.previous != current {
            self.previous = current;
            self.changed = Some(now);
        }
    }
    pub fn request(&mut self, now: Instant) {
        self.changed = Some(now);
    }
    pub fn take_ready(&mut self, now: Instant) -> bool {
        if self
            .changed
            .is_some_and(|t| now.duration_since(t) >= Duration::from_millis(500))
        {
            self.changed = None;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolved_custom_target_is_excluded_without_hiding_nested_source_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src/build")).unwrap();
        for relative in ["build", "artifacts/cargo-output"] {
            let target = root.join(relative);
            std::fs::create_dir_all(&target).unwrap();
            let before = snapshot_excluding(root, std::slice::from_ref(&target)).unwrap();
            std::fs::write(target.join("compiler-output"), "generated").unwrap();
            assert_eq!(
                before,
                snapshot_excluding(root, std::slice::from_ref(&target)).unwrap()
            );
            std::fs::write(root.join("src/build/mod.rs"), relative).unwrap();
            assert_ne!(
                before,
                snapshot_excluding(root, std::slice::from_ref(&target)).unwrap()
            );
        }
    }

    #[test]
    fn source_target_and_nested_resource_dist_are_watched() {
        let tmp = tempfile::tempdir().unwrap();
        for name in ["src/target", "resources/dist"] {
            std::fs::create_dir_all(tmp.path().join(name)).unwrap();
        }
        let mut watch = Watch::new(snapshot(tmp.path()).unwrap());
        let now = Instant::now();
        std::fs::write(tmp.path().join("src/target/mod.rs"), "// edit").unwrap();
        watch.observe(snapshot(tmp.path()).unwrap(), now);
        assert!(watch.take_ready(now + Duration::from_secs(1)));
        std::fs::write(tmp.path().join("resources/dist/texture.png"), b"asset").unwrap();
        watch.observe(snapshot(tmp.path()).unwrap(), now + Duration::from_secs(2));
        assert!(watch.take_ready(now + Duration::from_secs(3)));
    }

    #[test]
    fn edits_deletes_and_renames_debounce_without_watching_outputs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::write(root.join("sand.toml"), "one").unwrap();
        let initial = snapshot(root).unwrap();
        std::fs::create_dir(root.join("dist")).unwrap();
        std::fs::write(root.join("dist/generated"), "ignored").unwrap();
        assert_eq!(initial, snapshot(root).unwrap());
        let mut watch = Watch::new(initial);
        let now = Instant::now();
        std::fs::write(root.join("sand.toml"), "two").unwrap();
        watch.observe(snapshot(root).unwrap(), now);
        assert!(!watch.take_ready(now + Duration::from_millis(499)));
        std::fs::rename(root.join("sand.toml"), root.join("new.toml")).unwrap();
        watch.observe(snapshot(root).unwrap(), now + Duration::from_millis(400));
        assert!(!watch.take_ready(now + Duration::from_millis(800)));
        assert!(watch.take_ready(now + Duration::from_millis(900)));
        assert!(!watch.take_ready(now + Duration::from_secs(2)));
        std::fs::remove_file(root.join("new.toml")).unwrap();
        watch.observe(snapshot(root).unwrap(), now + Duration::from_secs(3));
        assert!(watch.take_ready(now + Duration::from_secs(4)));
    }
}
