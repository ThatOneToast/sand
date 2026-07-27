//! Export-level coverage for validated `temp_score!` lowering (#146).
//!
//! The unit tests in `compiler::export::temp_scores` cover the lowering rules
//! directly. These drive the real public export pipeline end-to-end, so the
//! generated `__sand_temp_scores` function is asserted exactly as a datapack
//! would receive it.

use sand_core::{TempScoreboard, temp_score};

temp_score!(tsv_plain);
temp_score!(tsv_criterion, "playerKillCount");
temp_score!(tsv_display, "dummy", "Total Kills");

// Submitted directly so the objective name exceeds Minecraft's 16-character
// limit — `temp_score!` takes an identifier, and a long identifier is the
// realistic way a user hits this.
sand_core::inventory::submit! {
    TempScoreboard {
        name: "tsv_an_extremely_long_objective_name",
        criteria: "dummy",
        display_name: None,
    }
}

// A duplicate of `tsv_plain` — dedup is on `(name, criteria)`.
sand_core::inventory::submit! {
    TempScoreboard {
        name: "tsv_plain",
        criteria: "dummy",
        display_name: None,
    }
}

fn export() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("tempscores")
        .expect("temp score export must succeed");
    serde_json::from_str(&json).expect("export must be valid JSON")
}

fn temp_score_lines(records: &[serde_json::Value]) -> Vec<String> {
    let content = records
        .iter()
        .find(|r| {
            r["dir"] == "function" && r["path"] == "__sand_temp_scores" && r["ext"] == "mcfunction"
        })
        .expect("__sand_temp_scores must be generated")["content"]
        .as_str()
        .expect("content is text")
        .to_string();
    content.lines().map(str::to_string).collect()
}

#[test]
fn valid_objective_and_criterion_reach_the_generated_function() {
    let lines = temp_score_lines(&export());
    assert!(
        lines.contains(&"scoreboard objectives add tsv_plain dummy".to_string()),
        "got: {lines:#?}"
    );
    assert!(
        lines.contains(&"scoreboard objectives add tsv_criterion playerKillCount".to_string()),
        "got: {lines:#?}"
    );
}

#[test]
fn display_names_are_emitted_as_json_text_components() {
    let lines = temp_score_lines(&export());
    assert!(
        lines.contains(
            &r#"scoreboard objectives add tsv_display dummy {"text":"Total Kills"}"#.to_string()
        ),
        "a display name must be a JSON text component, not bare text; got: {lines:#?}"
    );
}

#[test]
fn long_objective_names_are_hashed_within_the_length_limit() {
    let lines = temp_score_lines(&export());
    let expected = sand_commands::ObjectiveName::logical("tsv_an_extremely_long_objective_name")
        .as_str()
        .to_string();
    assert!(expected.len() <= 16, "hashed name must fit: {expected}");
    assert!(
        lines.contains(&format!("scoreboard objectives add {expected} dummy")),
        "got: {lines:#?}"
    );
}

#[test]
fn every_generated_objective_fits_the_minecraft_length_limit() {
    for line in temp_score_lines(&export()) {
        let objective = line
            .strip_prefix("scoreboard objectives add ")
            .and_then(|rest| rest.split_whitespace().next())
            .unwrap_or_else(|| panic!("unexpected line: {line}"));
        assert!(
            objective.len() <= 16,
            "objective `{objective}` exceeds the 16-char limit (line: {line})"
        );
    }
}

#[test]
fn duplicate_declarations_are_deduplicated() {
    let lines = temp_score_lines(&export());
    let count = lines
        .iter()
        .filter(|l| l.as_str() == "scoreboard objectives add tsv_plain dummy")
        .count();
    assert_eq!(
        count, 1,
        "duplicate declarations must collapse; got: {lines:#?}"
    );
}

#[test]
fn repeated_exports_are_byte_identical() {
    let first = temp_score_lines(&export());
    for _ in 0..16 {
        assert_eq!(temp_score_lines(&export()), first);
    }
}
