mod support;

use support::*;

const ABILITY: &str = include_str!("../../examples/lang_abilities/pack.sand");

#[test]
fn ability_shares_projectile_admission_with_state_cost_and_cooldown() {
    let analysis = analyze_one(ABILITY);
    let pack = compile(&analysis);
    let base = "scout_magic/function/__sand_system/lifetime/cast_spark";
    assert!(function(&pack, "scout_magic/function/cast").contains("cast_spark/start"));
    assert!(function(&pack, "scout_magic/function/status").contains("cast_spark/diagnose"));
    assert!(
        function(&pack, &format!("{base}/projectile_search")).contains("type=minecraft:zombie")
    );
    assert!(pack.resources.iter().any(|(name, content)| {
        name.ends_with(".mcfunction") && {
            let commands = String::from_utf8_lossy(content);
            commands.contains("scoreboard players remove @s")
                && commands.lines().any(|line| {
                    line.starts_with("scoreboard players set @s ") && line.ends_with(" 20")
                })
        }
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
fn ability_requires_the_right_state_field_and_callback_types() {
    for (from, to, expected) in [
        (
            "balance: Focus.mana",
            "balance: Focus.cooldown",
            "non-ticking player State number",
        ),
        (
            "timer: Focus.cooldown",
            "timer: Focus.mana",
            "player State timer",
        ),
        (
            "projectile: Spark",
            "projectile: CastSpark",
            "expected a declared projectile",
        ),
        (
            "fn cast_message()",
            "fn cast_message(value: number)",
            "parameterless function",
        ),
    ] {
        let analysis = analyze_one(&ABILITY.replace(from, to));
        let diagnostics = analysis.compile().expect_err("invalid ability compiled");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "{diagnostics:?}"
        );
    }
}
