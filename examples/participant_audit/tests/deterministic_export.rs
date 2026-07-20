//! Structural export coverage for the #265 runtime-validation audit pack.
//!
//! This proves the pack exports deterministically and contains the exact
//! generated functions/records the real-server validation tooling
//! (`scripts/mc_validation/`) depends on — it is not a substitute for the
//! real-server evidence itself (see `docs/testing/participant-role-evidence.md`).

fn export() -> String {
    participant_audit::__sand_export_json("paudit", "26.2").expect("export must succeed")
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
        .find("attacker.present set value 1b")
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
fn weapon_handler_body_captures_mainhand_before_the_audit_write() {
    let records = records(&export());
    let body = records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == "audit_on_hurt_entity/body")
        .and_then(|r| r["content"].as_str())
        .expect("audit_on_hurt_entity/body function must exist");

    let capture = body
        .find("SelectedItem")
        .expect("mainhand item snapshot capture present");
    let audit_write = body
        .find("weapon.present set value 1b")
        .expect("audit evidence write present");
    assert!(
        capture < audit_write,
        "item snapshot capture must run before the handler writes audit evidence"
    );
}
