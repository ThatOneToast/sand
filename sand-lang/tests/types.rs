//! The value types: `number`, `float`, and `bool`.

mod support;

use sand_lang::{FileId, ide};
use support::*;

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

fn pack(body: &str) -> String {
    format!("{PACK}{body}")
}

fn operations(analysis: &sand_lang::Analysis, name: &str) -> Vec<serde_json::Value> {
    let id = format!("demo:{name}");
    let function = analysis.program.as_ref().expect("lowered").modules[0]
        .functions
        .iter()
        .find(|f| f.id.to_string() == id)
        .unwrap();
    serde_json::to_value(&function.body)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|op| op["action"].clone())
        .collect()
}

fn summary(ops: &[serde_json::Value]) -> Vec<String> {
    ops.iter()
        .map(|a| match a["op"].as_str().unwrap() {
            "score_set" => format!("set {}", a["value"]),
            "score_add" => format!("add {}", a["value"]),
            "score_operation" => format!(
                "{} {}",
                a["operator"].as_str().unwrap(),
                a["operand"]
                    .get("field")
                    .map(|f| f.as_str().unwrap().to_string())
                    .unwrap_or_else(|| a["operand"].to_string())
            ),
            other => other.to_string(),
        })
        .collect()
}

#[test]
fn floats_are_fixed_point_and_rescale_multiplication() {
    let analysis = analyze_one(&pack(
        r#"
player state Hero {
    #[clamp(0.0..=2.0)]
    speed: float = 0.5,
    other: float,
}
const BOOST: float = 1.5;
fn f() {
    Hero.other = 2;
    Hero.other = Hero.other * 1.5;
    Hero.other /= 0.5;
    Hero.other = Hero.other * Hero.speed;
    Hero.other += BOOST;
}
"#,
    ));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    let state = &analysis.program.as_ref().unwrap().modules[0].states[0];
    let speed = serde_json::to_value(&state.fields[0]).unwrap();
    assert_eq!(speed["default"], 500);
    assert_eq!(speed["bounds"], serde_json::json!([0, 2000]));
    assert_eq!(
        summary(&operations(&analysis, "f")),
        vec![
            "set 2000", // 2.0
            "mul 3",
            "div 2", // * 1.5, reduced from 1500/1000
            "mul 2", // / 0.5, reduced from 1000/500
            "mul speed",
            "div 1000", // field * field rescales
            "add 1500", // + 1.5
        ]
    );
}

#[test]
fn casts_and_mixing_are_explicit() {
    let analysis = analyze_one(&pack(
        r#"
player state S { n: number, f: float }
fn ok() {
    S.f = S.n as float;
    S.n = S.f as number;
}
fn bad() {
    S.f = S.n;
    S.n = 1.5;
    S.f = S.f + S.n;
    let x: i32 = 1;
}
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "mismatched types: expected `float`, found `number`",
        "mismatched types: expected `number`, found `float`",
        "`i32` is written `number` in Sand",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
    assert_eq!(
        summary(&operations_of_ok(&pack_ok())),
        vec!["set n", "mul 1000", "set f", "div 1000"]
    );
}

fn pack_ok() -> String {
    pack(
        "player state S { n: number, f: float }\nfn ok() {\n    S.f = S.n as float;\n    S.n = S.f as number;\n}\n",
    )
}

fn operations_of_ok(text: &str) -> Vec<serde_json::Value> {
    operations(&analyze_one(text), "ok")
}

#[test]
fn bools_store_flags_and_read_as_conditions() {
    let analysis = analyze_one(&pack(
        r#"
player state Hero { coins: number, rich: bool = true, armed: bool }
fn f() {
    Hero.armed = false;
    Hero.rich = Hero.coins >= 50;
    Hero.armed = !Hero.armed;
    let ok = Hero.rich && Hero.armed;
    if ok {
        Hero.coins += 1;
    }
    if !Hero.rich {
        Hero.coins = 0;
    }
}
"#,
    ));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    let state =
        serde_json::to_value(&analysis.program.as_ref().unwrap().modules[0].states[0]).unwrap();
    assert_eq!(state["fields"][1]["default"], 1);
    let pack = compile(&analysis);
    let all = pack
        .resources
        .values()
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(all.contains("matches 1..1"), "{all}");
}

#[test]
fn type_mistakes_are_explained() {
    let analysis = analyze_one(&pack(
        r#"
player state S { n: number, b: bool, f: float }
fn f() {
    if S.n {}
    S.b += 1;
    S.b = 1;
    if S.b > S.b {}
    chat!("{S.f}");
}
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "expected a `bool` condition, found `number`",
        "`bool` and enum values can only be assigned with `=`",
        "mismatched types: expected `bool`, found `number`",
        "bool values can only be compared with `==` or `!=`",
        "text can only show whole numbers",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
}

#[test]
fn attributes_read_and_write_floats_with_decimals() {
    let analysis = analyze_one(&pack(
        "fn f() {\n    let speed: float = get_attribute(Attribute::MovementSpeed);\n    set_attribute(Attribute::MovementSpeed, speed * 1.5);\n}\n",
    ));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    let ops = operations(&analysis, "f");
    assert_eq!(ops[0]["scale"], 1000.0);
    let set = ops.iter().find(|o| o["op"] == "attribute_set").unwrap();
    assert_eq!(set["value"]["scale"], 1000.0);
}

#[test]
fn hover_shows_the_friendly_values() {
    let text = pack("player state Hero { speed: float = 0.5, rich: bool = true }\n");
    let analysis = analyze_one(&text);
    let hover = ide::hover(&analysis, FileId(0), text.find("Hero").unwrap()).unwrap();
    assert!(
        hover.markdown.contains("speed: float = 0.5"),
        "{}",
        hover.markdown
    );
    assert!(
        hover.markdown.contains("rich: bool = true"),
        "{}",
        hover.markdown
    );
}
