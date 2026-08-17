use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Run an API-contract consumer fixture in a target directory owned solely by
/// that fixture. It intentionally does not use Cargo's active workspace
/// target: an integration test is itself running under Cargo, and sharing the
/// outer target can deadlock on Cargo's build lock. Keeping the cache beneath
/// the ignored workspace `target/` directory also avoids the enormous
/// temporary-directory teardown which previously kept successful tests alive
/// long after their child Cargo processes had exited.
pub fn check_fixture(fixture: &str, features: Option<&str>) -> Output {
    let manifest = fixture_manifest(fixture);
    let target = fixture_target(fixture);
    let mut command = Command::new(env!("CARGO"));
    command
        .args(["check", "--quiet", "--manifest-path"])
        .arg(manifest)
        .env("CARGO_TARGET_DIR", target)
        .env("CARGO_TERM_COLOR", "never")
        .env("CARGO_INCREMENTAL", "0");
    if let Some(features) = features {
        command.args(["--features", features]);
    }
    command.output().expect("run API-contract consumer fixture")
}

pub fn assert_fixture_passes(fixture: &str, features: Option<&str>) {
    let output = check_fixture(fixture, features);
    assert!(
        output.status.success(),
        "consumer fixture `{fixture}` unexpectedly failed:\n{}",
        rendered_output(&output)
    );
}

pub fn assert_fixture_fails_with(fixture: &str, expected_diagnostic: &str) {
    let output = check_fixture(fixture, None);
    assert!(
        !output.status.success(),
        "consumer fixture `{fixture}` unexpectedly compiled"
    );
    let rendered = rendered_output(&output);
    assert!(
        rendered.contains(expected_diagnostic),
        "consumer fixture `{fixture}` had an unexpected diagnostic; expected `{expected_diagnostic}`:\n{rendered}"
    );
}

fn fixture_manifest(fixture: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture)
        .join("Cargo.toml")
}

fn fixture_target(fixture: &str) -> PathBuf {
    workspace_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .join("target/api-contract-consumer-fixtures")
        .join(fixture)
}

fn workspace_root(manifest_dir: &Path) -> &Path {
    manifest_dir
        .parent()
        .expect("sand-api-enforce is directly below the workspace root")
}

fn rendered_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    match (stdout.trim(), stderr.trim()) {
        ("", stderr) => stderr.to_owned(),
        (stdout, "") => stdout.to_owned(),
        (stdout, stderr) => format!("{stdout}\n{stderr}"),
    }
}
