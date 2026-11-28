//! Player encounter selection and child ownership lower through the shared runtime.
mod support;
use support::*;

const PACK: &str = r#"
#![pack(namespace = "arena_probe", minecraft = "26.2")]
player state Arena { active: bool, charged: bool, count: number }

encounter Trial {
    ticks: 100,
    when: Arena.active,
    attacks: [
        { ticks: 10, when: Arena.charged, start: surge, stop: cleanup },
        { ticks: 6, start: recover },
    ],
    stop: cleanup,
}

fn surge() { Arena.count += 2; }
fn recover() { Arena.count += 1; }
fn cleanup() { Arena.charged = false; }

#[player]
fn run() { start_encounter(Trial); }

#[player]
fn stop() { cancel_encounter(Trial); }
"#;

#[test]
fn portable_encounters_register_priority_children_and_owner_cleanup() {
    let analysis = analyze_one(PACK);
    let pack = compile(&analysis);
    let start = function(&pack, "arena_probe/function/run");
    assert!(start.contains("/trial/cancel"));
    assert!(start.contains("/trial/start"));
    let select = function(
        &pack,
        "arena_probe/function/__sand_system/lifetime/trial/encounter_select",
    );
    assert!(select.contains("/children_finished/"), "{select}");
    assert_eq!(
        select.matches("return run function ").count(),
        3,
        "{select}"
    );
    for (index, callback) in ["surge", "recover"].into_iter().enumerate() {
        let owner = format!("/trial/attack_{index}/");
        assert!(pack.resources.keys().any(|path| path.contains(&owner)));
        assert!(
            pack.resources
                .values()
                .filter_map(|bytes| std::str::from_utf8(bytes).ok())
                .any(|body| body.contains(&format!("function arena_probe:{callback}")))
        );
    }
    let json = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded = sand::advanced::compiler::Compiler::decode(&json, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn encounters_validate_children_and_owner_guards() {
    for (old, replacement) in [
        ("when: Arena.active", "when: Arena.count"),
        ("ticks: 10", "ticks: 0"),
        ("start: surge", "stop: surge"),
        ("start: surge", "start: Missing"),
    ] {
        let analysis = analyze_one(&PACK.replace(old, replacement));
        assert!(
            analysis.has_errors() || analysis.compile().is_err(),
            "{replacement}"
        );
    }
}
