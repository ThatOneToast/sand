use std::path::Path;
use std::process::Command;

#[test]
fn player_data_feature_exposes_its_lifecycle_dependency_and_compatibility_alias() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/player-data-feature-closure/Cargo.toml");
    let target = tempfile::tempdir().expect("create isolated feature-closure target");
    let output = Command::new(env!("CARGO"))
        .env("CARGO_TARGET_DIR", target.path())
        .args(["check", "--offline", "--manifest-path"])
        .arg(&manifest)
        .output()
        .expect("check player-data feature closure fixture");

    assert!(
        output.status.success(),
        "systems-player-data feature closure failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
