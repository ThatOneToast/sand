//! Recursion: calls that can re-enter their caller save its variables on a
//! storage stack, and every other call stays a plain function call.

mod support;

use support::*;

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

fn compiled(body: &str) -> sand::advanced::compiler::CompiledPack {
    let analysis = analyze_one(&format!("{PACK}{body}"));
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));
    compile(&analysis)
}

/// Every command of the pack, for checks that span helper functions.
fn all_commands(pack: &sand::advanced::compiler::CompiledPack) -> String {
    pack.resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, bytes)| String::from_utf8_lossy(bytes).into_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The portable operations of one function, as JSON.
fn body_of(source: &str, name: &str) -> Vec<serde_json::Value> {
    let analysis = analyze_one(&format!("{PACK}{source}"));
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));
    let json = serde_json::to_value(analysis.program.as_ref().expect("program")).unwrap();
    json["modules"][0]["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"].as_str().unwrap().ends_with(&format!(":{name}")))
        .unwrap_or_else(|| panic!("no function {name}"))["body"]
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn a_recursive_function_saves_its_frame_around_the_self_call() {
    let pack = compiled(
        r#"
fn factorial(n: number) -> number {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
}
"#,
    );
    let commands = all_commands(&pack);
    let push = commands
        .find("data modify storage demo:__sand_frames stack append value {}")
        .unwrap_or_else(|| panic!("no frame push in\n{commands}"));
    let call = commands[push..]
        .find("function demo:factorial")
        .map(|offset| push + offset)
        .unwrap_or_else(|| panic!("no self-call after the push in\n{commands}"));
    let pop = commands[call..]
        .find("data remove storage demo:__sand_frames stack[-1]")
        .map(|offset| call + offset)
        .unwrap_or_else(|| panic!("no frame pop after the call in\n{commands}"));
    // The parameter is saved before it is overwritten and restored after.
    assert!(
        commands[push..call].contains("execute store result storage demo:__sand_frames stack[-1].f0 int 1 run scoreboard players get"),
        "{}",
        &commands[push..call]
    );
    assert!(
        commands[call..pop].contains("execute store result score")
            && commands[call..pop].contains("run data get storage demo:__sand_frames stack[-1].f0"),
        "{}",
        &commands[call..pop]
    );
}

#[test]
fn calls_that_cannot_re_enter_their_caller_have_no_frame() {
    let pack = compiled(
        r#"
fn double(n: number) -> number { return n * 2; }
fn quad(n: number) -> number { return double(double(n)); }
"#,
    );
    assert!(!all_commands(&pack).contains("__sand_frames"));
}

#[test]
fn mutual_recursion_saves_each_caller() {
    let pack = compiled(
        r#"
fn is_even(n: number) -> bool {
    if n == 0 { return true; }
    return is_odd(n - 1);
}
fn is_odd(n: number) -> bool {
    if n == 0 { return false; }
    return is_even(n - 1);
}
"#,
    );
    for name in ["is_even", "is_odd"] {
        let body = function(&pack, &format!("demo/function/{name}"));
        let helpers = all_commands(&pack);
        assert!(
            body.contains("__sand_frames") || helpers.contains("__sand_frames"),
            "{name}: {body}"
        );
    }
}

#[test]
fn self_calls_with_swapped_arguments_read_the_old_values() {
    let pack = compiled(
        r#"
fn gcd(a: number, b: number) -> number {
    if b == 0 { return a; }
    return gcd(b, a % b);
}
"#,
    );
    assert!(all_commands(&pack).contains("demo:__sand_frames"));
    let body = body_of(
        r#"
fn gcd(a: number, b: number) -> number {
    if b == 0 { return a; }
    return gcd(b, a % b);
}
"#,
        "gcd",
    );
    // Both arguments are computed into temporaries before either parameter
    // is written, so `b` is not read after `a` was overwritten.
    let push = body
        .iter()
        .position(|op| op["action"]["op"] == "push_frame")
        .unwrap_or_else(|| panic!("{body:#?}"));
    for op in &body[push + 1..push + 3] {
        let from = op["action"]["operand"]["field"]["field"]
            .as_str()
            .or_else(|| op["action"]["operand"]["field"].as_str())
            .unwrap_or_else(|| panic!("{op:#?}"));
        assert!(from.contains("_t"), "parameter read from {from}: {body:#?}");
    }
}

#[test]
fn a_call_in_an_argument_runs_before_any_parameter_is_written() {
    let total = body_of(
        r#"
fn add(a: number, b: number) -> number { return a + b; }
fn total() -> number { return add(1, add(2, 3)); }
"#,
        "total",
    );
    // The inner call happens first; only then is the outer call's first
    // parameter set to 1.
    let calls: Vec<usize> = total
        .iter()
        .enumerate()
        .filter(|(_, op)| op["action"]["op"] == "call")
        .map(|(index, _)| index)
        .collect();
    assert_eq!(calls.len(), 2, "{total:#?}");
    let set_one = total
        .iter()
        .position(|op| op["action"]["op"] == "score_set" && op["action"]["value"] == 1)
        .unwrap_or_else(|| panic!("{total:#?}"));
    assert!(set_one > calls[0] && set_one < calls[1], "{total:#?}");
}

#[test]
fn a_handler_that_re_emits_its_event_saves_the_dispatcher() {
    let pack = compiled(
        r#"
event Countdown(n: number);
#[on(Countdown)]
fn tick_down(n: number) {
    if n > 0 { emit Countdown(n - 1); }
}
"#,
    );
    assert!(all_commands(&pack).contains("demo:__sand_frames"));
}
