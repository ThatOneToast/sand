//! Effective attributes use owned canonical numeric actions and guards.
use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "attribute_values", scope = player)]
struct AttributeSnapshot {
    #[state(scale = 100)]
    speed: FixedScore,
}

fn attribute_actions() -> Actions {
    Target::players().each(|player| {
        let speed = player.living().attribute(AttributeType::MovementSpeed);
        mcfunction![
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
    sand_core::FunctionDescriptor {
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "snapshot", make: cached_attribute_actions,
    }
}

#[test]
fn cached_effective_attributes_own_helpers_for_every_export() {
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
    assert!(!first.contains("__sand_local:"));
}
