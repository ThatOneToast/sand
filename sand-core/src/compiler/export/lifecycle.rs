//! Lifecycle and transition validation phase of the export pipeline.
//!
//! Owns the collision checks that keep Sand-generated private lifecycle and
//! transition function paths from overwriting user or component functions,
//! and the diagnostic error constructors for that phase.
#![allow(clippy::result_large_err)]

use super::records::{ComponentRecord, ExportResult};
use crate::component::ComponentExportError;

pub(crate) fn ensure_private_lifecycle_path_available(
    records: &[ComponentRecord],
    path: &str,
) -> ExportResult<()> {
    if records
        .iter()
        .any(|record| record.dir == "function" && record.path == path)
    {
        return Err(lifecycle_export_error(format!(
            "generated private function `{path}` collides with a user or component function"
        )));
    }
    Ok(())
}

pub(crate) fn lifecycle_export_error(message: impl Into<String>) -> ComponentExportError {
    ComponentExportError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "lifecycle")
            .expect("fixed lifecycle resource location is valid"),
        kind: "state_lifecycle".to_string(),
        field: "declarations".to_string(),
        message: message.into(),
    }
}

pub(crate) fn transition_export_error(message: impl Into<String>) -> ComponentExportError {
    ComponentExportError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "transitions")
            .expect("fixed transition resource location is valid"),
        kind: "tracked_transition".to_string(),
        field: "trackers".to_string(),
        message: message.into(),
    }
}

pub(crate) fn ensure_private_transition_path_available(
    records: &[ComponentRecord],
    path: &str,
    tracker_id: &str,
    source: &str,
) -> ExportResult<()> {
    if records
        .iter()
        .any(|record| record.dir == "function" && record.path == path)
    {
        return Err(transition_export_error(format!(
            "tracker `{tracker_id}` source `{source}` generated private function `{path}`, which collides with a user or component function"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::compiler::export::export_components_json;
    use crate::compiler::export::testing::{records_with_path, tag_values};

    #[test]
    fn lifecycle_load_objective_appears_in_export() {
        let _lock = crate::state::registry::registry_test_lock();
        // Drain any residual state from prior tests.
        let _ = crate::state::drain_load_commands();
        let _ = crate::state::drain_tick_commands();

        crate::state::register_load_objective("lc_test_mana", "dummy");

        let json_str = export_components_json("lcpack");
        let records: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        // A __sand_lifecycle_load function must exist.
        let load_recs = records_with_path(&records, "__sand_lifecycle_load");
        assert_eq!(
            load_recs.len(),
            1,
            "__sand_lifecycle_load record must appear exactly once"
        );
        assert_eq!(
            load_recs[0]["content"].as_str().unwrap_or(""),
            "scoreboard objectives add lc_test_mana dummy",
            "load function must contain exactly the registered objective command"
        );

        // The minecraft:load tag must reference it. Left as `.contains` (not
        // `assert_eq!`) because `tag_values` reads the global component
        // registry, which other tests (e.g. tags.rs) populate with their own
        // minecraft:load entries outside of `registry_test_lock`; the full
        // list's contents/order depend on which tests ran previously in this
        // process, so only membership is deterministic.
        let load_tag = tag_values(&records, "minecraft:load");
        assert!(
            load_tag.contains(&"lcpack:__sand_lifecycle_load".to_string()),
            "minecraft:load tag must contain lcpack:__sand_lifecycle_load, got: {load_tag:?}"
        );
    }

    #[test]
    fn lifecycle_tick_handler_appears_in_export() {
        let _lock = crate::state::registry::registry_test_lock();
        let _ = crate::state::drain_load_commands();
        let _ = crate::state::drain_tick_commands();

        crate::state::register_tick_handler(
            "lc_test/my_handler",
            vec!["scoreboard players remove @a lc_test_cd 1".to_string()],
        );

        let json_str = export_components_json("lcpack");
        let records: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        let tick_recs = records_with_path(&records, "__sand_lifecycle_tick");
        assert_eq!(
            tick_recs.len(),
            1,
            "__sand_lifecycle_tick record must appear exactly once"
        );
        assert_eq!(
            tick_recs[0]["content"].as_str().unwrap_or(""),
            "scoreboard players remove @a lc_test_cd 1",
            "tick function must contain exactly the registered handler commands"
        );

        // Left as `.contains` (not `assert_eq!`) for the same reason as the
        // minecraft:load tag check above: `tag_values` reads the global
        // component registry, which other tests can populate with their own
        // minecraft:tick entries outside of `registry_test_lock`.
        let tick_tag = tag_values(&records, "minecraft:tick");
        assert!(
            tick_tag.contains(&"lcpack:__sand_lifecycle_tick".to_string()),
            "minecraft:tick tag must contain lcpack:__sand_lifecycle_tick, got: {tick_tag:?}"
        );
    }

    #[test]
    fn empty_lifecycle_registry_produces_no_spurious_records() {
        let _lock = crate::state::registry::registry_test_lock();
        // Ensure both registries are empty before the export.
        let _ = crate::state::drain_load_commands();
        let _ = crate::state::drain_tick_commands();

        let json_str = export_components_json("lcpack");
        let records: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        assert!(
            records_with_path(&records, "__sand_lifecycle_load").is_empty(),
            "no __sand_lifecycle_load record should appear with empty registry"
        );
        assert!(
            records_with_path(&records, "__sand_lifecycle_tick").is_empty(),
            "no __sand_lifecycle_tick record should appear with empty registry"
        );
    }

    #[test]
    fn lifecycle_load_ordering_is_deterministic() {
        let _lock = crate::state::registry::registry_test_lock();
        let _ = crate::state::drain_load_commands();
        let _ = crate::state::drain_tick_commands();

        // Register in reverse alphabetical order.
        crate::state::register_load_objective("lc_zeta", "dummy");
        crate::state::register_load_objective("lc_alpha", "dummy");
        crate::state::register_load_objective("lc_mana", "dummy");

        let json_str = export_components_json("lcpack");
        let records: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        let load_recs = records_with_path(&records, "__sand_lifecycle_load");
        assert_eq!(load_recs.len(), 1);
        let content = load_recs[0]["content"].as_str().unwrap_or("");
        let lines: Vec<&str> = content.lines().collect();

        // BTreeMap guarantees alphabetical order.
        assert_eq!(
            lines,
            vec![
                "scoreboard objectives add lc_alpha dummy",
                "scoreboard objectives add lc_mana dummy",
                "scoreboard objectives add lc_zeta dummy",
            ],
            "commands must appear in alphabetical order, got: {lines:?}"
        );
    }
}
