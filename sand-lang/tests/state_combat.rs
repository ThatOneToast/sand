mod support;

use support::*;

const COMBAT: &str = include_str!("../../examples/lang_state_combat/pack.sand");

#[test]
fn custom_health_uses_shared_combat_resources_and_runtime_operands() {
    let source =
        format!("{COMBAT}\nfn strike(amount: number) {{ combat_damage(Training, amount); }}");
    let analysis = analyze_one(&source);
    let pack = compile(&analysis);
    assert!(function(&pack, "scout_combat/function/strike").contains("function scout_combat:"));
    for expected in [
        "#absorbed",
        "function scout_combat:show_hit",
        "function scout_combat:show_defeat",
        "function scout_combat:show_healed",
    ] {
        assert!(
            pack.resources.iter().any(|(name, content)| {
                name.ends_with(".mcfunction") && String::from_utf8_lossy(content).contains(expected)
            }),
            "missing {expected}"
        );
    }
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
fn combat_validates_distinct_state_bindings_and_policy_bounds() {
    for (from, to, expected) in [
        (
            "shield: Vitals.shield",
            "shield: Vitals.health",
            "distinct fields",
        ),
        ("max: 20", "max: 0", "maximum must be finite and positive"),
        (
            "resistance: 0.25",
            "resistance: 2.0",
            "resistance must be between zero and one",
        ),
        (
            "timer: Vitals.immunity",
            "timer: Vitals.health",
            "expected a player State timer",
        ),
        (
            "multiplier: 2",
            "multiplier: 0.5",
            "multiplier must be finite and at least one",
        ),
    ] {
        let analysis = analyze_one(&COMBAT.replace(from, to));
        let diagnostics = analysis.compile().expect_err("invalid combat compiled");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "{diagnostics:?}"
        );
    }
}
