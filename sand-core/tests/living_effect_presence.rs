//! Cached living-effect Conditions own typed predicates and helper functions.
use sand::prelude::*;

fn body() -> Actions {
    static BODY: std::sync::OnceLock<Actions> = std::sync::OnceLock::new();
    BODY.get_or_init(|| {
        Target::players().each(|player| {
            let speedy = player.living().has_effect(EffectId::Speed);
            mcfunction![
                when(speedy.clone()).then_all(["say speedy"]);
                when(!speedy.clone()).then_all(["say no_speed"]);
                when(speedy.and(player.inventory().contains(vanilla::Item::Diamond)))
                    .then_all(["say speed_and_diamond"]);
            ]
        })
    })
    .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor { owner: module_path!(),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "query", make: body,
    }
}

fn verify(namespace: &str, json: &str) {
    let records: Vec<serde_json::Value> = serde_json::from_str(json).unwrap();
    let predicates: Vec<_> = records.iter().filter(|r| r["dir"] == "predicate").collect();
    assert_eq!(
        predicates.len(),
        1,
        "equivalent effect queries share one predicate"
    );
    let predicate = predicates[0];
    assert_eq!(predicate["namespace"], namespace);
    let content: serde_json::Value =
        serde_json::from_str(predicate["content"].as_str().unwrap()).unwrap();
    assert_eq!(
        content,
        serde_json::json!({
            "condition": "minecraft:entity_properties", "entity": "this",
            "predicate": {"minecraft:effects": {"minecraft:speed": {}}}
        })
    );
    let path = predicate["path"].as_str().unwrap();
    let helpers: Vec<_> = records
        .iter()
        .filter(|r| {
            r["dir"] == "function"
                && r["content"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("if predicate {namespace}:{path}"))
        })
        .collect();
    assert_eq!(
        helpers.len(),
        1,
        "positive and negative guards share one owner query"
    );
    let helper = helpers[0];
    assert_eq!(
        helper["content"],
        format!("execute as @s if predicate {namespace}:{path} run return 1\nreturn 0")
    );
    let helper_path = helper["path"].as_str().unwrap();
    let functions = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .map(|r| r["content"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(functions.contains(&format!("if function {namespace}:{helper_path}")));
    assert!(functions.contains(&format!("unless function {namespace}:{helper_path}")));
    assert!(functions.contains("say speed_and_diamond"));
    assert!(functions.contains("clear @s minecraft:diamond 0"));
    assert!(!json.contains("__sand_local:"));
}

#[test]
fn effect_predicates_survive_cold_cached_and_cross_namespace_exports() {
    let first = sand_core::try_export_components_json("effect_presence").unwrap();
    verify("effect_presence", &first);
    assert_eq!(
        first,
        sand_core::try_export_components_json("effect_presence").unwrap()
    );
    let second = sand_core::try_export_components_json("second_pack").unwrap();
    verify("second_pack", &second);
    assert!(!second.contains("effect_presence:"));
}
