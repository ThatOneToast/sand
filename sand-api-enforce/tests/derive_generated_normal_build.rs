use std::path::PathBuf;
use std::process::Command;

#[test]
fn real_derive_expansion_is_provider_connected_and_missing_member_contract_fails_check() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/derive-generated-missing/Cargo.toml");
    let target = tempfile::tempdir().expect("temporary target directory");

    let complete = Command::new(env!("CARGO"))
        .args(["check", "--quiet", "--manifest-path"])
        .arg(&fixture)
        .args(["--features", "complete-provider"])
        .env("CARGO_TARGET_DIR", target.path())
        .output()
        .expect("run complete provider fixture");
    assert!(
        complete.status.success(),
        "real derive expansion or complete provider failed:\n{}",
        String::from_utf8_lossy(&complete.stderr)
    );

    let missing = Command::new(env!("CARGO"))
        .args(["check", "--quiet", "--manifest-path"])
        .arg(&fixture)
        .env("CARGO_TARGET_DIR", target.path())
        .output()
        .expect("run missing generated contract fixture");
    assert!(!missing.status.success(), "fixture unexpectedly compiled");
    let stderr = String::from_utf8_lossy(&missing.stderr);
    assert!(
        stderr.contains("enforced API scope `sand` has missing contracts: sand::PlayerMagic::mana"),
        "unexpected cargo diagnostic:\n{stderr}"
    );
}
