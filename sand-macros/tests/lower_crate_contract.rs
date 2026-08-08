use std::process::Command;

#[test]
fn lower_crate_contract_reaches_facade_catalog_and_build_enforcement() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/lower-api-contract/Cargo.toml");
    let target =
        std::env::temp_dir().join(format!("sand-lower-api-contract-{}", std::process::id()));
    let output = Command::new(env!("CARGO"))
        .args(["test", "--quiet", "--manifest-path"])
        .arg(fixture)
        .env("CARGO_TARGET_DIR", &target)
        .output()
        .expect("run lower-crate API contract fixture");
    let _ = std::fs::remove_dir_all(target);
    assert!(
        output.status.success(),
        "lower-crate fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
