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

/// Finalize explicit tag memberships after compiler lifecycle work.
pub(crate) fn assemble_tags(
    namespace: &str,
    records: &mut Vec<super::records::ComponentRecord>,
    mut tag_map: std::collections::BTreeMap<String, Vec<String>>,
    mut user_tag_entries: Vec<(String, String)>,
) {
    use super::records::ComponentRecord;
    sort_function_tag_entries(&mut user_tag_entries);
    for (tag, function) in user_tag_entries {
        tag_map.entry(tag).or_default().push(function);
    }

    // ── Finalize tag_map → records ────────────────────────────────────────────
    for (tag_rl, values) in tag_map {
        let (tag_ns, tag_path) = match tag_rl.split_once(':') {
            Some((ns, path)) => (ns.to_string(), path.to_string()),
            None => (namespace.to_string(), tag_rl.clone()),
        };
        // Registration can reach the same lifecycle tag through multiple
        // framework paths. Preserve first-seen execution order while emitting
        // each function reference only once.
        let values = dedupe_preserve_order(values);
        let json = serde_json::json!({ "values": values });
        records.push(ComponentRecord {
            namespace: tag_ns,
            dir: "tags/function".to_string(),
            path: tag_path,
            ext: "json".to_string(),
            content_type: "text".to_string(),
            content: serde_json::to_string_pretty(&json).unwrap(),
        });
    }
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
