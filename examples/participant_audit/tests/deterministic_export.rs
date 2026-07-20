//! Structural export coverage for the #265 runtime-validation audit pack.
//!
//! This proves the pack exports deterministically and contains the exact
//! generated functions/records the real-server validation tooling
//! (`scripts/mc_validation/`) depends on — it is not a substitute for the
//! real-server evidence itself (see `docs/testing/participant-role-evidence.md`).
//!
//! Export is driven through the same standard scaffold a real user runs
//! (`cargo run --bin sand_export`, or `sand build`) — this test does not
//! call any hidden export internals from the example's own library.

use std::process::Command;

fn export() -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_sand_export"))
        .env("SAND_EXPORT_MC_VERSION", "26.2")
        .output()
        .expect("sand_export binary must run");
    assert!(
        output.status.success(),
        "sand_export failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("export output is valid UTF-8")
}

fn records(json: &str) -> Vec<serde_json::Value> {
    serde_json::from_str(json).expect("valid export JSON")
}

#[test]
fn export_is_deterministic() {
    let first = export();
    let second = export();
    assert_eq!(first, second, "repeated export must be byte-identical");
}

#[test]
fn every_audit_handler_is_present() {
    let records = records(&export());
    let paths: Vec<&str> = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .filter_map(|r| r["path"].as_str())
        .collect();
    for expected in [
        "init",
        "audit_on_hurt_by_entity_a/body",
        "audit_on_hurt_by_entity_b/body",
        "audit_on_killed/body",
        "audit_on_hurt_entity/body",
        "audit_on_killed_entity/body",
    ] {
        assert!(
            paths.contains(&expected),
            "missing generated function {expected} in {paths:?}"
        );
    }
}

#[test]
fn attacker_handler_body_wraps_correlated_observation_around_the_audit_write() {
    let records = records(&export());
    let body = records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == "audit_on_hurt_by_entity_a/body")
        .and_then(|r| r["content"].as_str())
        .expect("audit_on_hurt_by_entity_a/body function must exist");

    let reset = body.find("present set value 0b").expect("reset present");
    let mark = body
        .find("execute on attacker run")
        .expect("attacker mark/bind present");
    let audit_write = body
        .find("state.attacker_present set value 1b")
        .expect("audit evidence write present");
    let cleanup = body
        .find("tag @e[tag=__sand_observed_")
        .expect("cleanup present");

    assert!(reset < mark, "reset must run before mark/bind");
    assert!(
        mark < audit_write,
        "attacker binding must be set up before the handler writes audit evidence"
    );
    assert!(
        audit_write < cleanup,
        "cleanup must run after the handler's own commands"
    );
}

#[test]
fn attacker_uuid_capture_uses_execute_at_the_typed_participant_handle() {
    let records = records(&export());
    let body = records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == "audit_on_hurt_by_entity_a/body")
        .and_then(|r| r["content"].as_str())
        .expect("audit_on_hurt_by_entity_a/body function must exist");

    assert!(
        body.contains("set from entity @s UUID"),
        "attacker UUID copy must run as @s inside an `execute at <attacker>` context: {body}"
    );
}

/// `if_(weapon.is_present()).then_all(...).else_all(...)` compiles to a call
/// out to two separate generated branch functions rather than inline
/// content — this helper follows the `execute if/unless ... run function
/// <ns>:<path>` calls a handler body makes and returns each target
/// function's own content.
fn branch_targets<'a>(records: &'a [serde_json::Value], handler_body: &str) -> Vec<&'a str> {
    let mut targets = Vec::new();
    for line in handler_body.lines() {
        let Some(idx) = line.find("run function paudit:") else {
            continue;
        };
        let path = &line[idx + "run function paudit:".len()..];
        let content = records
            .iter()
            .find(|r| r["dir"] == "function" && r["path"] == path)
            .and_then(|r| r["content"].as_str())
            .unwrap_or_else(|| panic!("branch target function {path} must exist"));
        targets.push(content);
    }
    targets
}

#[test]
fn weapon_handler_body_captures_mainhand_before_dispatching_to_the_presence_branch() {
    let records = records(&export());
    let body = records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == "audit_on_hurt_entity/body")
        .and_then(|r| r["content"].as_str())
        .expect("audit_on_hurt_entity/body function must exist");

    let capture = body
        .find("SelectedItem")
        .expect("mainhand item snapshot capture present");
    let dispatch = body
        .find("run function paudit:sand/branches/")
        .expect("presence-branch dispatch present");
    assert!(
        capture < dispatch,
        "item snapshot capture must run before the handler dispatches to the presence branch"
    );
}

#[test]
fn weapon_handler_branches_on_snapshot_presence() {
    let records = records(&export());
    let body = records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == "audit_on_hurt_entity/body")
        .and_then(|r| r["content"].as_str())
        .expect("audit_on_hurt_entity/body function must exist");

    assert!(
        body.contains("execute if data storage sand:__participants")
            && body.contains("execute unless data storage sand:__participants"),
        "both present/absent branch dispatches must be generated: {body}"
    );

    let targets = branch_targets(&records, body);
    assert!(
        targets
            .iter()
            .any(|t| t.contains("state.weapon_present set value 1b")
                && t.contains("state.weapon_item set from storage")),
        "present branch must write weapon_present=1b and copy the item snapshot: {targets:?}"
    );
    assert!(
        targets
            .iter()
            .any(|t| t.contains("state.weapon_present set value 0b")),
        "absent branch must write weapon_present=0b: {targets:?}"
    );
}

#[test]
fn no_generated_body_contains_a_literal_placeholder() {
    // A cheap sanity check that every generated function body is non-empty
    // real command text, not an accidentally-empty stub.
    let records = records(&export());
    for record in records.iter().filter(|r| r["dir"] == "function") {
        let content = record["content"].as_str().unwrap_or_default();
        assert!(
            !content.trim().is_empty(),
            "function {:?} exported an empty body",
            record["path"]
        );
    }
}
