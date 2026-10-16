//! JSON reports analyze the same runtime files that publication preserves.
use std::process::Command;

/// Preserved functions and lifecycle tags remain visible; overlays stay explicitly unknown.
#[test]
fn portable_report_includes_preserved_runtime_resources() {
    let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
    let input = temp.path().join("fixture.sand.json");
    std::fs::write(
        &input,
        serde_json::json!({
            "format": "sand.program", "format_version": 1,
            "target": {"minecraft": "26.2"},
            "pack": {"namespace": "demo", "description": "Publication report"},
            "modules": [{"id": "demo:module", "functions": [
                {"id": "demo:owned", "context": "server", "body": []}
            ]}]
        })
        .to_string(),
    )
    .unwrap();
    let output = temp.path().join("pack");
    for (path, content) in [
        ("data/external/function/load.mcfunction", "say preserved"),
        (
            "data/minecraft/tags/function/load.json",
            r#"{"values":["external:load"]}"#,
        ),
        (
            "overlay/data/external/function/extra.mcfunction",
            "say overlay",
        ),
    ] {
        let file = output.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    let result = Command::new(env!("CARGO_BIN_EXE_sand"))
        .args(["program", "compile", "--input"])
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let report = &response["runtime_report"];
    assert_eq!(
        report["functions"]["external:load"]["evaluations"]["maximum"],
        1
    );
    assert!(
        report["load"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["reference"] == "external:load")
    );
    assert!(
        report["limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry
                .as_str()
                .unwrap()
                .contains("overlay/data/external/function/extra.mcfunction"))
    );
}
