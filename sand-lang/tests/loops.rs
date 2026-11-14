//! `while`, `loop`, counting `for`, `break`, and `continue`.

mod support;

use support::*;

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

fn pack(body: &str) -> String {
    format!("{PACK}{body}")
}

#[test]
fn every_loop_form_compiles() {
    let analysis = analyze_one(&pack(
        r#"
fn sum_to(n: number) -> number {
    let mut total = 0;
    for i in 1..=n { total += i; }
    return total;
}
fn first_over(limit: number) -> number {
    let mut found = 0;
    for i in 0..100 {
        if i * i > limit { found = i; break; }
    }
    return found;
}
fn odds(n: number) -> number {
    let mut total = 0;
    for i in 0..n {
        if i % 2 == 0 { continue; }
        total += i;
    }
    return total;
}
fn steps(start: number) -> number {
    let mut n = start;
    let mut count = 0;
    while n != 1 && count < 1000 {
        if n % 2 == 0 { n = n / 2; } else { n = n * 3 + 1; }
        count += 1;
    }
    return count;
}
fn passes() -> number {
    let mut n = 0;
    loop {
        n += 1;
        if n >= 3 { break; }
    }
    return n;
}
"#,
    ));
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));
    assert!(warnings(&analysis).is_empty(), "{:#?}", warnings(&analysis));
    compile(&analysis);
}

#[test]
fn a_simple_counting_loop_tests_its_counter_directly() {
    let analysis = analyze_one(&pack(
        r#"
global state Totals { sum: number }
fn add_up() {
    for i in 0..5 { Totals.sum += i; }
}
"#,
    ));
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));
    let pack = compile(&analysis);
    let add_up = function(&pack, "demo/function/add_up");
    // The counter starts at 0 and the loop runs while it is at most 4.
    assert!(
        add_up.contains("matches -2147483648..4 run function demo:__sand_program/"),
        "{add_up}"
    );
}

#[test]
fn loop_mistakes_are_explained() {
    let analysis = analyze_one(&pack(
        r#"
fn stray() { break; }
fn stray_continue() { continue; }
fn every_player() {
    for player in players() { break; }
}
fn open_range() {
    for i in 0.. { let _x = i; }
}
fn decimals() {
    for i in 0.0..2.5 { let _x = i; }
}
fn forever() {
    let mut n = 0;
    loop { n += 1; }
}
fn counter_is_fixed() {
    for i in 0..3 { i = 2; }
}
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "`break` only works inside a loop",
        "`continue` only works inside a loop",
        "`break` cannot stop a `players()` or `entities(..)` loop",
        "a counting `for` needs both ends",
        "a counting `for` counts `number`s, but this is `float`",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
    assert!(
        errors.iter().any(|e| e.contains("`i`")),
        "assigning the counter is rejected: {errors:#?}"
    );
    assert!(warnings(&analysis).contains(&"this `loop` never ends".to_string()));
}

#[test]
fn continue_in_a_while_inside_a_player_loop_targets_the_while() {
    let analysis = analyze_one(&pack(
        r#"
player state Score { points: number }
fn tally() {
    for player in players() {
        let mut n = 0;
        while n < 3 {
            n += 1;
            if n == 2 { continue; }
            Score.points += 1;
        }
    }
}
"#,
    ));
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));
    let program = serde_json::to_string(analysis.program.as_ref().unwrap()).unwrap();
    // `continue` sets the jump flag instead of ending the player's turn.
    assert!(!program.contains("\"op\":\"return\""), "{program}");
}
