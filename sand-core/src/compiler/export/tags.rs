//! Function-tag assembly phase of the export pipeline.
//!
//! Owns the deterministic ordering rules for `tags/function` entries:
//! user-declared entries sort by (tag, function), while merged tag values
//! preserve first-seen execution order with duplicates removed.

pub(crate) fn dedupe_preserve_order(values: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut deduped = Vec::with_capacity(values.len());

    for value in values {
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }

    deduped
}

pub(crate) fn sort_function_tag_entries(entries: &mut [(String, String)]) {
    entries.sort_by(|(left_tag, left_function), (right_tag, right_function)| {
        left_tag
            .cmp(right_tag)
            .then_with(|| left_function.cmp(right_function))
    });
}

#[cfg(test)]
mod tests {
    use crate::compiler::export::export_components_json;
    use crate::compiler::export::testing::tag_values;

    inventory::submit! {
        crate::function::FunctionTagDescriptor {
            tag: "minecraft:load",
            function_path: "__test_user_load_after_setup",
        }
    }

    #[test]
    fn function_tag_values_dedupe_without_sorting() {
        let values = vec![
            "pack:z".to_string(),
            "pack:a".to_string(),
            "pack:z".to_string(),
            "pack:m".to_string(),
        ];

        assert_eq!(
            super::dedupe_preserve_order(values),
            vec![
                "pack:z".to_string(),
                "pack:a".to_string(),
                "pack:m".to_string()
            ]
        );
    }

    #[test]
    fn user_function_tag_entries_sort_deterministically() {
        let mut entries = vec![
            ("minecraft:tick".to_string(), "pack:z".to_string()),
            ("minecraft:load".to_string(), "pack:m".to_string()),
            ("minecraft:load".to_string(), "pack:a".to_string()),
        ];
        super::sort_function_tag_entries(&mut entries);
        assert_eq!(
            entries,
            vec![
                ("minecraft:load".to_string(), "pack:a".to_string()),
                ("minecraft:load".to_string(), "pack:m".to_string()),
                ("minecraft:tick".to_string(), "pack:z".to_string()),
            ]
        );
    }

    #[test]
    fn exported_load_tag_preserves_generated_insertion_order() {
        let _lock = crate::state::registry::registry_test_lock();
        let _ = crate::state::drain_load_commands();
        let _ = crate::state::drain_tick_commands();
        let _ = crate::state::score::drain_internal_score_setup();

        let _ = crate::state::ScoreConst::<i32>::new("tag order setup", 7).ref_();
        crate::state::register_load_objective("tag_order_life", "dummy");

        let json_str = export_components_json("orderpack");
        let records: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        assert_eq!(
            tag_values(&records, "minecraft:load"),
            vec![
                "orderpack:__sand_score_init".to_string(),
                "orderpack:__sand_lifecycle_load".to_string(),
                "orderpack:__test_user_load_after_setup".to_string(),
            ]
        );
    }
}
