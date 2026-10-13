//! Inventory reads compose with canonical State, arithmetic, and Conditions.
use sand::prelude::*;
use std::sync::OnceLock;

#[expect(
    dead_code,
    reason = "State declarations are accessed through generated runtime bindings."
)]
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
            let present = player.inventory().contains(sand::vanilla::Item::Diamond);
            let state = Counts::on(*player);
            let health = StatCurve::from(player.living().health());
            mcfunction![
                state.diamonds.set(count.clone());
                state.weighted.set(StatCurve::linear(count.clone(), 1.5, 0.0));
                when(count.gte(3).and(health.lt(10))).then_all(["say can heal"]);
                when(present.clone()).then_all(["say diamonds present"]);
                when(!present).then_all(["say diamonds absent"]);
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
    // Follow each emitted payload's caller rather than merely finding its text:
    // an unconditional, inverted, or same-polarity branch must fail this test.
    let mut presence_predicates = Vec::new();
    for (payload, polarity) in [
        ("say diamonds present", "if"),
        ("say diamonds absent", "unless"),
    ] {
        let (path, _) = functions
            .iter()
            .find(|(_, body)| body.lines().any(|line| line == payload))
            .expect("presence branch payload must be exported");
        let suffix = format!(" run function inventory_numbers:{path}");
        let callers: Vec<_> = functions
            .values()
            .flat_map(|body| body.lines())
            .filter(|line| line.ends_with(&suffix))
            .collect();
        assert_eq!(
            callers.len(),
            1,
            "unexpected callers for {payload}: {callers:?}"
        );
        let prefix = format!("execute {polarity} function inventory_numbers:");
        let predicate = callers[0]
            .strip_prefix(&prefix)
            .and_then(|line| line.strip_suffix(&suffix))
            .unwrap_or_else(|| panic!("wrong presence guard for {payload}: {}", callers[0]));
        assert!(
            functions.contains_key(predicate),
            "missing presence predicate {predicate}"
        );
        presence_predicates.push(predicate);
    }
    assert_eq!(
        presence_predicates[0], presence_predicates[1],
        "positive and negative branches must test the same predicate"
    );
    let predicate = functions[presence_predicates[0]];
    assert!(
        predicate
            .lines()
            .any(|line| line.contains(" > #comparison ") && line.ends_with(" run return 1")),
        "presence must use a strict positive comparison: {predicate}"
    );
    assert!(functions.values().any(|body| body.contains("say can heal")));
    assert!(functions["__sand_score_init"].contains("scoreboard objectives add"));
}
