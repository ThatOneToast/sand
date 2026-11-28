mod support;
use support::*;

#[test]
fn native_world_predicate_is_shared_by_execute_and_loot() {
    let compiled = compile(&analyze_one(include_str!("fixtures/world_predicates.sand")));
    let predicate: serde_json::Value =
        serde_json::from_slice(&compiled.resources["data/language_probe/predicate/camp.json"])
            .unwrap();
    assert_eq!(predicate["condition"], "minecraft:all_of");
    assert_eq!(
        predicate["terms"][0]["condition"],
        "minecraft:location_check"
    );
    assert_eq!(predicate["terms"][1]["chance"], 1.0);
    let loot: serde_json::Value =
        serde_json::from_slice(&compiled.resources["data/language_probe/loot_table/supplies.json"])
            .unwrap();
    assert_eq!(
        loot["pools"][0]["conditions"][0]["name"],
        "language_probe:camp"
    );
    assert!(compiled.resources.values().any(|body| {
        String::from_utf8_lossy(body).contains("execute if predicate language_probe:camp")
    }));
}

#[test]
fn predicates_require_valid_conditions() {
    for (body, expected) in [
        ("predicate Empty {}", "needs at least one condition"),
        (
            "predicate Invalid { chance: 2 }",
            "chance must be between zero and one",
        ),
        (
            "fn run() { if predicate_matches(\"missing_namespace\") { return; } }",
            "expected a namespaced predicate",
        ),
    ] {
        let issues = errors(&analyze_one(&format!(
            "#![pack(namespace = \"rules\")]\n{body}"
        )));
        assert!(
            issues.iter().any(|issue| issue.contains(expected)),
            "{issues:?}"
        );
    }
}
