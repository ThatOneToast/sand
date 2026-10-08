//! Numeric capabilities feed the canonical State assignment compiler.
use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "native_numeric", scope = player)]
struct NativeNumbers {
    #[state(scale = 100)]
    health: FixedScore,
    air: Score,
}

#[function]
fn snapshot_native() {
    Target::players().each(|player| {
        let state = NativeNumbers::on(*player);
        mcfunction![
            state.health.set(player.living().health());
            state.air.set(player.data().field::<i16>("Air"));
        ]
    });
}

#[test]
fn native_reads_are_owned_scaled_and_repeat_export_stable() {
    let first = sand_core::try_export_components_json("native_numeric").unwrap();
    assert_eq!(
        first,
        sand_core::try_export_components_json("native_numeric").unwrap()
    );
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let functions: Vec<_> = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .collect();
    for path in ["Health", "Air"] {
        let expected = format!("run data get entity @s {path} 1000");
        let body = functions
            .iter()
            .find_map(|record| {
                record["content"]
                    .as_str()
                    .filter(|body| body.contains(&expected))
            })
            .unwrap();
        assert_eq!(
            body.matches(&expected).count(),
            1,
            "native source is read once"
        );
        assert!(body.contains("execute store success score @s "));
        assert!(body.contains(" store result score @s "));
        assert!(body.contains("matches 1 run return fail"));
        assert!(body.contains("matches -2147483648 run return fail"));
        assert!(body.contains("matches 2147483647 run return fail"));
    }
}
