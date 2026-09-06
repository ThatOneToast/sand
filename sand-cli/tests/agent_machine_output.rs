use std::process::Command;

#[test]
fn context_json_is_schema_versioned_and_byte_stable_outside_a_project() {
    let temp = tempfile::tempdir().unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_sand"))
            .args(["context", "--format", "json"])
            .current_dir(temp.path())
            .output()
            .unwrap()
    };
    let first = run();
    let second = run();
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let value: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["is_sand_project"], false);
    assert_eq!(value["compatibility"]["status"], "not_a_project");
}

#[test]
fn api_query_json_has_scores_counts_contract_fields_and_context() {
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sand"))
        .args([
            "api",
            "search",
            "damage",
            "player",
            "--all-terms",
            "--field",
            "summary",
            "--limit",
            "2",
            "--format",
            "json",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert!(value["total_matches"].as_u64().is_some());
    assert!(value["truncated"].is_boolean());
    assert_eq!(value["project_context"]["is_sand_project"], false);
    for (index, result) in value["results"].as_array().unwrap().iter().enumerate() {
        assert_eq!(result["match_reason"]["rank"], index + 1);
        assert!(result["match_reason"]["score"].as_u64().is_some());
        assert_eq!(result["match_reason"]["field"], "summary");
        assert!(
            result["canonical_path"]
                .as_str()
                .unwrap()
                .starts_with("sand::")
        );
        assert!(result["kind"].is_string());
        assert!(result["summary"].is_string());
        assert!(result["availability"].is_array());
    }
}

#[test]
fn show_module_and_alternatives_have_stable_json_shapes() {
    let temp = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_sand"))
            .args(args)
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()
    };
    let show = run(&[
        "api",
        "show",
        "sand::command::Target::nearby",
        "--format",
        "json",
    ]);
    assert_eq!(show["schema_version"], 1);
    assert_eq!(
        show["api"]["canonical_path"],
        "sand::command::Target::nearby"
    );
    assert!(show["api"]["signature"].is_string());
    assert!(show["related_apis"].is_array());

    let module = run(&["api", "module", "sand::entity", "--format", "json"]);
    assert_eq!(module["schema_version"], 1);
    assert_eq!(module["module"], "sand::entity");
    assert!(module["results"].is_array());
    assert!(module["nested_modules"].is_array());

    let alternatives = run(&[
        "api",
        "alternatives",
        "damage entity",
        "--limit",
        "2",
        "--format",
        "json",
    ]);
    assert_eq!(alternatives["schema_version"], 1);
    assert_eq!(alternatives["compatible_with_project"], true);
    assert!(alternatives["typed_alternatives"].is_array());
    assert!(alternatives["advanced_escape_hatches"].is_array());
}

#[test]
fn build_and_check_failures_are_machine_readable_and_nonzero() {
    let temp = tempfile::tempdir().unwrap();
    let build = Command::new(env!("CARGO_BIN_EXE_sand"))
        .args(["build", "--format", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!build.status.success());
    let build_json: serde_json::Value = serde_json::from_slice(&build.stdout).unwrap();
    assert_eq!(build_json["schema_version"], 1);
    assert_eq!(build_json["success"], false);
    assert_eq!(build_json["diagnostics"][0]["code"], "SAND_BUILD_CONFIG");

    let check = Command::new(env!("CARGO_BIN_EXE_sand"))
        .args(["check", "--agent", "--format", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!check.status.success());
    let check_json: serde_json::Value = serde_json::from_slice(&check.stdout).unwrap();
    assert_eq!(check_json["schema_version"], 1);
    assert_eq!(check_json["success"], false);
    assert_eq!(
        check_json["diagnostics"][0]["code"],
        "SAND_CHECK_NOT_PROJECT"
    );
}
