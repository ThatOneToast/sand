mod support;
use support::*;

#[test]
fn native_transitions_share_one_provider_per_flag_and_keep_handler_order() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "motion", minecraft = "26.2")]
player state History { events: number }
#[on(start_sneaking)] fn first() { History.events += 1; }
#[on(start_sneaking)] fn second() { History.events += 2; }
#[on(stop_sneaking)] fn stop() { History.events += 4; }
#[on(start_sprinting)] fn sprint() { History.events += 8; }
#[on(stop_sprinting)] fn stand() { History.events += 16; }
#[on(start_swimming)] fn swim() { History.events += 32; }
#[on(stop_swimming)] fn surface() { History.events += 64; }
#[on(start_flying)] fn fly() { History.events += 128; }
#[on(stop_flying)] fn land() { History.events += 256; }
"#,
    );
    let pack = compile(&analysis);
    let dispatch = function(&pack, "motion/function/__sand_program/event/start_sneaking");
    assert!(
        dispatch.find("function motion:first").unwrap()
            < dispatch.find("function motion:second").unwrap()
    );
    for flag in ["is_sneaking", "is_sprinting", "is_swimming"] {
        let predicate = format!(
            "data/motion/predicate/__sand/player_{}.json",
            flag.trim_start_matches("is_")
        );
        let json: serde_json::Value = serde_json::from_slice(&pack.resources[&predicate]).unwrap();
        assert_eq!(json["predicate"]["flags"][flag], true);
        let predicate_id = format!(
            "predicate motion:__sand/player_{}",
            flag.trim_start_matches("is_")
        );
        let samples = pack
            .resources
            .values()
            .filter_map(|bytes| std::str::from_utf8(bytes).ok())
            .map(|content| content.matches(&predicate_id).count())
            .sum::<usize>();
        assert_eq!(samples, 1, "{flag}");
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
fn native_and_custom_hooks_can_filter_a_typed_event_chain() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "motion", minecraft = "26.2")]
predicate Always { chance: 1 }
predicate Never { chance: 0 }

event Moved(amount: number);
player state History { events: number }

#[on(start_sneaking, when = Always)]
fn start() { emit Moved(3); }

#[on(Moved, when = Never)]
fn skipped(amount: number) { History.events += amount; }

#[on(Moved)]
fn counted(amount: number) { History.events += amount; }
"#,
    );
    let pack = compile(&analysis);
    let dispatch = function(&pack, "motion/function/__sand_program/event/start_sneaking");
    assert!(dispatch.contains("execute if predicate motion:always run function motion:start"));
    assert!(function(&pack, "motion/function/start").contains("function motion:sand_event/moved"));
    let custom = function(&pack, "motion/function/sand_event/moved");
    assert!(custom.contains("function motion:counted"), "{custom}");
    assert!(
        pack.resources.iter().any(|(path, bytes)| {
            path.ends_with(".mcfunction")
                && std::str::from_utf8(bytes)
                    .is_ok_and(|body| body.contains("predicate motion:never"))
        }),
        "{custom}"
    );
    assert!(pack.resources.values().any(|bytes| {
        std::str::from_utf8(bytes).is_ok_and(|body| body.contains("function motion:skipped"))
    }));

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
fn event_guards_require_an_authored_predicate() {
    for guard in ["Missing", "Biome::Desert", "17"] {
        let source = format!(
            "#![pack(namespace = \"motion\")] #[on(start_sneaking, when = {guard})] fn start() {{}}",
        );
        assert!(analyze_one(&source).has_errors(), "{guard}");
    }
}
