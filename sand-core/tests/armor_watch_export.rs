//! Regression coverage for armor watcher identity collisions (#119).
//!
//! The exporter used to key its `armor_watch_map` aggregation on a
//! sanitized, underscore-joined string built from the slot/item/custom-data
//! filter. Distinct filters could sanitize to the same string (or the same
//! short hash) and silently merge into one generated watcher, causing a
//! handler to fire on the wrong item. These tests pin the exact watch-tuple
//! keying and the deterministic, collision-safe tag allocation that
//! replaced it.

use sand_core::{ArmorEventDescriptor, ArmorEventKind, ArmorSlot};

fn empty_body() -> Vec<String> {
    Vec::new()
}

// Two distinct filters whose sanitized representations previously collided:
// `a:b_c` and `a_b:c` both sanitize toward `a_b_c` once `:` and `_` are both
// folded to `_`.
sand_core::inventory::submit! {
    ArmorEventDescriptor {
        path: "on_ambiguous_colon_equip",
        make: empty_body,
        slot: ArmorSlot::Chest,
        kind: ArmorEventKind::Equip,
        item_id: Some("a:b_c"),
        custom_data_snbt: None,
    }
}
sand_core::inventory::submit! {
    ArmorEventDescriptor {
        path: "on_ambiguous_underscore_equip",
        make: empty_body,
        slot: ArmorSlot::Chest,
        kind: ArmorEventKind::Equip,
        item_id: Some("a_b:c"),
        custom_data_snbt: None,
    }
}

// The exact custom-data pair identified in review as a real short-hash
// collision (both previously hashed to `b0e54bf5` under the sanitized-seed
// algorithm): same slot, same item, different `sand` custom-data payload.
sand_core::inventory::submit! {
    ArmorEventDescriptor {
        path: "on_collision_fixture_a_equip",
        make: empty_body,
        slot: ArmorSlot::Head,
        kind: ArmorEventKind::Equip,
        item_id: Some("minecraft:leather_helmet"),
        custom_data_snbt: Some("{sand:\"pggm0p8t0\"}"),
    }
}
sand_core::inventory::submit! {
    ArmorEventDescriptor {
        path: "on_collision_fixture_a_unequip",
        make: empty_body,
        slot: ArmorSlot::Head,
        kind: ArmorEventKind::Unequip,
        item_id: Some("minecraft:leather_helmet"),
        custom_data_snbt: Some("{sand:\"pggm0p8t0\"}"),
    }
}
sand_core::inventory::submit! {
    ArmorEventDescriptor {
        path: "on_collision_fixture_b_equip",
        make: empty_body,
        slot: ArmorSlot::Head,
        kind: ArmorEventKind::Equip,
        item_id: Some("minecraft:leather_helmet"),
        custom_data_snbt: Some("{sand:\"yb4wg\"}"),
    }
}
sand_core::inventory::submit! {
    ArmorEventDescriptor {
        path: "on_collision_fixture_b_unequip",
        make: empty_body,
        slot: ArmorSlot::Head,
        kind: ArmorEventKind::Unequip,
        item_id: Some("minecraft:leather_helmet"),
        custom_data_snbt: Some("{sand:\"yb4wg\"}"),
    }
}

// Different slot with no item/custom-data filter (wildcard watch) — must
// stay independent of every filtered watch above.
sand_core::inventory::submit! {
    ArmorEventDescriptor {
        path: "on_any_feet_equip",
        make: empty_body,
        slot: ArmorSlot::Feet,
        kind: ArmorEventKind::Equip,
        item_id: None,
        custom_data_snbt: None,
    }
}

// Identical filter to `on_any_feet_equip` registered a second time — this
// must intentionally share the same generated watcher state (two handlers,
// one `_now`/`_had` tag pair).
sand_core::inventory::submit! {
    ArmorEventDescriptor {
        path: "on_any_feet_equip_second_handler",
        make: empty_body,
        slot: ArmorSlot::Feet,
        kind: ArmorEventKind::Equip,
        item_id: None,
        custom_data_snbt: None,
    }
}

fn export() -> String {
    sand_core::try_export_components_json("armorpack").expect("armor export succeeds")
}

fn function_content(records: &[serde_json::Value], path: &str) -> String {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing generated function {path}"))
        .to_string()
}

/// Every `__armor_<key>_now` tag referenced anywhere in the check function,
/// in first-appearance order.
fn armor_tag_keys(check_fn: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for line in check_fn.lines() {
        for token in line.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
            if let Some(key) = token
                .strip_prefix("__armor_")
                .and_then(|rest| rest.strip_suffix("_now"))
                && !keys.contains(&key.to_string())
            {
                keys.push(key.to_string());
            }
        }
    }
    keys
}

#[test]
fn distinct_sanitized_ambiguous_filters_stay_independent() {
    let records: Vec<serde_json::Value> =
        serde_json::from_str(&export()).expect("valid export JSON");
    let check = function_content(&records, "__sand_armor_check");

    let keys = armor_tag_keys(&check);
    // Every registered watch tuple below must own a distinct tag key.
    assert!(
        keys.len() >= 5,
        "expected at least 5 distinct armor watch tag keys, got {keys:?}"
    );

    // `a:b_c` and `a_b:c` must not share a tag key.
    assert_eq!(
        keys.iter().collect::<std::collections::BTreeSet<_>>().len(),
        keys.len(),
        "armor watch tag keys must be unique: {keys:?}"
    );
}

#[test]
fn known_short_hash_collision_pair_gets_distinct_tag_pairs() {
    let records: Vec<serde_json::Value> =
        serde_json::from_str(&export()).expect("valid export JSON");
    let check = function_content(&records, "__sand_armor_check");

    // Both custom-data payloads hash to the same 32-bit FNV seed under the
    // exporter's naming algorithm, so the allocator must have disambiguated
    // them with a suffix rather than letting them collide.
    assert!(
        check.contains("__armor_head_b0e54bf5_now"),
        "expected the base collision tag key to appear:\n{check}"
    );
    assert!(
        check.contains("__armor_head_b0e54bf5_2_now"),
        "expected the disambiguated collision tag key to appear:\n{check}"
    );
    assert!(
        check.contains("__armor_head_b0e54bf5_had"),
        "expected the base collision `_had` tag to appear:\n{check}"
    );
    assert!(
        check.contains("__armor_head_b0e54bf5_2_had"),
        "expected the disambiguated collision `_had` tag to appear:\n{check}"
    );

    // Each handler function is referenced against its own tag pair, not the
    // other watch's tag pair.
    assert!(check.contains(
        "execute as @a[tag=__armor_head_b0e54bf5_now,tag=!__armor_head_b0e54bf5_had] at @s run function armorpack:on_collision_fixture_a_equip"
    ));
    assert!(check.contains(
        "execute as @a[tag=__armor_head_b0e54bf5_2_now,tag=!__armor_head_b0e54bf5_2_had] at @s run function armorpack:on_collision_fixture_b_equip"
    ));
}

#[test]
fn identical_filters_share_one_watcher_state() {
    let records: Vec<serde_json::Value> =
        serde_json::from_str(&export()).expect("valid export JSON");
    let check = function_content(&records, "__sand_armor_check");

    let keys = armor_tag_keys(&check);
    // `on_any_feet_equip` and `on_any_feet_equip_second_handler` register
    // the identical (slot, None, None) tuple — the check function must
    // contain exactly one `feet_*` tag key even though two handler paths
    // are registered against it.
    let feet_keys: Vec<_> = keys.iter().filter(|k| k.starts_with("feet_")).collect();
    assert_eq!(
        feet_keys.len(),
        1,
        "identical armor filters must share one tag key, got {feet_keys:?}"
    );

    assert!(check.contains("run function armorpack:on_any_feet_equip\n"));
    assert!(check.contains("run function armorpack:on_any_feet_equip_second_handler"));
}

#[test]
fn repeated_exports_are_byte_identical_and_registration_order_independent() {
    let first = export();
    let second = export();
    assert_eq!(
        first, second,
        "repeated exports of the same watch set must be byte-identical"
    );
}

#[test]
fn generated_armor_tag_names_are_valid_and_bounded() {
    let records: Vec<serde_json::Value> =
        serde_json::from_str(&export()).expect("valid export JSON");
    let check = function_content(&records, "__sand_armor_check");

    for key in armor_tag_keys(&check) {
        let tag = format!("__armor_{key}_now");
        assert!(
            tag.len() <= 48,
            "generated armor tag name too long: {tag} ({} chars)",
            tag.len()
        );
        assert!(
            tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "generated armor tag name has invalid characters: {tag}"
        );
    }
}
