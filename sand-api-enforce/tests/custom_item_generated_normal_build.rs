use std::path::PathBuf;
use std::process::Command;

#[test]
fn custom_item_generated_public_apis_require_exact_consumer_contracts() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/custom-item-generated-missing/Cargo.toml");
    let target = tempfile::tempdir().expect("temporary target directory");
    let complete = Command::new(env!("CARGO"))
        .args(["check", "--quiet", "--manifest-path"])
        .arg(&fixture)
        .args(["--features", "complete-provider"])
        .env("CARGO_TARGET_DIR", target.path())
        .output()
        .expect("run complete custom-item provider fixture");
    assert!(
        complete.status.success(),
        "custom-item expansion or complete provider failed:\n{}",
        String::from_utf8_lossy(&complete.stderr)
    );
    let missing = Command::new(env!("CARGO"))
        .args(["check", "--quiet", "--manifest-path"])
        .arg(&fixture)
        .env("CARGO_TARGET_DIR", target.path())
        .output()
        .expect("run missing custom-item generated contract fixture");
    assert!(!missing.status.success(), "fixture unexpectedly compiled");
    assert!(
        String::from_utf8_lossy(&missing.stderr)
            .contains("enforced API scope `sand` has missing contracts: sand::ShardBlade::DAMAGE"),
        "unexpected cargo diagnostic:\n{}",
        String::from_utf8_lossy(&missing.stderr)
    );
}
