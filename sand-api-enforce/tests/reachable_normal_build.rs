use std::path::PathBuf;
use std::process::Command;

#[test]
fn ordinary_cargo_check_rejects_doc_hidden_uncontracted_method_in_enforced_scope() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/reachable-enforced-missing/Cargo.toml");
    let target = tempfile::tempdir().expect("temporary target directory");
    let output = Command::new(env!("CARGO"))
        .args(["check", "--quiet", "--manifest-path"])
        .arg(fixture)
        .env("CARGO_TARGET_DIR", target.path())
        .output()
        .expect("run cargo check for reachable enforced-scope fixture");

    assert!(!output.status.success(), "fixture unexpectedly compiled");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("enforced API scope `sand::api` has missing contracts: sand::api::Builder::uncontracted_method"),
        "unexpected cargo diagnostic:\n{stderr}"
    );
}

#[test]
fn ordinary_cargo_check_rejects_unbound_include_in_reachable_scope() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/reachable-include-unbound/Cargo.toml");
    let target = tempfile::tempdir().expect("temporary target directory");
    let output = Command::new(env!("CARGO"))
        .args(["check", "--quiet", "--manifest-path"])
        .arg(fixture)
        .env("CARGO_TARGET_DIR", target.path())
        .output()
        .expect("run cargo check for unbound reachable-include fixture");

    assert!(!output.status.success(), "fixture unexpectedly compiled");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("reachable module `sand::generated` contains include!")
            && stderr.contains(
                "neither a literal source include nor bound to a named generated API provider"
            ),
        "unexpected cargo diagnostic:\n{stderr}"
    );
}
