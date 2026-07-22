//! Export coverage for automatic participant-plan integration on
//! advancement-backed combat events (#230): proves the export pipeline
//! splices `AdvancementEvent::participants()`'s commands around the
//! generated body without any manual wiring in the handler's own
//! `#[event]` function, for both the correlated-attacker backend
//! (`EntityDamagePlayerEvent`) and the held-item snapshot backend
//! (`PlayerDamageEntityEvent`).

use sand_example::participant_context_example::{on_hurt_by_entity, on_hurt_entity};

fn records(json: &str) -> Vec<serde_json::Value> {
    serde_json::from_str(json).unwrap()
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing function {path}"))
}

#[test]
fn combat_participant_plans_export_deterministically_and_wrap_the_body() {
    assert!(!on_hurt_by_entity().is_empty());
    assert!(!on_hurt_entity().is_empty());

    let first = sand_core::try_export_components_json("participantpack").unwrap();
    let second = sand_core::try_export_components_json("participantpack").unwrap();
    assert_eq!(
        first, second,
        "repeated participant-plan export must be byte-identical"
    );
    let records = records(&first);

    // EntityDamagePlayerEvent: automatic correlated-attacker plan.
    let attacker_body = function(&records, "on_hurt_by_entity/body");
    let reset_pos = attacker_body
        .find("present set value 0b")
        .expect("attacker plan setup (reset) is spliced into the body");
    let mark_pos = attacker_body
        .find("execute on attacker run")
        .expect("attacker plan mark/bind is spliced into the body");
    let handler_pos = attacker_body
        .find("# attacker =")
        .expect("the handler's own command is present");
    let cleanup_pos = attacker_body
        .find("tag @e[tag=__sand_observed_")
        .expect("attacker plan cleanup is spliced into the body");
    assert!(reset_pos < mark_pos, "reset must run before mark/bind");
    assert!(
        mark_pos < handler_pos,
        "setup must run before the handler's own commands"
    );
    assert!(
        handler_pos < cleanup_pos,
        "cleanup must run after the handler's own commands"
    );

    // PlayerDamageEntityEvent: automatic held-item (weapon) snapshot plan.
    let weapon_body = function(&records, "on_hurt_entity/body");
    let capture_pos = weapon_body
        .find("SelectedItem")
        .expect("weapon snapshot capture (mainhand) is spliced into the body");
    let weapon_handler_pos = weapon_body
        .find("# weapon storage")
        .expect("the handler's own command is present");
    assert!(
        capture_pos < weapon_handler_pos,
        "item snapshot capture must run before the handler's own commands"
    );
}
