//! Armor equip/unequip aggregation phase of the export pipeline.
//!
//! Owns the watch-map keying that groups armor watches by slot + item filters
//! and the `execute if items` condition rendering for the generated
//! `__sand_armor_check` tick function.

use std::collections::BTreeMap;

/// Exact semantic identity of an armor watch — slot byte, item ID filter,
/// and custom-data filter. Used as the aggregation map key so two watches
/// can only ever merge when every field of the tuple is identical; unlike a
/// sanitized/concatenated string, distinct filters cannot collide onto the
/// same key.
pub(crate) type ArmorWatchKey = (i8, Option<&'static str>, Option<&'static str>);

/// `(slot, item_id, custom_data_snbt, handlers)` grouped under one
/// [`ArmorWatchKey`]. `handlers` is `(is_equip, path)` for every descriptor
/// that watches this exact tuple.
pub(crate) type ArmorWatchEntry = (
    crate::function::ArmorSlot,
    Option<&'static str>,
    Option<&'static str>,
    Vec<(bool, &'static str)>,
);

/// Build the exact watch-map key for a slot + item + custom-data filter.
pub(crate) fn armor_watch_key(
    slot: crate::function::ArmorSlot,
    item_id: Option<&'static str>,
    custom_data: Option<&'static str>,
) -> ArmorWatchKey {
    (slot.slot_byte(), item_id, custom_data)
}

/// Build an unambiguous pre-hash seed from the exact watch tuple.
///
/// Length-prefixing each field (`sN:value` / `n`) keeps `None` distinct from
/// `Some("")` and keeps the field boundary unambiguous no matter what
/// characters `item_id`/`custom_data` contain — unlike joining sanitized
/// substrings with `_`, two different tuples can never produce the same
/// seed string.
fn armor_watch_tag_seed(
    slot: crate::function::ArmorSlot,
    item_id: Option<&str>,
    custom_data: Option<&str>,
) -> String {
    fn field(value: Option<&str>) -> String {
        match value {
            Some(value) => format!("s{}:{value}", value.len()),
            None => "n".to_string(),
        }
    }

    format!(
        "{}|{}|{}",
        slot.slot_byte(),
        field(item_id),
        field(custom_data)
    )
}

/// FNV-1a 32-bit hash, matching the algorithm already used for schedule
/// objective names ([`super::schedules::schedule_key`]) so generated names
/// stay consistent across the export pipeline.
fn fnv1a_hex(value: &str) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in value.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    format!("{h:08x}")
}

/// Deterministically allocate a bounded, private entity-tag key for every
/// registered armor watch.
///
/// The 32-bit seed hash can still collide between two distinct semantic
/// keys (birthday paradox, not a bug in the hash itself), so this walks all
/// watches in `ArmorWatchKey` order — independent of inventory registration
/// order, since `watches` is a `BTreeMap` keyed by the exact tuple — and
/// appends a stable `_<n>` disambiguating suffix to every key beyond the
/// first that maps to the same base tag. Two runs over the same watch set
/// always produce the same mapping.
pub(crate) fn allocate_armor_tag_keys(
    watches: &BTreeMap<ArmorWatchKey, ArmorWatchEntry>,
) -> BTreeMap<ArmorWatchKey, String> {
    let mut seen: BTreeMap<String, u32> = BTreeMap::new();
    let mut allocated = BTreeMap::new();

    for (key, (slot, item_id, custom_data, _)) in watches {
        let seed = armor_watch_tag_seed(*slot, *item_id, *custom_data);
        let base = format!("{}_{}", slot.tag_name_segment(), fnv1a_hex(&seed));

        let count = seen.entry(base.clone()).or_insert(0);
        *count += 1;
        let tag_key = if *count == 1 {
            base
        } else {
            format!("{base}_{count}")
        };
        allocated.insert(*key, tag_key);
    }

    allocated
}

pub(crate) fn build_item_cond(
    slot: crate::function::ArmorSlot,
    item_id: Option<&str>,
    custom_data: Option<&str>,
) -> String {
    let predicate = match (item_id, custom_data) {
        (None, _) => "*".to_string(),
        (Some(id), None) => id.to_string(),
        (Some(id), Some(cd)) => format!("{}[minecraft:custom_data~{}]", id, cd),
    };
    format!("items entity @s {} {}", slot.slot_name(), predicate)
}
