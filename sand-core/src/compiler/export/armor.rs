//! Armor equip/unequip aggregation phase of the export pipeline.
//!
//! Owns the watch-map keying that groups armor watches by slot + item filters
//! and the `execute if items` condition rendering for the generated
//! `__sand_armor_check` tick function.

/// Build the unique key used to group armor watch entries by slot + filters.
pub(crate) fn armor_watch_key(
    slot: crate::function::ArmorSlot,
    item_id: Option<&str>,
    custom_data: Option<&str>,
) -> String {
    let mut parts = vec![slot.tag_name_segment().to_string()];
    if let Some(id) = item_id {
        parts.push(sanitize_armor_tag(id));
    }
    if let Some(cd) = custom_data {
        parts.push(sanitize_armor_tag(cd));
    }
    parts.join("_")
}

fn sanitize_armor_tag(s: &str) -> String {
    let raw: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    raw.trim_matches('_').to_string()
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
