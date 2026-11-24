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
            "no return value",
        ),
        (
            "fn eager() { for p in players() { wait(1); } }",
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
fn cancellation_groups_select_wait_sites_and_allow_missing_groups() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
fn first() { wait(1, group = "cooking"); }
fn second() { wait(seconds = 1, group = "cooking"); }
fn third() { wait(1, group = "deliveries"); }
fn fourth() { wait(1); }
fn cancel() {
    cancel_waits(group = "never_started");
    cancel_waits(group = "cooking");
    cancel_waits();
}
"#,
    );
    let encoded = serde_json::to_vec(analysis.program.as_ref().unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&encoded, &Default::default()).unwrap();
    let pack = compile(&analysis);
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
    let cooking = function(
        &pack,
        "wait_probe/function/__sand_program/cancel_waits/cooking",
    );
    let all = function(&pack, "wait_probe/function/__sand_program/cancel_waits");
    assert_eq!(cooking.lines().count(), 2);
    assert_eq!(all.lines().count(), 4);
    for command in cooking.lines() {
        assert!(all.lines().any(|line| line == command));
    }
    assert!(
        function(
            &pack,
            "wait_probe/function/__sand_program/cancel_waits/never_started",
        )
        .is_empty()
    );
}

#[test]
fn cancellation_group_names_are_checked_by_the_portable_compiler() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
fn invalid() {
    cancel_waits(group = "Wrong/name");
    wait(1, group = "");
}
"#,
    );
    let Err(diagnostics) = analysis.compile() else {
        panic!("invalid wait group names built successfully");
    };
    assert_eq!(
        diagnostics
            .iter()
            .filter(|d| d.message.contains("a wait group needs"))
            .count(),
        2
    );
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

#[test]
fn nested_conditional_waits_outline_one_suffix_and_keep_both_scopes() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
player state Result { sum: number, marker: number }
fn branch(a: bool, b: bool, c: bool) {
    let mut value = 5;
    if a || b || c {
        if a {
            let value = 7;
            wait(3);
            Result.marker = value;
        } else {
            wait(4);
            return;
        }
        value += 7;
    }
    Result.sum += value;
}
"#,
    );
    let pack = compile(&analysis);
    // Short-circuit operations must not duplicate a wait or its lifetime.
    assert_eq!(
        pack.resources
            .keys()
            .filter(|name| name.ends_with("/continuation.mcfunction"))
            .count(),
        2,
    );
    let functions = &analysis.program.as_ref().unwrap().modules[0].functions;
    assert!(
        functions
            .iter()
            .any(|function| function.id.path().starts_with("__wait/"))
    );
    let document = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&document, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn match_alternatives_and_short_circuit_guards_keep_single_wait_sites() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
player state Ledger { sum: number }
fn choose(mode: number) {
    match mode {
        1 | 3 => { wait(3); Ledger.sum += 1; },
        let candidate if candidate == 2 || candidate == 99 => {
            wait(4);
            Ledger.sum += candidate;
        },
        _ => { Ledger.sum += 100; },
    }
    Ledger.sum += 10;
}
"#,
    );
    let pack = compile(&analysis);
    assert_eq!(
        pack.resources
            .keys()
            .filter(|name| name.ends_with("/continuation.mcfunction"))
            .count(),
        2,
    );
}

#[test]
fn local_loops_outline_resumable_steps_and_preserve_collection_snapshots() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
player state Ledger { sum: number }
fn wait_loop(values: Vec<number>) {
    let mut total = 0;
    while total < 2 { wait(2); total += 1; }
    for value in values {
        if value < 0 { continue; }
        wait(3);
        if value == 10 { break; }
        total += value;
    }
    loop { wait(1); break; }
    Ledger.sum += total;
}
"#,
    );
    let pack = compile(&analysis);
    assert_eq!(
        pack.resources
            .keys()
            .filter(|name| name.ends_with("/continuation.mcfunction"))
            .count(),
        3,
    );
    let program = analysis.program.unwrap();
    assert!(
        program.modules[0]
            .functions
            .iter()
            .filter(|function| function.id.path().starts_with("__wait/"))
            .count()
            >= 3
    );
    let document = serde_json::to_vec(&program).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&document, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}
