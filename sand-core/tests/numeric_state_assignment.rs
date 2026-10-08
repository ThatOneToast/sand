//! Runtime State assignments use the canonical numeric engine and export lifecycle.
use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "numeric_fixture", scope = player)]
struct Numbers {
    source: Score,
    destination: Score,
    #[state(default = 1.25, min = 0, max = 10, scale = 100)]
    fraction: FixedScore,
    #[state(default = 0, min = 0, max = 20, scale = 10)]
    result: FixedScore,
}

#[allow(dead_code)]
#[derive(StateQuery)]
#[query(scope = player)]
struct NumberPlayers {
    numbers: Numbers,
}

#[function]
fn assign_numbers() {
    NumberPlayers::each(|item| {
        let state = item.numbers;
        mcfunction![
            state.destination.set(state.source);
            state.result.set(StatCurve::add([
                StatCurve::from(state.fraction),
                StatCurve::constant(1.25),
            ]));
        ]
    });
}

#[test]
fn runtime_assignments_provision_scratch_before_lifecycle_and_export_stably() {
    let first = sand_core::try_export_components_json("numeric").unwrap();
    assert_eq!(
        first,
        sand_core::try_export_components_json("numeric").unwrap()
    );
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let functions: Vec<_> = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .collect();
    let helpers: Vec<_> = functions
        .iter()
        .filter(|record| {
            record["path"]
                .as_str()
                .unwrap()
                .starts_with("sand/numeric/")
        })
        .collect();
    assert_eq!(helpers.len(), 2);
    let copied = helpers
        .iter()
        .find(|record| {
            record["content"]
                .as_str()
                .unwrap()
                .contains(&format!("= @s {}", Numbers::source.objective()))
        })
        .unwrap();
    assert!(
        !copied["content"].as_str().unwrap().contains(" *= "),
        "same-scale integer copies must preserve the full i32 range"
    );
    let converted = helpers
        .iter()
        .find(|record| {
            record["content"]
                .as_str()
                .unwrap()
                .contains(&format!("= @s {}", Numbers::fraction.objective()))
        })
        .unwrap();
    let content = converted["content"].as_str().unwrap();
    assert!(content.contains(" += "));
    assert!(content.contains(" /= "));
    assert!(content.contains(&format!(
        "scoreboard players operation @s {} =",
        Numbers::result.objective()
    )));
    assert!(content.contains(&format!(
        "if score @s {} matches 201..",
        Numbers::result.objective()
    )));

    let declared: std::collections::BTreeSet<_> = functions
        .iter()
        .flat_map(|record| record["content"].as_str().unwrap().lines())
        .filter_map(|line| {
            line.strip_prefix("scoreboard objectives add ")
                .and_then(|rest| rest.split_whitespace().next())
        })
        .collect();
    for helper in helpers {
        for line in helper["content"].as_str().unwrap().lines() {
            let words: Vec<_> = line.split_whitespace().collect();
            if words.starts_with(&["scoreboard", "players"]) {
                assert!(
                    declared.contains(words[4]),
                    "undeclared destination: {line}"
                );
                if words[2] == "operation" {
                    assert!(declared.contains(words[7]), "undeclared source: {line}");
                }
            }
        }
    }
    let load = records
        .iter()
        .find(|record| {
            record["namespace"] == "minecraft"
                && record["path"] == "load"
                && record["dir"] == "tags/function"
        })
        .unwrap();
    let tag: serde_json::Value = serde_json::from_str(load["content"].as_str().unwrap()).unwrap();
    assert_eq!(tag["values"][0], "numeric:__sand_score_init");
}
