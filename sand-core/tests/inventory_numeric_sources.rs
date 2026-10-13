//! Inventory reads compose with canonical State, arithmetic, and Conditions.
use sand::prelude::*;
use std::sync::OnceLock;

#[derive(State)]
#[state(namespace = "inventory_numbers", scope = player)]
struct Counts {
    diamonds: Score,
    #[state(scale = 100)]
    weighted: FixedScore,
}

static BODY: OnceLock<Actions> = OnceLock::new();
fn body() -> Actions {
    BODY.get_or_init(|| {
        Target::players().each(|player| {
            let count = player.inventory().count(sand::vanilla::Item::Diamond);
            let state = Counts::on(*player);
            let health = StatCurve::from(player.living().health());
            mcfunction![
                state.diamonds.set(count.clone());
                state.weighted.set(StatCurve::linear(count.clone(), 1.5, 0.0));
                when(count.gte(3).and(health.lt(10))).then_all(["say can heal"]);
            ]
        })
    })
    .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "count",
        make: body,
    }
}

#[test]
fn inventory_numbers_retain_non_mutating_helpers_across_exports() {
    let _ = body();
    let first = sand_core::try_export_components_json("inventory_numbers").unwrap();
    for _ in 0..2 {
        assert_eq!(
            first,
            sand_core::try_export_components_json("inventory_numbers").unwrap()
        );
    }
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let functions: std::collections::BTreeMap<_, _> = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .map(|record| {
            (
                record["path"].as_str().unwrap(),
                record["content"].as_str().unwrap(),
            )
        })
        .collect();
    let mut reads = 0;
    for body in functions.values() {
        assert!(!body.contains("__sand_local:"));
        for line in body.lines() {
            if let Some((_, clear)) = line.split_once("run clear ") {
                assert_eq!(clear, "@s minecraft:diamond 0");
                reads += 1;
            }
            if let Some((_, target)) = line.split_once("function inventory_numbers:") {
                assert!(
                    functions.contains_key(target.split_whitespace().next().unwrap()),
                    "missing helper in {line}"
                );
            }
        }
    }
    assert!(reads >= 3);
    assert!(functions.values().any(|body| body.contains("say can heal")));
    assert!(functions["__sand_score_init"].contains("scoreboard objectives add"));
}
