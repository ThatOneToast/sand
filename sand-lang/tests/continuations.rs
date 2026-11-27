mod support;
use support::*;

#[test]
fn composed_wait_runtime_fixture_compiles() {
    compile(&analyze_one(include_str!("fixtures/composed_waits.sand")));
}

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
            "fn eager() { let _result = { wait(1); 2 }; }",
            "use `wait` directly",
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
fn returning_calls_retain_caller_locals_and_intermediate_values_through_waits() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
global state Journal { label: string, total: number }
fn later(delay: number, value: number) -> number {
    if delay == 0 { return value; }
    wait(delay, server = true);
    return value * 2;
}
fn twice(delay: number, value: number) -> number {
    return later(delay, value) + later(delay, value + 1);
}
pub fn run(delay: number, label: string) {
    let initial = delay + 1;
    Journal.total = initial + twice(delay, 7);
    Journal.label = label;
}
"#,
    );
    let pack = compile(&analysis);
    let program = serde_json::to_value(analysis.program.as_ref().unwrap()).unwrap();
    let functions = program["modules"][0]["functions"].as_array().unwrap();
    let saved = functions
        .iter()
        .filter(|function| function["id"].as_str().unwrap().ends_with("/run"))
        .map(|function| serde_json::to_string(&function["body"]).unwrap())
        .find(|body| body.contains("\"op\":\"suspend\"") && body.contains("_label"))
        .expect("the wait retains the original caller's owned string");
    assert!(saved.contains("_t0"));
    assert!(
        !serde_json::to_string(&program)
            .unwrap()
            .contains("__sand_wait_call")
    );
    assert!(
        pack.resources
            .keys()
            .any(|path| path.contains("server_wait_drain"))
    );
}

#[test]
fn waiting_method_commits_the_local_receiver_before_the_caller_resumes() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
struct Count { value: number }
impl Count {
    fn add(&mut self, amount: number) -> number {
        wait(2, server = true);
        self.value += amount;
        return self.value;
    }
}
global state Journal { total: number }
pub fn run(index: number) {
    let mut counts = [Count { value: 1 }, Count { value: 10 }];
    Journal.total = counts[index].add(2) + counts[index].value;
}
"#,
    );
    compile(&analysis);
}

#[test]
fn composed_waits_reject_recursion_and_shared_borrowed_receivers() {
    for (source, expected) in [
        (
            "fn retry(value: number) -> number { wait(1, server = true); return retry(value); }",
            "cannot be recursive",
        ),
        (
            r#"
struct Count { value: number }
impl Count {
    fn add(&mut self) { wait(1, server = true); self.value += 1; }
}
global state Journal { count: Count }
pub fn run() { Journal.count.add(); }
"#,
            "held across a wait must be local",
        ),
    ] {
        let source = format!("#![pack(namespace = \"wait_probe\")]\n{source}");
        let diagnostics = errors(&analyze_one(&source));
        assert!(
            diagnostics.iter().any(|item| item.contains(expected)),
            "{diagnostics:?}"
        );
    }
}

#[test]
fn branching_return_chains_report_excessive_wait_specialization() {
    let mut source = r#"
#![pack(namespace = "wait_probe")]
fn level_0() -> number { wait(1, server = true); return 1; }
"#
    .to_owned();
    for level in 1..10 {
        let previous = level - 1;
        source.push_str(&format!(
            "fn level_{level}() -> number {{ return level_{previous}() + level_{previous}(); }}\n"
        ));
    }
    let diagnostics = errors(&analyze_one(&source));
    assert!(
        diagnostics
            .iter()
            .any(|message| message.contains("too many distinct continuations")),
        "{diagnostics:?}"
    );
}

#[test]
fn selector_waits_keep_their_executor_and_do_not_suspend_the_caller() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
player state Journal { score: number }
global state World { finished: bool }
fn run(delay: number) -> number {
    for player in players() {
        if delay > 0 { wait(delay); }
        Journal.score += delay;
    }
    World.finished = true;
    return delay;
}
"#,
    );
    compile(&analysis);
    let functions = &analysis.program.unwrap().modules[0].functions;
    let run = functions.iter().find(|f| f.id.path() == "run").unwrap();
    assert_eq!(
        run.context,
        sand::advanced::compiler::ExecutionContext::Server
    );
    assert!(matches!(
        run.body[0].action,
        sand::advanced::compiler::Action::Players { .. }
    ));
    assert!(functions.iter().any(|f| f.id.path().starts_with("__wait/")
        && f.context == sand::advanced::compiler::ExecutionContext::Player));
}

#[test]
fn server_waits_compile_without_a_player_and_keep_cancellation_separate() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
global state World { visits: number }
fn run(delay: number) {
    wait(ticks = delay, server = true, group = "server");
    World.visits += delay;
}
fn clear() { cancel_waits(server = true, group = "server"); }
"#,
    );
    let pack = compile(&analysis);
    let tick = function(&pack, "wait_probe/function/__sand_lifecycle_tick");
    assert!(tick.contains(".draining set from storage"));
    assert!(tick.contains("server_wait_drain"));
    let clear = function(
        &pack,
        "wait_probe/function/__sand_program/cancel_server_waits/server",
    );
    assert!(clear.contains("{pending:[],draining:[]}"));

    let invalid = analyze_one(
        r#"
#![pack(namespace = "wait_probe")]
#[player]
fn run() { wait(1, server = true); }
"#,
    );
    let Err(errors) = invalid.compile() else {
        panic!("server wait allowed in a player frame");
    };
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("must run in server context"))
    );
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
fn branching_paths_keep_one_wait_site_for_each_authored_call() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
player state Ledger { sum: number }
fn delayed() -> number { wait(1); return 1; }
fn delayed_test() -> bool { wait(1); return true; }
fn choose(mode: number) {
    if mode == 1 || mode == 2 || mode == 3 { Ledger.sum += delayed(); }
    match mode {
        1 | 2 | 3 => { Ledger.sum += delayed(); },
        _ => {},
    }
}
fn check(mode: number) {
    if (mode == 1 || mode == 2) && delayed_test() { Ledger.sum += 1; }
}
fn select(mode: number) {
    if mode == 1 || mode == 2 || mode == 3 {
        for player in players() { wait(1); Ledger.sum += 1; }
    }
}
"#,
    );
    let pack = compile(&analysis);
    assert_eq!(
        pack.resources
            .keys()
            .filter(|name| name.ends_with("/continuation.mcfunction"))
            .count(),
        6,
    );
}

#[test]
fn a_wide_match_does_not_exhaust_the_wait_site_budget() {
    let alternatives = (1..=80)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(" | ");
    let analysis = analyze_one(&format!(
        r#"
#![pack(namespace = "wait_probe", minecraft = "26.2")]
global state Ledger {{ sum: number }}
fn delayed() -> number {{ wait(1, server = true); return 1; }}
fn choose(mode: number) {{
    match mode {{
        {alternatives} => {{ Ledger.sum += delayed(); }},
        _ => {{}},
    }}
}}
"#
    ));
    compile(&analysis);
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
