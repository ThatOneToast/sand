use std::path::Path;
use std::process::Command;

#[test]
fn unreported_generated_registry_api_fails_normal_cargo_check() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/generated-registry-uncontracted/Cargo.toml");
    let target = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO"))
        .arg("check")
        .arg("--manifest-path")
        .arg(fixture)
        .env("CARGO_TARGET_DIR", target.path())
        .output()
        .expect("run generated-registry missing-contract fixture");

    assert!(!output.status.success(), "fixture unexpectedly compiled");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("public but unreported")
            && stderr.contains("fixture_core::generated::Item::Uncontracted (Variant)")
            && stderr.contains("fixture_core::generated::Item::uncontracted_method (Method)")
            && stderr.contains("fixture_core::generated::UncontractedRegistry (Enum)"),
        "unexpected cargo check diagnostic:\n{stderr}"
    );
}
