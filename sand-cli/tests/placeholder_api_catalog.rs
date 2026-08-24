use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn placeholder_installation_supports_api_search_show_and_export() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sand-cli is in the workspace");
    let target = tempfile::tempdir().expect("create isolated placeholder target");
    let status = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", target.path())
        .env("SAND_MC_VERSION", "definitely-not-a-release")
        .env("SAND_ALLOW_PLACEHOLDER_CODEGEN", "1")
        .args([
            "build",
            "--offline",
            "-p",
            "sand-cli",
            "--no-default-features",
        ])
        .status()
        .expect("build placeholder CLI");
    assert!(status.success(), "placeholder CLI must compile");

    let binary = target
        .path()
        .join("debug")
        .join(format!("sand{}", std::env::consts::EXE_SUFFIX));
    let search = run(&binary, &["api", "search", "player", "--limit", "5"]);
    assert!(search.contains("showing 5 of"));
    assert!(search.contains("sand::event::DamageEvent::player"));

    let show = run(&binary, &["api", "show", "sand::command::IntoGiveItem"]);
    assert!(show.contains("sand::registry::ItemId"));
    assert!(!show.contains("sand_core::generated"));

    let export = target.path().join("placeholder-api.json");
    run(
        &binary,
        &[
            "api",
            "export",
            "--output",
            export.to_str().expect("UTF-8 temp path"),
        ],
    );
    let json = fs::read_to_string(export).expect("read placeholder export");
    let catalog: serde_json::Value = serde_json::from_str(&json).expect("parse placeholder export");
    assert_eq!(
        catalog["configuration"]["surface_profile"],
        "placeholder-codegen"
    );
    assert_eq!(
        catalog["configuration"]["minecraft_version"],
        "definitely-not-a-release"
    );
    assert_eq!(catalog["configuration"]["placeholder_codegen"], true);
    assert_eq!(
        catalog["configuration"]["compiled_surface_items"].as_u64(),
        catalog["entries"]
            .as_array()
            .map(|entries| entries.len() as u64),
        "placeholder metadata must report the exact installed entry count"
    );
    assert!(!json.contains("sand_core::generated"));
}

fn run(binary: &Path, arguments: &[&str]) -> String {
    let output = Command::new(binary)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("run {}: {error}", binary.display()));
    assert!(
        output.status.success(),
        "{} {arguments:?} failed:\n{}",
        binary.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("CLI output is UTF-8")
}
