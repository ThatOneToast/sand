mod support;

use sand::advanced::compiler::Compiler;
use support::*;

const SOURCE: &str = r#"
#![pack(namespace = "status_probe", minecraft = "26.2")]
player state Ledger { starts: number, ticks: number, stops: number }
status_effect Guard {
    ticks: 60,
    start: admitted,
    tick: stepped,
    stop: cleared,
}
status_effect Charge {
    ticks: 60,
    stacks: 2,
    start: admitted,
    stop: cleared,
}
fn admitted() { Ledger.starts += 1; }
fn stepped() { Ledger.ticks += 1; }
fn cleared() { Ledger.stops += 1; }
pub fn apply_guard() { apply_status(Guard); }
pub fn remove_guard() { remove_status(Guard); }
pub fn apply_charge() { apply_status(Charge); }
pub fn remove_charge() { remove_status(Charge); }
"#;

#[test]
fn statuses_round_trip_through_the_owned_runtime_with_all_callbacks() {
    let analysis = analyze_one(SOURCE);
    let pack = compile(&analysis);
    let commands = pack
        .resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, text)| std::str::from_utf8(text).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for operation in [
        "function status_probe:admitted",
        "function status_probe:stepped",
        "function status_probe:cleared",
        "start",
        "cancel",
    ] {
        assert!(commands.contains(operation), "{operation}");
    }
    let program = analysis.program.unwrap();
    assert_eq!(program.modules[0].status_effects[0].stacks, None);
    assert_eq!(program.modules[0].status_effects[1].stacks, Some(2));
    let encoded = serde_json::to_vec(&program).unwrap();
    let decoded = Compiler::decode(&encoded, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        Compiler::compile(&decoded).unwrap().resources
    );
}

#[test]
fn statuses_validate_context_duration_and_stack_count() {
    for (old, replacement, expected) in [
        ("ticks: 60", "ticks: 0", "expected a value from 1"),
        ("stacks: 2", "stacks: 129", "expected a value from 1 to 128"),
        (
            "pub fn apply_guard()",
            "#[load] fn apply_guard()",
            "needs a player",
        ),
    ] {
        let diagnostics = errors(&analyze_one(&SOURCE.replacen(old, replacement, 1)));
        assert!(
            diagnostics.iter().any(|error| error.contains(expected)),
            "{diagnostics:?}"
        );
    }
}
