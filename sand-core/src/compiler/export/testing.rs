//! Test-only helpers shared by export pipeline phase tests.

/// Parse the JSON output of `export_components_json` and return the subset of
/// records matching the given `path`.
pub(crate) fn records_with_path<'a>(
    records: &'a [serde_json::Value],
    path: &str,
) -> Vec<&'a serde_json::Value> {
    records
        .iter()
        .filter(|r| r["path"].as_str() == Some(path))
        .collect()
}

/// Return the "values" array from the tag record for `tag_rl` (e.g.
/// `"minecraft:load"`), or an empty vec if no such record exists.
pub(crate) fn tag_values(records: &[serde_json::Value], tag_rl: &str) -> Vec<String> {
    // Tag records: dir = "tags/function", path = everything after the colon.
    let tag_path = tag_rl.split_once(':').map(|(_, p)| p).unwrap_or(tag_rl);
    for r in records {
        if r["dir"].as_str() == Some("tags/function")
            && r["path"].as_str() == Some(tag_path)
            && let Some(arr) = r["content"]
                .as_str()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .and_then(|v| v["values"].as_array().cloned())
        {
            return arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect();
        }
    }
    Vec::new()
}
