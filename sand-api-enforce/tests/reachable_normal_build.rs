use std::path::PathBuf;
use std::process::Command;

#[test]
fn ordinary_cargo_check_rejects_new_members_in_enforced_source_scopes() {
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
        stderr.contains("sand::predicate::Builder::uncontracted_field")
            && stderr.contains("sand::predicate::Builder::uncontracted_method")
            && stderr.contains("sand::predicate::Choice::UncontractedVariant")
            && stderr.contains("sand::execute_when::WhenBuilder::uncontracted_branch")
            && stderr.contains("sand::condition::Condition::uncontracted_leaf")
            && stderr.contains("sand::resource_ref::DialogId::uncontracted_local"),
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
