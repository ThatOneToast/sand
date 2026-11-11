//! Observable semantics: emitted commands for language constructs, and the
//! diagnostics authors see for mistakes.

mod support;

use support::*;

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

fn pack(body: &str) -> String {
    format!("{PACK}{body}")
}

// ── Lowering ────────────────────────────────────────────────────────────────

#[test]
fn conditions_lower_to_single_evaluation_branches() {
    let analysis = analyze_one(&pack(
        r#"
global state World { ticks: number }
#[tick]
fn clock() {
    if World.ticks >= 200 && World.ticks != 999 {
        World.ticks = 0;
    } else if World.ticks in 100..110 {
        raw!("say halfway");
    }
}
"#,
    ));
    let program = analysis.program.as_ref().expect("lowered");
    let body = serde_json::to_value(&program.modules[0].functions[0].body).unwrap();
    // `a && b` nests b inside a; `!=` swaps branches; `..` excludes its end.
    let outer = &body[0]["action"];
    assert_eq!(outer["op"], "branch");
    assert_eq!(outer["condition"]["min"], 200);
    assert!(outer["condition"]["max"].is_null());
    let inner = &outer["then"][0]["action"];
    assert_eq!(
        (
            inner["condition"]["min"].clone(),
            inner["condition"]["max"].clone()
        ),
        (999.into(), 999.into())
    );
    assert_eq!(inner["otherwise"][0]["action"]["op"], "score_set");
    assert_eq!(inner["then"][0]["action"]["condition"]["max"], 109);
    assert_eq!(outer["otherwise"][0]["action"]["condition"]["min"], 100);
    // Raw commands opt into the portable capability automatically.
    assert_eq!(program.requires, vec!["raw_commands".to_string()]);
    compile(&analysis);
}

#[test]
fn players_loops_run_bodies_as_each_player() {
    let analysis = analyze_one(&pack(
        r#"
player state Hero { gold: number }
#[tick]
fn payday() {
    for player in players() {
        Hero.gold += 5;
    }
}
"#,
    ));
    let pack = compile(&analysis);
    let payday = function(&pack, "demo/function/payday");
    assert!(payday.starts_with("execute as @a run function"), "{payday}");
}

#[test]
fn constants_enums_and_casts_fold_at_compile_time() {
    let analysis = analyze_one(&pack(
        r#"
const BASE: number = 40;
const MAX: number = BASE * 2 + Phase::Done as number;
enum Phase { Idle, Busy = 7, Done }
player state Hero {
    phase: Phase,
    #[clamp(0..=MAX)]
    energy: number = MAX,
}
fn reset() {
    Hero.phase = Phase::Busy;
    Hero.energy = MAX - 1;
}
"#,
    ));
    let program = analysis.program.as_ref().expect("lowered");
    let state = &program.modules[0].states[0];
    assert_eq!(
        state.fields[0].default, 0,
        "enum fields default to the first variant"
    );
    assert_eq!(state.fields[1].default, 88);
    assert_eq!(state.fields[1].bounds, Some((0, 88)));
    let body = serde_json::to_value(&program.modules[0].functions[0].body).unwrap();
    assert_eq!(body[0]["action"]["value"], 7);
    assert_eq!(body[1]["action"]["value"], 87);
}

#[test]
fn flow_hooks_accept_bare_variants_and_inline_bodies() {
    let analysis = analyze_one(&pack(
        r#"
enum Phase { Idle, Fight }
player state Hero { phase: Phase, rage: number }
flow Combat on Hero.phase {
    Idle -> Fight if Hero.rage >= 10;
    enter Fight { Hero.rage = 0; }
    tick Fight every 20 { Hero.rage += 1; }
}
"#,
    ));
    let pack = compile(&analysis);
    let enter = function(&pack, "demo/function/combat.flow/enter_fight");
    assert!(enter.contains("scoreboard players set @s"), "{enter}");
    let program = analysis.program.as_ref().unwrap();
    let flow = &program.modules[0].flows[0];
    assert_eq!((flow.transitions[0].from, flow.transitions[0].to), (0, 1));
    assert_eq!(flow.hooks[1].cadence, 20);
}

#[test]
fn modules_use_paths_and_identities_follow_the_module_tree() {
    let analysis = analyze(&[
        (
            "pack.sand",
            &pack(
                r#"
mod combat;
use combat::hits::strike;
#[tick]
fn tick() { for player in players() { strike(); } }
"#,
            ),
        ),
        ("combat.sand", "pub mod hits;\n"),
        (
            "combat/hits.sand",
            "use crate::Rage;\npub fn strike() { Rage.value += 1; }\n",
        ),
    ]);
    // `Rage` is not declared; the import is reported at its path.
    assert!(
        errors(&analysis)[0].contains("cannot find `Rage`"),
        "{:?}",
        errors(&analysis)
    );

    let analysis = analyze(&[
        (
            "pack.sand",
            &pack(
                r#"
mod combat;
use combat::hits::strike;
player state Rage { value: number }
#[tick]
fn tick() { for player in players() { strike(); } }
"#,
            ),
        ),
        ("combat/mod.sand", "pub mod hits;\n"),
        (
            "combat/hits.sand",
            "use crate::Rage;\npub fn strike() { Rage.value += 1; }\n",
        ),
    ]);
    let pack = compile(&analysis);
    let tick = function(&pack, "demo/function/tick");
    assert!(tick.contains("function demo:combat/hits/strike"), "{tick}");
}

#[test]
fn inline_modules_share_the_file_and_privacy_rules() {
    let analysis = analyze_one(&pack(
        r#"
mod shop {
    pub fn buy() {}
    fn audit() {}
}
fn checkout() {
    shop::buy();
    shop::audit();
}
"#,
    ));
    assert_eq!(errors(&analysis), vec!["`audit` is private".to_string()]);
}

// ── Diagnostics ─────────────────────────────────────────────────────────────

#[test]
fn entry_points_that_need_a_player_explain_why() {
    let analysis = analyze_one(&pack(
        r#"
player state Hero { energy: number }
#[tick]
fn tick() { regen(); }
fn regen() { Hero.energy += 1; }
"#,
    ));
    let errors = errors(&analysis);
    assert_eq!(
        errors,
        vec![
            "`tick` runs from #[tick] without a player, but it calls `regen`, which needs a player"
        ]
    );
    assert_eq!(spanned(&analysis, "without a player"), "tick");
    let diagnostic = &analysis.diagnostics[0];
    assert_eq!(diagnostic.notes.len(), 2);
    assert!(
        diagnostic
            .help
            .as_deref()
            .unwrap()
            .contains("for player in players()")
    );
}

#[test]
fn global_flow_hooks_cannot_touch_player_state() {
    let analysis = analyze_one(&pack(
        r#"
player state Hero { energy: number }
global state World { phase: number }
fn heal() { Hero.energy += 1; }
flow Day on World.phase {
    0 -> 1;
    enter 1 => heal;
    tick 1 { Hero.energy -= 1; }
}
"#,
    ));
    let errors = errors(&analysis);
    assert!(
        errors.iter().any(|e| e.contains("`heal` needs a player")),
        "{errors:?}"
    );
    assert!(
        errors.iter().any(|e| e
            .contains("global flow hook has no player, but it writes player state `Hero.energy`")),
        "{errors:?}"
    );
}

#[test]
fn type_errors_point_at_the_expression() {
    let analysis = analyze_one(&pack(
        r#"
enum Phase { A, B }
player state Hero { phase: Phase, hp: number }
fn f() {
    Hero.phase = 1;
    Hero.hp = Phase::B;
    Hero.phase += Phase::A;
    if Hero.phase > Phase::A {}
}
"#,
    ));
    assert_eq!(
        errors(&analysis),
        vec![
            "mismatched types: expected `Phase`, found `number`",
            "mismatched types: expected `number`, found `Phase`",
            "`bool` and enum values can only be assigned with `=`",
            "enum values can only be compared with `==` or `!=`",
        ]
    );
    assert_eq!(spanned(&analysis, "found `number`"), "1");
    assert_eq!(spanned(&analysis, "found `Phase`"), "Phase::B");
}

#[test]
fn resolution_errors_suggest_fixes() {
    let analysis = analyze(&[
        (
            "pack.sand",
            &pack(
                "mod util;\nplayer state Hero { hp: number }\nfn f() { Hero.mana = 1; util::nope(); }\n",
            ),
        ),
        ("util.sand", "fn g() { Hero.hp = 1; }\n"),
    ]);
    let rendered: Vec<_> = analysis
        .diagnostics
        .iter()
        .map(|d| (d.message.clone(), d.help.clone()))
        .collect();
    assert!(rendered.contains(&(
        "state `Hero` has no field `mana`".into(),
        Some("available fields: `hp`".into())
    )));
    assert!(
        rendered
            .iter()
            .any(|(m, _)| m == "cannot find `nope` in `crate::util`")
    );
    assert!(rendered.iter().any(|(m, _)| m
        == "cannot find `Hero` in this scope; it is defined at the crate root, so import it with `use crate::Hero;`"));
}

#[test]
fn declaration_errors() {
    let analysis = analyze_one(&pack(
        r#"
const A: number = B;
const B: number = A;
const BIG: number = 3000000000;
enum Twice { X = 1, Y = 1 }
fn dup() {}
fn dup() {}
player state S {
    #[clamp(10..=0)]
    v: number,
}
mod missing;
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "cycle detected while evaluating `A`",
        "3000000000 does not fit in an i32 score",
        "discriminant 1 is already used",
        "the name `dup` is defined multiple times",
        "clamp range is empty",
        "file not found for module `missing`",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
}

#[test]
fn conditions_that_cannot_lower_are_rejected() {
    let analysis = analyze_one(&pack(
        r#"
player state Hero { a: number, b: number }
enum P { X, Y }
player state Mode { p: P }
flow M on Mode.p {
    X -> Y if Hero.a > 1 && Hero.b > 1;
}
fn f() {
    if Hero.a < -2147483648 {}
    if Mode.p == Hero.a {}
}
"#,
    ));
    let errors = errors(&analysis);
    assert!(
        errors
            .contains(&"a transition guard must be a single comparison or `in` range".to_string())
    );
    assert!(errors.contains(&"mismatched types: expected `P`, found `number`".to_string()));
    assert!(errors.contains(&"this condition can never be true".to_string()));
}

#[test]
fn clamped_constant_writes_warn() {
    let analysis = analyze_one(&pack(
        "player state H {\n    #[clamp(0..=10)]\n    v: number = 20,\n}\nfn f() { H.v = 50; }\n",
    ));
    assert!(!analysis.has_errors());
    assert_eq!(warnings(&analysis).len(), 2, "{:?}", warnings(&analysis));
}

#[test]
fn pack_metadata_is_required_and_validated() {
    let analysis = analyze_one("fn f() {}\n");
    assert_eq!(
        errors(&analysis),
        vec!["the package root must declare a namespace"]
    );

    let analysis = analyze_one("#![pack(namespace = \"Bad NS\", colour = \"red\")]\n");
    let errors = errors(&analysis);
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("unknown pack setting `colour`"))
    );
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("namespaces may only contain"))
    );

    // The description defaults to the root file's inner docs.
    let analysis = analyze_one("//! Arena minigame.\n#![pack(namespace = \"arena\")]\n");
    assert_eq!(analysis.pack.unwrap().description, "Arena minigame.");
}

#[test]
fn backend_diagnostics_map_to_the_originating_statement() {
    let analysis = analyze_one(&pack("fn f() {\n    raw!(\"say one\\nsay two\");\n}\n"));
    assert!(!analysis.has_errors());
    let errors = analysis
        .compile()
        .expect_err("multi-line raw commands are rejected");
    let error = &errors[0];
    assert!(error.code.starts_with("backend/"), "{error:?}");
    let text = &analysis.package.files[error.file.0].text[error.span.start..error.span.end];
    assert_eq!(text, "raw!(\"say one\\nsay two\");");
}
