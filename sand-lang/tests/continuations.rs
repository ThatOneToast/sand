mod support;
use support::*;

#[test]
fn wait_preserves_the_function_context_and_retains_typed_lexical_slots() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
player state Journal { notes: Vec<string> }
struct Payload { amount: number, label: string }
fn report(input: Payload) {
    let mut amount = input.amount;
    wait(ticks = 5);
    amount += 1;
    wait(seconds = 1);
    Journal.notes.push(input.label + amount.to_string());
}
fn cancel() { cancel_waits(); }
"#,
    );
    compile(&analysis);
    let program = serde_json::to_value(analysis.program.as_ref().unwrap()).unwrap();
    let functions = program["modules"][0]["functions"].as_array().unwrap();
    let report = functions
        .iter()
        .find(|function| function["id"] == "wait_probe:report")
        .unwrap();
    assert_eq!(report["context"], "player");
    let wait = report["body"].as_array().unwrap().last().unwrap();
    assert_eq!(wait["action"]["op"], "suspend");
    assert_eq!(wait["action"]["ticks"], 5);
    let saved = wait["action"]["fields"].as_array().unwrap();
    assert_eq!(saved.len(), 3);
    assert!(
        saved
            .iter()
            .any(|field| field["field"].as_str().unwrap().starts_with("__sand_data_"))
    );
    let next = wait["action"]["body"].as_array().unwrap().last().unwrap();
    assert_eq!(next["action"]["ticks"], 20);
    let map = analysis.source_map().unwrap();
    assert!(
        map["commands"]
            .as_object()
            .unwrap()
            .keys()
            .any(|name| name.ends_with("/continuation.mcfunction"))
    );
}

#[test]
fn wait_diagnostics_explain_unsupported_positions_and_delays() {
    for (body, expected) in [
        (
            "fn eager() -> number { wait(1); return 2; }",
            "returns no value",
        ),
        (
            "fn eager() { if true { wait(1); } }",
            "directly in a function body",
        ),
        ("fn eager() { wait(0); }", "expected a value from 1"),
        ("fn eager(n: string) { wait(n); }", "mismatched types"),
        ("#[load] fn eager() { wait(1); }", "on the server"),
    ] {
        let source = format!("#![pack(namespace = \"wait_probe\")]\n{body}");
        let diagnostics = errors(&analyze_one(&source));
        assert!(
            diagnostics.iter().any(|item| item.contains(expected)),
            "{body}: {diagnostics:?}"
        );
    }
}

#[test]
fn computed_wait_durations_are_read_once_and_stored_on_the_lifetime() {
    let source = r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
player state Ledger { resumes: number }
fn resume_later(seconds: number) {
    wait(seconds = seconds + 1);
    Ledger.resumes += 1;
}
"#;
    let analysis = analyze_one(source);
    let pack = compile(&analysis);
    let commands = pack
        .resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, bytes)| std::str::from_utf8(bytes).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(commands.contains("matches 1..107374182"));
    assert!(commands.contains("spawn.duration int 1 run scoreboard players get"));
    assert!(
        commands
            .lines()
            .filter(|line| line.starts_with("$scoreboard players set @s")
                && line.ends_with("$(duration)"))
            .count()
            == 2
    );
    assert!(commands.lines().any(|line| {
        line.contains("matches 1.. run return run function") && line.contains("/continuation")
    }));
}
