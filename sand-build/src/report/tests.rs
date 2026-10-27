use super::*;
use crate::download::tests::{Fake, fixture};

fn jar(cache: &VersionCache, bytes: &[u8]) -> PathBuf {
    let (entry, pkg) = fixture(bytes);
    let fake = Fake::new();
    fake.push(&entry.url, &pkg);
    fake.push("https://test.invalid/server", bytes);
    crate::download::ensure_server_jar(cache, &entry, &fake).unwrap()
}
fn generate(dir: &Path, _: &Path) -> Result<()> {
    let reports = dir.join("generated/reports");
    std::fs::create_dir_all(&reports)?;
    for file in REQUIRED {
        std::fs::write(reports.join(file), b"{\"fixture\":true}")?;
    }
    Ok(())
}

#[test]
fn reports_follow_jar_provenance_and_never_reuse_legacy_sentinel() {
    let root = tempfile::tempdir().unwrap();
    let cache = VersionCache::acquire(root.path(), "26.2").unwrap();
    let legacy = cache.dir.join("generated/reports");
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("registries.json"), b"legacy").unwrap();
    let first = jar(&cache, b"first");
    let reports = ensure_reports_with(&cache, &first, generate).unwrap();
    assert!(reports.starts_with(first.parent().unwrap()));
    assert_eq!(
        ensure_reports_with(&cache, &first, |_, _| panic!("warm generator")).unwrap(),
        reports
    );
    let second = jar(&cache, b"second");
    let other = ensure_reports_with(&cache, &second, generate).unwrap();
    assert_ne!(reports, other);
    // A complete-looking copy with the wrong provenance is a miss.
    std::fs::copy(
        reports.join(".sand-provenance.json"),
        other.join(".sand-provenance.json"),
    )
    .unwrap();
    assert!(ensure_reports_with(&cache, &second, |_, _| Err(invalid("generator outage"))).is_err());
    ensure_reports_with(&cache, &second, generate).unwrap();
    assert_eq!(
        std::fs::read(legacy.join("registries.json")).unwrap(),
        b"legacy"
    );
}

#[test]
fn incomplete_failed_or_corrupt_reports_never_become_hits() {
    let root = tempfile::tempdir().unwrap();
    let cache = VersionCache::acquire(root.path(), "26.2").unwrap();
    let jar = jar(&cache, b"jar");
    assert!(
        ensure_reports_with(&cache, &jar, |stage, _| {
            let dir = stage.join("generated/reports");
            std::fs::create_dir_all(&dir)?;
            std::fs::write(dir.join("registries.json"), b"{}")?;
            Ok(())
        })
        .is_err()
    );
    assert!(!jar.parent().unwrap().join("reports-v1").exists());
    assert!(
        ensure_reports_with(&cache, &jar, |stage, jar| {
            generate(stage, jar)?;
            Err(invalid("killed generator"))
        })
        .is_err()
    );
    let reports = ensure_reports_with(&cache, &jar, generate).unwrap();
    std::fs::write(reports.join("blocks.json"), b"{\"changed\":true}").unwrap();
    let previous = std::fs::read(reports.join(".sand-provenance.json")).unwrap();
    assert!(ensure_reports_with(&cache, &jar, |_, _| Err(invalid("failed writer"))).is_err());
    assert_eq!(
        std::fs::read(reports.join(".sand-provenance.json")).unwrap(),
        previous
    );
    ensure_reports_with(&cache, &jar, generate).unwrap();
    assert_eq!(std::fs::read_dir(jar.parent().unwrap()).unwrap().count(), 2);
}
