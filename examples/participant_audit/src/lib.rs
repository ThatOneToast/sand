//! Dedicated runtime-validation datapack for #230's participant backends
//! (#265) — every scenario writes deterministic, machine-readable evidence
//! to namespaced scoreboards/storage rather than relying on tellraw text.
//!
//! Storage: `paudit:audit` — see each handler for the exact paths it
//! writes. Scoreboards (all `dummy`, ≤16 chars): `paudit_seq` (global
//! occurrence sequence number), `paudit_att1`/`paudit_att2` (per-handler
//! invocation counts for the two-handlers-share-one-occurrence scenario),
//! `paudit_kill`, `paudit_wpn`, `paudit_kwpn`.
//!
//! `#[event]` handler bodies cannot contain Rust `if`/`match` directly (the
//! attribute macro's restricted command DSL) — every branch on a
//! `ParticipantAvailability` result is built in a plain helper function
//! returning `Vec<String>` and called as a single statement, per the
//! documented "call a helper fn directly" pattern.

use sand::events::{
    EntityDamagePlayerEvent, EntityKillEvent, PlayerDamageEntityEvent, PlayerKillEvent,
};
use sand::participant::{EntityParticipant, ParticipantAvailability};
use sand::prelude::*;

const AUDIT_STORAGE: &str = "paudit:audit";

#[component]
fn load() -> Advancement {
    // Placeholder component so the pack always has at least one resource
    // even before any handler fires; keeps `sand build`'s output non-empty
    // for tooling that expects at least one advancement/function.
    Advancement::new("paudit:root".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
}

fn audit_load_commands() -> Vec<String> {
    vec![
        "scoreboard objectives add paudit_seq dummy".into(),
        "scoreboard objectives add paudit_att1 dummy".into(),
        "scoreboard objectives add paudit_att2 dummy".into(),
        "scoreboard objectives add paudit_kill dummy".into(),
        "scoreboard objectives add paudit_wpn dummy".into(),
        "scoreboard objectives add pwpn_pres dummy".into(),
        "scoreboard objectives add paudit_kwpn dummy".into(),
        "scoreboard players set audit_seq_holder paudit_seq 0".into(),
        format!("data modify storage {AUDIT_STORAGE} attacker.present set value 0b"),
        format!("data modify storage {AUDIT_STORAGE} killer.present set value 0b"),
        format!("data modify storage {AUDIT_STORAGE} weapon.present set value 0b"),
        format!("data modify storage {AUDIT_STORAGE} kill_weapon.present set value 0b"),
    ]
}

#[function]
fn init() -> Vec<String> {
    audit_load_commands()
}

fn bump_sequence_commands() -> Vec<String> {
    vec![
        "scoreboard players add audit_seq_holder paudit_seq 1".to_string(),
        format!(
            "execute store result storage {AUDIT_STORAGE} sequence int 1 run scoreboard players get audit_seq_holder paudit_seq"
        ),
    ]
}

fn attacker_a_commands(availability: ParticipantAvailability<EntityParticipant>) -> Vec<String> {
    let mut cmds = vec!["scoreboard players add @s paudit_att1 1".to_string()];
    cmds.extend(bump_sequence_commands());
    match availability {
        ParticipantAvailability::Available(attacker) => {
            let sel = attacker.selector().selector().to_string();
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} attacker.present set value 1b"
            ));
            cmds.push(format!(
                "execute at {sel} run data modify storage {AUDIT_STORAGE} attacker.uuid set from entity {sel} UUID"
            ));
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} subject.uuid set from entity @s UUID"
            ));
        }
        ParticipantAvailability::Unavailable(_) => {
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} attacker.present set value 0b"
            ));
        }
    }
    cmds
}

fn attacker_b_commands(availability: ParticipantAvailability<EntityParticipant>) -> Vec<String> {
    let mut cmds = vec!["scoreboard players add @s paudit_att2 1".to_string()];
    if let ParticipantAvailability::Available(attacker) = availability {
        let sel = attacker.selector().selector().to_string();
        cmds.push(format!(
            "execute at {sel} run data modify storage {AUDIT_STORAGE} attacker_b.uuid set from entity {sel} UUID"
        ));
    }
    cmds
}

fn killer_commands(availability: ParticipantAvailability<EntityParticipant>) -> Vec<String> {
    let mut cmds = vec!["scoreboard players add @s paudit_kill 1".to_string()];
    cmds.extend(bump_sequence_commands());
    match availability {
        ParticipantAvailability::Available(killer) => {
            let sel = killer.selector().selector().to_string();
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} killer.present set value 1b"
            ));
            cmds.push(format!(
                "execute at {sel} run data modify storage {AUDIT_STORAGE} killer.uuid set from entity {sel} UUID"
            ));
        }
        ParticipantAvailability::Unavailable(_) => {
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} killer.present set value 0b"
            ));
        }
    }
    cmds
}

fn weapon_commands(availability: ParticipantAvailability<sand::item::ItemSnapshot>) -> Vec<String> {
    let mut cmds = vec!["scoreboard players add @s paudit_wpn 1".to_string()];
    cmds.extend(bump_sequence_commands());
    match availability {
        ParticipantAvailability::Available(weapon) => {
            cmds.extend(
                weapon
                    .is_present()
                    .execute_commands(false, "scoreboard players set @s pwpn_pres 1"),
            );
            cmds.extend(
                weapon
                    .is_present()
                    .execute_commands(true, "scoreboard players set @s pwpn_pres 0"),
            );
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} weapon.present set value 1b"
            ));
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} weapon.item set from storage {} {}",
                weapon.storage(),
                weapon.item_path().as_str()
            ));
        }
        ParticipantAvailability::Unavailable(_) => {
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} weapon.present set value 0b"
            ));
        }
    }
    cmds
}

fn kill_weapon_commands(
    availability: ParticipantAvailability<sand::item::ItemSnapshot>,
) -> Vec<String> {
    let mut cmds = vec!["scoreboard players add @s paudit_kwpn 1".to_string()];
    cmds.extend(bump_sequence_commands());
    match availability {
        ParticipantAvailability::Available(weapon) => {
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} kill_weapon.present set value 1b"
            ));
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} kill_weapon.item set from storage {} {}",
                weapon.storage(),
                weapon.item_path().as_str()
            ));
        }
        ParticipantAvailability::Unavailable(_) => {
            cmds.push(format!(
                "data modify storage {AUDIT_STORAGE} kill_weapon.present set value 0b"
            ));
        }
    }
    cmds
}

/// `EntityDamagePlayerEvent` — correlated attacker. Two independent
/// handlers, both reading `.attacker()`, to validate that a same-occurrence
/// attacker binding is observable from more than one handler.
#[event]
pub fn audit_on_hurt_by_entity_a(event: Event<EntityDamagePlayerEvent>) {
    attacker_a_commands(event.attacker())
}

#[event]
pub fn audit_on_hurt_by_entity_b(event: Event<EntityDamagePlayerEvent>) {
    attacker_b_commands(event.attacker())
}

/// `PlayerKillEvent` — correlated killer.
#[event]
pub fn audit_on_killed(event: Event<PlayerKillEvent>) {
    killer_commands(event.killer())
}

/// `PlayerDamageEntityEvent` — weapon (mainhand) snapshot.
#[event]
pub fn audit_on_hurt_entity(event: Event<PlayerDamageEntityEvent>) {
    weapon_commands(event.weapon())
}

/// `EntityKillEvent` — weapon (mainhand) snapshot on a killing blow.
#[event]
pub fn audit_on_killed_entity(event: Event<EntityKillEvent>) {
    kill_weapon_commands(event.weapon())
}

#[doc(hidden)]
pub fn __sand_export(namespace: &str, mc_version: &str) {
    match __sand_export_json(namespace, mc_version) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("sand export failed: {e}");
            std::process::exit(1);
        }
    }
}

/// Testable core of [`__sand_export`] — returns the exported JSON instead of
/// printing/exiting, for `tests/deterministic_export.rs`.
#[doc(hidden)]
pub fn __sand_export_json(namespace: &str, mc_version: &str) -> Result<String, String> {
    let resolved = sand::version::resolve_export_caps(mc_version).map_err(|e| format!("{e}"))?;
    sand::advanced::try_export_components_json_for_version(
        namespace,
        &resolved.caps,
        &resolved.version,
        resolved.is_fallback,
    )
    .map_err(|e| format!("{e}"))
}
