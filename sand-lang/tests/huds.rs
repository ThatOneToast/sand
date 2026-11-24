mod support;

use support::*;

const HUD: &str = r#"
#![pack(namespace = "scout", minecraft = "26.2")]
player state Scout { progress: number }
hud Survey {
    ticks: 100,
    every: 4,
    actionbar: "Survey in progress",
    bossbar: { title: "Survey", value: Scout.progress, max: 80 },
}
fn begin() { start_hud(Survey); }
fn finish() { cancel_hud(Survey); }
"#;

#[test]
fn authored_hud_uses_the_registered_lifetime_and_reads_canonical_state() {
    compile(&analyze_one(
        &HUD.replace("progress: number", "progress: timer"),
    ));
    let analysis = analyze_one(HUD);
    let pack = compile(&analysis);
    for (function_name, operation) in [("begin", "start"), ("finish", "cancel")] {
        assert!(
            function(&pack, &format!("scout/function/{function_name}")).contains(&format!(
                "function scout:__sand_system/lifetime/survey/{operation}"
            ))
        );
    }
    let create = function(
        &pack,
        "scout/function/__sand_system/lifetime/survey/hud_create",
    );
    assert!(
        create
            .contains("bossbar set scout:__sand_system/lifetime/survey/hud_bar/$(instance) max 80")
    );
    let refresh = function(
        &pack,
        "scout/function/__sand_system/lifetime/survey/hud_render",
    );
    assert!(refresh.contains(
        "bossbar set scout:__sand_system/lifetime/survey/hud_bar/$(instance) players @s"
    ));
    let document = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&document, &Default::default()).unwrap();
    let exported = sand::advanced::compiler::Compiler::compile(&decoded).unwrap();
    assert_eq!(pack.resources, exported.resources);
}

#[test]
fn huds_require_positive_times_channels_and_numeric_progress() {
    for (from, to, expected) in [
        ("ticks: 100", "ticks: 0", "expected a value from 1"),
        ("every: 4", "every: 0", "expected a value from 1"),
        ("progress: number", "progress: bool", "expected `number`"),
        ("max: 80", "max: 0", "expected a value from 1"),
    ] {
        let analysis = analyze_one(&HUD.replace(from, to));
        assert!(
            errors(&analysis)
                .iter()
                .any(|error| error.contains(expected)),
            "{:?}",
            errors(&analysis)
        );
    }
    let empty = analyze_one(r#"#![pack(namespace = "scout")] hud Survey { ticks: 100 }"#);
    assert!(
        errors(&empty)
            .iter()
            .any(|error| error.contains("needs an actionbar"))
    );
}
