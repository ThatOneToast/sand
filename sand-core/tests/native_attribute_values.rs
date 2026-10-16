//! Effective and base attributes use owned canonical numeric actions and guards.
use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "attribute_values", scope = player)]
struct AttributeSnapshot {
    #[state(scale = 100)]
    speed: FixedScore,
    #[state(scale = 100)]
    base_speed: FixedScore,
}

fn attribute_actions() -> Actions {
    Target::players().each(|player| {
        let speed = player.living().attribute(AttributeType::MovementSpeed);
        let base = player.living().attribute_base(AttributeType::MovementSpeed);
        mcfunction![
            AttributeSnapshot::on(*player).base_speed.set(base.clone());
            when(speed.gt(base)).then_all(["say boosted"]);
            AttributeSnapshot::on(*player).speed.set(speed.clone());
            when(speed.gt(0)).then_all(["say moving"]);
        ]
    })
}

fn cached_attribute_actions() -> Actions {
    static ACTIONS: std::sync::OnceLock<Actions> = std::sync::OnceLock::new();
    ACTIONS.get_or_init(attribute_actions).clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor { owner: module_path!(),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "snapshot", make: cached_attribute_actions,
    }
}

#[test]
fn cached_native_attributes_own_helpers_for_every_export() {
    let first = sand_core::try_export_components_json("attribute_values").unwrap();
    assert_eq!(
        first,
        sand_core::try_export_components_json("attribute_values").unwrap()
    );
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let queries: Vec<_> = records
        .iter()
        .filter_map(|r| r["content"].as_str())
        .filter(|body| body.contains("run attribute @s minecraft:movement_speed get 1000"))
        .collect();
    assert!(
        queries.len() >= 2,
        "assignment and guard must both retain native reads"
    );
    for query in queries {
        assert_eq!(
            query
                .matches("run attribute @s minecraft:movement_speed get 1000")
                .count(),
            1
        );
        assert!(query.contains("store success score #attribute"));
        assert!(query.contains("store result score #attribute"));
    }
    let base_queries: Vec<_> = records
        .iter()
        .filter_map(|r| r["content"].as_str())
        .filter(|body| body.contains("run attribute @s minecraft:movement_speed base get 1000"))
        .collect();
    assert!(
        base_queries.len() >= 2,
        "base assignment and mixed comparison retain distinct sources"
    );
    for query in base_queries {
        assert_eq!(query.matches(" base get 1000").count(), 1);
        assert!(query.contains("store success score #attribute"));
    }
    assert!(first.contains("say boosted"));
    assert!(!first.contains("__sand_local:"));
}
