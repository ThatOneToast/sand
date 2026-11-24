mod support;

use support::*;

const PROJECTILE: &str = include_str!("../../examples/lang_projectiles/pack.sand");

#[test]
fn projectile_exports_native_flight_filters_callbacks_and_damage() {
    let analysis = analyze_one(PROJECTILE);
    let pack = compile(&analysis);
    let base = "scout_sparks/function/__sand_system/lifetime/spark";
    assert!(function(&pack, "scout_sparks/function/cast").contains("spark/start"));
    assert!(function(&pack, "scout_sparks/function/cancel").contains("spark/cancel"));
    assert!(
        function(&pack, &format!("{base}/projectile_search")).contains("type=minecraft:zombie")
    );
    assert!(function(&pack, &format!("{base}/projectile_move")).contains("tp @s ^ ^ ^0.25"));
    assert!(pack.resources.iter().any(|(name, content)| {
        name.ends_with(".mcfunction")
            && String::from_utf8_lossy(content).contains("6 minecraft:generic by")
    }));
    assert!(pack.resources.iter().any(|(name, content)| {
        name.ends_with(".mcfunction")
            && String::from_utf8_lossy(content).contains("function scout_sparks:impact")
    }));
    let portable = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&portable, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn projectile_diagnostics_guard_bounds_callbacks_and_player_context() {
    for (from, to, expected) in [
        ("ticks: 24", "ticks: 0", "expected a value from 1"),
        ("radius: 0.75", "radius: 0.001", "more than 256 samples"),
        ("damage: 6", "damage: -1", "damage"),
        (
            "fn impact()",
            "fn impact(value: number)",
            "parameterless function",
        ),
        ("pub fn cast()", "#[load] pub fn cast()", "without a player"),
    ] {
        let analysis = analyze_one(&PROJECTILE.replace(from, to));
        let messages: Vec<_> = analysis
            .compile()
            .expect_err("invalid projectile compiled")
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect();
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "expected {expected:?}, found {messages:?}"
        );
    }
}
