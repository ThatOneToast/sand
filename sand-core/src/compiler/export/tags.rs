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
}
