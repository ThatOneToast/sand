use super::*;
use crate::download::tests::{Fake, fixture};

fn manifest(entry: &VersionEntry) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"latest": {"release": "99.9", "snapshot": "99.9"}, "versions": [entry]})).unwrap()
}

#[test]
fn pinned_and_bundled_latest_warm_outage_preserve_complete_entry() {
    let root = tempfile::tempdir().unwrap();
    let (mut entry, _) = fixture(b"jar");
    entry.id = sand_version::LATEST_KNOWN.into();
    std::fs::write(
        root.path().join("download-manifest-v1.json"),
        manifest(&entry),
    )
    .unwrap();
    for requested in [entry.id.as_str(), "latest"] {
        let fake = Fake::new();
        let got = crate::resolve_version(root.path(), requested, &fake).unwrap();
        assert_eq!(got.id, entry.id);
        assert_eq!(got.url, entry.url);
        assert_eq!(got.sha1, entry.sha1);
        assert!(fake.calls.borrow().is_empty());
    }
    assert_eq!(crate::latest_release_version(), sand_version::LATEST_KNOWN);
}

#[test]
fn imports_legacy_manifest_once_and_never_overwrites_legacy() {
    let root = tempfile::tempdir().unwrap();
    let (entry, _) = fixture(b"jar");
    let bytes = manifest(&entry);
    let legacy = root.path().join("version_manifest_v2.json");
    std::fs::write(&legacy, &bytes).unwrap();
    resolve(root.path(), &entry.id, &Fake::new()).unwrap();
    assert_eq!(std::fs::read(&legacy).unwrap(), bytes);
    // Legacy changing later does not retarget the independent manifest.
    let mut changed = entry.clone();
    changed.url.push_str("?changed");
    std::fs::write(&legacy, manifest(&changed)).unwrap();
    assert_eq!(
        resolve(root.path(), &entry.id, &Fake::new()).unwrap().url,
        entry.url
    );
    std::fs::write(root.path().join("download-manifest-v1.json"), b"bad").unwrap();
    let fake = Fake::new();
    fake.fail(MANIFEST_URL);
    assert!(resolve(root.path(), &entry.id, &fake).is_err());
}

#[test]
fn fresh_selected_identity_replaces_only_after_complete_validation() {
    let root = tempfile::tempdir().unwrap();
    let (entry, _) = fixture(b"old");
    let path = root.path().join("download-manifest-v1.json");
    let old = manifest(&entry);
    std::fs::write(&path, &old).unwrap();
    let fake = Fake::new();
    fake.push(MANIFEST_URL, b"truncated");
    assert!(resolve(root.path(), "26.3", &fake).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), old);
    let (mut changed, _) = fixture(b"new");
    changed.id = "26.3".into();
    fake.push(MANIFEST_URL, &manifest(&changed));
    assert_eq!(
        resolve(root.path(), "26.3", &fake).unwrap().sha1,
        changed.sha1
    );
    assert_eq!(std::fs::read(&path).unwrap(), manifest(&changed));
}

#[test]
fn absent_anchor_does_not_fall_back_to_manifest_latest() {
    let root = tempfile::tempdir().unwrap();
    let (mut entry, _) = fixture(b"jar");
    entry.id = "99.9".into();
    let fake = Fake::new();
    fake.push(MANIFEST_URL, &manifest(&entry));
    assert!(matches!(
        crate::resolve_version(root.path(), "latest", &fake),
        Err(Error::UnknownVersion(_))
    ));
    assert!(!root.path().join("download-manifest-v1.json").exists());
}

#[test]
fn status_or_short_manifest_preserves_previous_bytes() {
    use crate::download::Response;
    for status in [200, 503] {
        let root = tempfile::tempdir().unwrap();
        let (entry, _) = fixture(b"jar");
        let old = manifest(&entry);
        let path = root.path().join("download-manifest-v1.json");
        std::fs::write(&path, &old).unwrap();
        let fake = Fake::new();
        fake.response(
            MANIFEST_URL,
            Response {
                status,
                size: Some(99999),
                body: Box::new(std::io::Cursor::new(old.clone())),
            },
        );
        assert!(resolve(root.path(), "26.3", &fake).is_err());
        assert_eq!(std::fs::read(path).unwrap(), old);
    }
}

#[test]
fn changed_same_version_manifest_selects_new_artifact_without_retargeting_old_path() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("download-manifest-v1.json");
    let (first, pkg) = fixture(b"first");
    std::fs::write(&path, manifest(&first)).unwrap();
    let fake = Fake::new();
    fake.push(&first.url, &pkg);
    fake.push("https://test.invalid/server", b"first");
    let old = crate::ensure_server_jar_in(root.path(), &first.id, &fake).unwrap();
    let snapshot = crate::resolve_version(root.path(), &first.id, &Fake::new()).unwrap();
    let (next, pkg) = fixture(b"next");
    std::fs::write(&path, manifest(&next)).unwrap();
    fake.push(&next.url, &pkg);
    fake.push("https://test.invalid/server", b"next");
    let new = crate::ensure_server_jar_in(root.path(), &next.id, &fake).unwrap();
    assert_ne!(new, old);
    let cache = crate::cache::VersionCache::acquire(root.path(), &snapshot.id).unwrap();
    assert_eq!(
        crate::download::ensure_server_jar(&cache, &snapshot, &Fake::new()).unwrap(),
        old
    );
    assert_eq!(std::fs::read(old).unwrap(), b"first");
    assert_eq!(std::fs::read(new).unwrap(), b"next");
}
