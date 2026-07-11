use std::process::Command;

fn export_binary() -> &'static str {
    env!("CARGO_BIN_EXE_sand_export")
}

#[test]
fn export_binary_requires_and_uses_runtime_version_transport() {
    let missing = Command::new(export_binary())
        .env_remove("SAND_EXPORT_MC_VERSION")
        .output()
        .expect("run sand_export without version transport");
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("SAND_EXPORT_MC_VERSION"));

    let malformed = Command::new(export_binary())
        .env("SAND_EXPORT_MC_VERSION", "not-a-version")
        .output()
        .expect("run sand_export with malformed version transport");
    assert!(!malformed.status.success());
    assert!(String::from_utf8_lossy(&malformed.stderr).contains("not-a-version"));

    let valid = Command::new(export_binary())
        .env("SAND_EXPORT_MC_VERSION", "1.21.4")
        .output()
        .expect("run sand_export with the configured version");
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    assert!(
        String::from_utf8_lossy(&valid.stdout)
            .trim_start()
            .starts_with('[')
    );
}
