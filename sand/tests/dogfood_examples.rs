use std::path::Path;
use std::process::Command;

#[test]
fn dogfood_examples_compile_every_target() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sand is in the repository workspace");
    let target = tempfile::tempdir().expect("create isolated dogfood target");

    for example in ["arcane_pack", "book_project"] {
        let manifest = workspace.join("examples").join(example).join("Cargo.toml");
        let output = Command::new(env!("CARGO"))
            .current_dir(workspace)
            .env("CARGO_TARGET_DIR", target.path())
            .args(["check", "--offline", "--all-targets", "--manifest-path"])
            .arg(&manifest)
            .output()
            .unwrap_or_else(|error| panic!("check {}: {error}", manifest.display()));

        assert!(
            output.status.success(),
            "{example} all-target check failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
