//! Two independent exports of identical built-in event subscriptions.
use sand_core::{ArmorEventDescriptor, ArmorEventKind, ArmorSlot, EventDescriptor, EventDispatch};
use serde_json::Value;
use std::collections::BTreeSet;

fn counter(objective: &str) -> Vec<String> {
    ["SandAlpha", "SandBeta"].map(|player| format!(
        "execute if entity @s[name={player}] run scoreboard players add #__sand_local:{player} {objective} 1"
    )).into()
}
macro_rules! event {
    ($path:literal, $dispatch:ident, $objective:literal) => {
        sand_core::inventory::submit! {
            EventDescriptor { path: $path, id_override: None,
                make: || sand::component::IntoCommands::into_commands(counter($objective)),
                dispatch: EventDispatch::$dispatch }
        }
    };
}
event!("death", DeathTick, "audit_death");
event!("respawn", RespawnTick, "audit_respawn");
event!("level", XpLevelUp, "audit_level");
macro_rules! armor {
    ($path:literal, $kind:ident, $objective:literal) => {
        sand_core::inventory::submit! {
            ArmorEventDescriptor { path: $path,
                make: || sand::component::IntoCommands::into_commands(counter($objective)),
                slot: ArmorSlot::Head, kind: ArmorEventKind::$kind,
                item_id: Some("minecraft:leather_helmet"), custom_data_snbt: None }
        }
    };
}
armor!("equip", Equip, "audit_equip");
armor!("unequip", Unequip, "audit_unequip");

fn records(namespace: &str) -> Vec<Value> {
    serde_json::from_str(&sand_core::try_export_components_json(namespace).unwrap()).unwrap()
}
fn objectives(records: &[Value]) -> BTreeSet<String> {
    records
        .iter()
        .filter(|r| r["dir"] == "function")
        .flat_map(|r| r["content"].as_str().unwrap().lines())
        .filter_map(|line| line.strip_prefix("scoreboard objectives add "))
        .map(|line| line.split_whitespace().next().unwrap().to_owned())
        .collect()
}

#[test]
fn detector_scores_are_export_owned_and_do_not_touch_legacy_globals() {
    let a = records("detecta");
    let b = records("detectb");
    let a_names = objectives(&a);
    let b_names = objectives(&b);
    assert_eq!(a_names.len(), 7);
    assert_eq!(b_names.len(), 7);
    assert!(a_names.is_disjoint(&b_names));
    assert!(a_names.iter().chain(&b_names).all(|name| name.len() <= 16));
    for record in a.iter().chain(&b) {
        let body = record["content"].as_str().unwrap();
        for legacy in [
            "__sand_dc",
            "__sand_rp",
            "__sand_tsd",
            "__sand_xp_lvl",
            "__sand_xp_prev",
            "__sand_xp_delta",
            "__sand_xp_seen",
            "__sand_just_died",
        ] {
            assert!(
                !body.split_whitespace().any(|word| word == legacy),
                "legacy state {legacy}: {body}"
            );
        }
    }
    let workers = ["detecta", "detectb", "detecta", "detectb"]
        .map(|namespace| std::thread::spawn(move || (namespace, records(namespace))));
    for worker in workers {
        let (namespace, actual) = worker.join().unwrap();
        assert_eq!(
            actual,
            if namespace == "detecta" {
                a.clone()
            } else {
                b.clone()
            }
        );
    }
}

#[test]
#[ignore = "external live-server fixture export"]
fn dump_detector_packs() {
    let output = std::env::var("SAND_DETECTOR_DUMP_PATH").expect("dump path");
    std::fs::write(
        output,
        serde_json::to_vec(&serde_json::json!({
            "detecta": records("detecta"), "detectb": records("detectb")
        }))
        .unwrap(),
    )
    .unwrap();
}
