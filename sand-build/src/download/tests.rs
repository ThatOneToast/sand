use super::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::Cursor;

pub(crate) struct Fake {
    pub(crate) calls: RefCell<Vec<String>>,
    replies: RefCell<VecDeque<(String, Result<Response>)>>,
}
impl Fake {
    pub(crate) fn new() -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
            replies: RefCell::new(VecDeque::new()),
        }
    }
    pub(crate) fn push(&self, url: &str, bytes: &[u8]) {
        self.response(
            url,
            Response {
                status: 200,
                size: Some(bytes.len() as u64),
                body: Box::new(Cursor::new(bytes.to_vec())),
            },
        );
    }
    pub(crate) fn response(&self, url: &str, response: Response) {
        self.replies
            .borrow_mut()
            .push_back((url.to_owned(), Ok(response)));
    }
    pub(crate) fn fail(&self, url: &str) {
        self.replies
            .borrow_mut()
            .push_back((url.to_owned(), Err(invalid("injected outage"))));
    }
}
impl Transport for Fake {
    fn get(&self, url: &str) -> Result<Response> {
        self.calls.borrow_mut().push(url.to_owned());
        let (expected, reply) = self
            .replies
            .borrow_mut()
            .pop_front()
            .expect("unexpected HTTP request (warm outage regression)");
        assert_eq!(url, expected);
        reply
    }
}

pub(crate) fn fixture(jar: &[u8]) -> (VersionEntry, Vec<u8>) {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "id": "26.2", "downloads": {"server": {"sha1": hash_bytes(jar), "size": jar.len(), "url": "https://test.invalid/server"}}
    })).unwrap();
    (
        VersionEntry {
            id: "26.2".into(),
            url: "https://test.invalid/package".into(),
            sha1: hash_bytes(&bytes),
        },
        bytes,
    )
}
fn warm(root: &Path, jar: &[u8]) -> (VersionEntry, Vec<u8>, PathBuf) {
    let (entry, bytes) = fixture(jar);
    let fake = Fake::new();
    fake.push(&entry.url, &bytes);
    fake.push("https://test.invalid/server", jar);
    let cache = VersionCache::acquire(root, &entry.id).unwrap();
    let path = ensure_server_jar(&cache, &entry, &fake).unwrap();
    (entry, bytes, path)
}

#[test]
fn warm_outage_reuses_verified_package_and_rehashes_jar_without_http() {
    let root = tempfile::tempdir().unwrap();
    let (entry, _, path) = warm(root.path(), b"synthetic jar");
    let cache = VersionCache::acquire(root.path(), &entry.id).unwrap();
    let outage = Fake::new();
    assert_eq!(ensure_server_jar(&cache, &entry, &outage).unwrap(), path);
    assert!(outage.calls.borrow().is_empty());
    // A hit is not just existence/mtime: changing bytes forces a repair request.
    std::fs::write(&path, b"corrupt").unwrap();
    outage.fail("https://test.invalid/server");
    assert!(ensure_server_jar(&cache, &entry, &outage).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"corrupt");
}

#[test]
fn legacy_jar_requires_metadata_then_imports_without_mutating_legacy_paths() {
    let root = tempfile::tempdir().unwrap();
    let (entry, bytes) = fixture(b"jar");
    let cache = VersionCache::acquire(root.path(), &entry.id).unwrap();
    let legacy = cache.dir.join("server.jar");
    std::fs::write(&legacy, b"jar").unwrap();
    let old_lock = cache.dir.join(".sand-codegen.lock");
    std::fs::write(&old_lock, b"legacy owner").unwrap();
    let fake = Fake::new();
    fake.fail(&entry.url);
    assert!(ensure_server_jar(&cache, &entry, &fake).is_err());
    fake.push(&entry.url, &bytes);
    let path = ensure_server_jar(&cache, &entry, &fake).unwrap();
    assert_ne!(path, legacy);
    assert_eq!(std::fs::read(path).unwrap(), b"jar");
    assert_eq!(std::fs::read(&old_lock).unwrap(), b"legacy owner");
    assert_eq!(std::fs::read(legacy).unwrap(), b"jar");
    assert_eq!(fake.calls.borrow().len(), 2);
}

#[test]
fn raw_checksum_precedes_json_and_id_validation() {
    for bytes in [
        b"not json".as_slice(),
        b"{}",
        b"\xff",
        b"{\"id\":\"wrong\"}",
    ] {
        let root = tempfile::tempdir().unwrap();
        let (entry, _) = fixture(b"jar");
        let cache = VersionCache::acquire(root.path(), &entry.id).unwrap();
        let fake = Fake::new();
        fake.push(&entry.url, bytes);
        assert!(matches!(
            ensure_server_jar(&cache, &entry, &fake),
            Err(Error::ChecksumMismatch { .. })
        ));
        let mut matching = entry.clone();
        matching.sha1 = hash_bytes(bytes);
        fake.push(&entry.url, bytes);
        assert!(ensure_server_jar(&cache, &matching, &fake).is_err());
        assert!(!cache.dir.join("packages-v1").exists());
    }
}

#[test]
fn changed_url_digest_and_jar_isolate_identity_and_keep_returned_path_stable() {
    let root = tempfile::tempdir().unwrap();
    let (mut entry, mut bytes, old) = warm(root.path(), b"original");
    let cache = VersionCache::acquire(root.path(), &entry.id).unwrap();
    let fake = Fake::new();
    // URL only: must fetch package despite identical digest; no JAR download.
    entry.url.push_str("?mirror=2");
    fake.push(&entry.url, &bytes);
    assert_eq!(ensure_server_jar(&cache, &entry, &fake).unwrap(), old);
    // Same URL, new raw package digest but same JAR: whitespace matters.
    bytes.push(b' ');
    entry.sha1 = hash_bytes(&bytes).to_uppercase();
    fake.push(&entry.url, &bytes);
    assert_eq!(ensure_server_jar(&cache, &entry, &fake).unwrap(), old);
    // New package identity must fail closed on outage, never use prior record.
    let (mut next, next_bytes) = fixture(b"replacement");
    next.url = entry.url;
    fake.fail(&next.url);
    assert!(ensure_server_jar(&cache, &next, &fake).is_err());
    fake.push(&next.url, &next_bytes);
    fake.push("https://test.invalid/server", b"replacement");
    let new = ensure_server_jar(&cache, &next, &fake).unwrap();
    assert_ne!(new, old);
    assert_eq!(std::fs::read(old).unwrap(), b"original");
    assert_eq!(std::fs::read(new).unwrap(), b"replacement");
}

#[test]
fn package_corruption_repairs_from_selected_identity_and_jar_corruption_downloads_only_jar() {
    let root = tempfile::tempdir().unwrap();
    let (entry, bytes, jar) = warm(root.path(), b"good");
    let cache = VersionCache::acquire(root.path(), &entry.id).unwrap();
    let record = std::fs::read_dir(cache.dir.join("packages-v1"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    for bad in [b"truncated".as_slice(), b"{}"] {
        std::fs::write(&record, bad).unwrap();
        let fake = Fake::new();
        fake.fail(&entry.url);
        assert!(ensure_server_jar(&cache, &entry, &fake).is_err());
        fake.push(&entry.url, &bytes);
        assert_eq!(ensure_server_jar(&cache, &entry, &fake).unwrap(), jar);
    }
    // Even a parseable record with another identity or corrupted raw bytes is a miss.
    let saved = std::fs::read(&record).unwrap();
    for field in ["url", "bytes"] {
        let mut value: serde_json::Value = serde_json::from_slice(&saved).unwrap();
        value[field] = if field == "url" {
            serde_json::json!("https://other.invalid")
        } else {
            serde_json::json!([0, 1])
        };
        std::fs::write(&record, serde_json::to_vec(&value).unwrap()).unwrap();
        let fake = Fake::new();
        fake.push(&entry.url, &bytes);
        assert_eq!(ensure_server_jar(&cache, &entry, &fake).unwrap(), jar);
    }
    std::fs::write(&jar, b"wrong").unwrap();
    let fake = Fake::new();
    fake.push("https://test.invalid/server", b"good");
    assert_eq!(ensure_server_jar(&cache, &entry, &fake).unwrap(), jar);
    assert_eq!(
        fake.calls.borrow().as_slice(),
        ["https://test.invalid/server"]
    );
}

struct Broken;
impl Read for Broken {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("interrupted body"))
    }
}

#[test]
fn failed_status_size_checksum_and_stream_never_publish_or_destroy_previous_jar() {
    for mode in 0..5 {
        let root = tempfile::tempdir().unwrap();
        let (entry, _, jar) = warm(root.path(), b"good");
        std::fs::write(&jar, b"previous corrupt bytes").unwrap();
        let cache = VersionCache::acquire(root.path(), &entry.id).unwrap();
        let fake = Fake::new();
        let response = match mode {
            0 => Response {
                status: 503,
                size: Some(4),
                body: Box::new(Cursor::new(b"good")),
            },
            1 => Response {
                status: 200,
                size: Some(99),
                body: Box::new(Cursor::new(b"good")),
            },
            2 => Response {
                status: 200,
                size: None,
                body: Box::new(Cursor::new(b"evil")),
            },
            3 => Response {
                status: 200,
                size: None,
                body: Box::new(Broken),
            },
            _ => Response {
                status: 200,
                size: None,
                body: Box::new(Cursor::new(b"shorter")),
            },
        };
        fake.response("https://test.invalid/server", response);
        assert!(ensure_server_jar(&cache, &entry, &fake).is_err());
        assert_eq!(std::fs::read(&jar).unwrap(), b"previous corrupt bytes");
        assert_eq!(std::fs::read_dir(jar.parent().unwrap()).unwrap().count(), 1);
    }
}

#[test]
fn invalid_descriptor_and_guard_mismatch_fail_before_io() {
    let root = tempfile::tempdir().unwrap();
    for id in ["../escape", "", ".", "..", "/abs", "a/b", "a\\b"] {
        assert!(VersionCache::acquire(root.path(), id).is_err());
    }
    let cache = VersionCache::acquire(root.path(), "26.2").unwrap();
    let (mut entry, _) = fixture(b"jar");
    entry.sha1 = "not a digest".into();
    assert!(ensure_server_jar(&cache, &entry, &Fake::new()).is_err());
    entry.id = "26.3".into();
    assert!(ensure_server_jar(&cache, &entry, &Fake::new()).is_err());
}

#[test]
fn public_pipeline_warm_outage_makes_zero_http_requests_for_pinned_and_latest() {
    let root = tempfile::tempdir().unwrap();
    let (mut entry, pkg) = fixture(b"jar");
    entry.id = sand_version::LATEST_KNOWN.into();
    let mut package: serde_json::Value = serde_json::from_slice(&pkg).unwrap();
    package["id"] = serde_json::json!(entry.id);
    let pkg = serde_json::to_vec(&package).unwrap();
    entry.sha1 = hash_bytes(&pkg);
    let bytes = serde_json::to_vec(&serde_json::json!({"versions": [entry]})).unwrap();
    std::fs::write(root.path().join("download-manifest-v1.json"), bytes).unwrap();
    let initial = Fake::new();
    initial.push(&entry.url, &pkg);
    initial.push("https://test.invalid/server", b"jar");
    let jar = crate::ensure_server_jar_in(root.path(), &entry.id, &initial).unwrap();
    let outage = Fake::new();
    for version in [entry.id.as_str(), "latest"] {
        assert_eq!(
            crate::ensure_server_jar_in(root.path(), version, &outage).unwrap(),
            jar
        );
    }
    assert!(outage.calls.borrow().is_empty());
}

#[test]
fn matching_raw_checksum_still_requires_correct_package_id_and_server_descriptor() {
    for field in ["id", "server-sha1", "server-url", "missing-server"] {
        let root = tempfile::tempdir().unwrap();
        let cache = VersionCache::acquire(root.path(), "26.2").unwrap();
        let (mut entry, bytes) = fixture(b"jar");
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        match field {
            "id" => value["id"] = serde_json::json!("26.3"),
            "server-sha1" => value["downloads"]["server"]["sha1"] = serde_json::json!("invalid"),
            "server-url" => {
                value["downloads"]["server"]["url"] = serde_json::json!("file:///tmp/jar")
            }
            _ => value["downloads"] = serde_json::json!({}),
        }
        let bytes = serde_json::to_vec(&value).unwrap();
        entry.sha1 = hash_bytes(&bytes);
        let fake = Fake::new();
        fake.push(&entry.url, &bytes);
        assert!(ensure_server_jar(&cache, &entry, &fake).is_err());
        assert_eq!(fake.calls.borrow().len(), 1);
        assert!(!cache.dir.join("packages-v1").exists());
    }
}

#[test]
fn streaming_verification_checks_multiple_chunks_and_recovers_missing_jar() {
    let bytes = (0..=255u8)
        .cycle()
        .take(1024 * 1024 + 13)
        .collect::<Vec<_>>();
    let root = tempfile::tempdir().unwrap();
    let (entry, _, jar) = warm(root.path(), &bytes);
    let cache = VersionCache::acquire(root.path(), &entry.id).unwrap();
    assert_eq!(
        ensure_server_jar(&cache, &entry, &Fake::new()).unwrap(),
        jar
    );
    std::fs::remove_file(&jar).unwrap();
    let fake = Fake::new();
    fake.push("https://test.invalid/server", &bytes);
    assert_eq!(ensure_server_jar(&cache, &entry, &fake).unwrap(), jar);
    assert_eq!(std::fs::read(jar).unwrap(), bytes);
}
