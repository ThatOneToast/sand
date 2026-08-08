use std::path::PathBuf;
use std::process::Command;

#[test]
fn normal_cargo_check_rejects_an_unannotated_public_api() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/missing-contract/Cargo.toml");
    let target = tempfile::tempdir().expect("temporary target directory");
    let output = Command::new(env!("CARGO"))
        .args(["check", "--quiet", "--manifest-path"])
        .arg(fixture)
        .env("CARGO_TARGET_DIR", target.path())
        .output()
        .expect("run cargo check for missing-contract fixture");

    assert!(!output.status.success(), "fixture unexpectedly compiled");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("public API `sand::fixture::forgotten_api` (function) is missing #[api]"),
        "unexpected cargo diagnostic:\n{stderr}"
    );
}
