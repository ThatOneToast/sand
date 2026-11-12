//! `value as T` between `number`, `float`, `bool`, and enums.

mod support;

use support::*;

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

fn pack(body: &str) -> String {
    format!("{PACK}{body}")
}

#[test]
fn constant_casts_fold_with_the_documented_rules() {
    let analysis = analyze_one(&pack(
        r#"
enum Phase { Idle, Busy, Done }
const WHOLE: number = 2.75 as number;
const BELOW: number = -1.5 as number;
const DECIMAL: float = 3 as float;
const ON: bool = 5 as bool;
const OFF: bool = 0 as bool;
const ONE: number = true as number;
const LAST: number = Phase::Done as number;
const PICKED: Phase = 1 as Phase;
"#,
    ));
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));
    let hover = |name: &str| {
        let text = &analysis.package.files[0].text;
        sand_lang::ide::hover(&analysis, sand_lang::FileId(0), text.find(name).unwrap())
            .unwrap()
            .markdown
    };
    assert!(hover("WHOLE").contains("= 2"), "{}", hover("WHOLE"));
    // Rounds down, like the scoreboard division used at run time.
    assert!(hover("BELOW").contains("= -2"), "{}", hover("BELOW"));
    assert!(hover("DECIMAL").contains("= 3"), "{}", hover("DECIMAL"));
    assert!(hover("ON").contains("= true"), "{}", hover("ON"));
    assert!(hover("OFF").contains("= false"), "{}", hover("OFF"));
    assert!(hover("ONE").contains("= 1"), "{}", hover("ONE"));
    assert!(hover("LAST").contains("= 2"), "{}", hover("LAST"));
    assert!(hover("PICKED").contains("Busy"), "{}", hover("PICKED"));
}

#[test]
fn runtime_casts_compile_to_score_operations() {
    let analysis = analyze_one(&pack(
        r#"
enum Phase { Idle, Busy }
player state S { n: number, f: float, b: bool, p: Phase }
fn convert() {
    S.f = S.n as float;
    S.n = S.f as number;
    S.b = S.n as bool;
    S.n = S.b as number;
    S.p = S.n as Phase;
    S.n = S.p as number;
}
"#,
    ));
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));
    let program = serde_json::to_value(analysis.program.as_ref().unwrap()).unwrap();
    let body = program["modules"][0]["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == "demo:convert")
        .unwrap()["body"]
        .clone();
    let text = body.to_string();
    // `as bool` keeps zero and turns anything else into one.
    assert!(text.contains("\"max\":0,\"min\":0"), "{text}");
    compile(&analysis);
}

#[test]
fn impossible_casts_explain_the_alternative() {
    let analysis = analyze_one(&pack(
        r#"
enum Phase { Idle, Busy }
enum Mode { Off, On }
const A: bool = Phase::Busy as bool;
const B: Phase = true as Phase;
const C: Phase = 1.5 as Phase;
const D: Mode = Phase::Busy as Mode;
const E: Phase = 7 as Phase;
const F: number = 1 as text;
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "cannot cast `Phase` to `bool`",
        "cannot cast `bool` to `Phase`",
        "cannot cast `float` to `Phase`",
        "cannot cast `Phase` to `Mode`",
        "no `Phase` variant has the value 7",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
    assert!(
        errors.iter().any(|e| e.contains("text")),
        "an unknown cast type is reported: {errors:#?}"
    );
}
