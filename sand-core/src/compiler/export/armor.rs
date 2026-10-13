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

/// Allocate namespace-owned armor tags through the shared identity allocator.
///
/// Exact watch tuples remain the aggregation identity. Their namespace-prefixed
/// seeds produce deterministic keys, and shared probing resolves hash collisions
/// without depending on inventory registration order.
pub(crate) fn allocate_armor_tag_keys(
    namespace: &str,
    watches: &BTreeMap<ArmorWatchKey, ArmorWatchEntry>,
) -> Result<BTreeMap<ArmorWatchKey, String>, crate::component::ComponentExportError> {
    let owners = watches
        .iter()
        .map(|(key, (slot, item, data, _))| {
            (
                *key,
                format!(
                    "{namespace}:armor:{}",
                    armor_watch_tag_seed(*slot, *item, *data)
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let segments = watches
        .iter()
        .map(|(key, (slot, _, _, _))| (owners[key].as_str(), slot.tag_name_segment()))
        .collect::<BTreeMap<_, _>>();
    let keys = super::identities::allocate_collision_safe_keys(
        owners.values().map(String::as_str),
        |owner, attempt| {
            let seed = if attempt == 0 {
                owner.to_owned()
            } else {
                format!("{owner}#{attempt}")
            };
            format!("{}_{}", segments[owner], fnv1a_hex(&seed))
        },
        |_, _| Ok(()),
        |owner, previous, key| crate::component::ComponentExportError::ComponentValidation {
            location: sand_components::ResourceLocation::new(namespace, "__sand_armor_check")
                .expect("export namespace is validated"),
            kind: "armor observation".into(),
            field: "private tag".into(),
            message: format!(
                "armor watches `{owner}` and `{previous}` cannot allocate distinct private tag key `{key}`"
            ),
        },
    )?;
    Ok(owners
        .into_iter()
        .map(|(watch, owner)| (watch, keys[&owner].clone()))
        .collect())
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
