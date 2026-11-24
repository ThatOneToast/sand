mod support;
use support::*;

#[test]
fn runtime_text_fixture_compiles() {
    compile(&analyze_one(include_str!("fixtures/text.sand")));
}

#[test]
fn text_rejects_numeric_operations_and_constant_cycles() {
    for code in [
        "fn f() { let s: string = 12; }",
        "fn f() { let s = \"a\" - \"b\"; }",
        "fn f() { let mut s = \"a\"; s *= \"b\"; }",
        "fn f() { let s = \"a\" as number; }",
        "fn f() { let s = \"a\"; let part = s.slice(0..2, 3); }",
        "fn f() { if \"a\" < \"b\" { } }",
        "const A: string = B; const B: string = A;",
        "global state S { #[clamp(0..=5)] name: string }",
    ] {
        assert!(
            analyze_one(&format!("#![pack(namespace = \"text\")] {code}")).has_errors(),
            "accepted {code}"
        );
    }
}

#[test]
fn strings_decode_escapes_and_reject_malformed_unicode() {
    let source =
        r#"#![pack(namespace = "text")] global state S { value: string = "\0\r\n\t\u{1f30d}" }"#;
    let pack = compile(&analyze_one(source));
    let commands: Vec<_> = pack
        .resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, bytes)| std::str::from_utf8(bytes).unwrap())
        .collect();
    assert!(
        commands
            .iter()
            .any(|commands| commands.contains(r#"["\u0000","\u000d","\u000a","\u0009","🌍"]"#))
    );
    for escape in [
        r"\q",
        r"\u{}",
        r"\u{d800}",
        r"\u{110000}",
        r"\u{0000000}",
        r"\u{xyz}",
    ] {
        let source =
            format!(r#"#![pack(namespace = "text")] fn f() {{ let value = "{escape}"; }}"#);
        assert!(analyze_one(&source).has_errors(), "accepted {escape}");
    }
}

#[test]
fn grouped_receivers_keep_postfixes_in_conditions() {
    compile(&analyze_one(
        r#"#![pack(namespace = "text")]
fn f(n: number) -> bool { return (n + 1).to_string() == "3"; }
fn g(s: string) -> bool { return (s).is_empty() || (s + "a").len() == 2; }
"#,
    ));
}

#[test]
fn string_constant_expansion_has_a_checked_byte_budget() {
    let half = "x".repeat(32768);
    let source = format!(
        r#"#![pack(namespace = "limits")]
const HALF: string = "{half}";
const WHOLE: string = HALF + HALF;
const TOO_BIG: string = WHOLE + HALF;
"#
    );
    assert!(
        errors(&analyze_one(&source))
            .iter()
            .any(|message| message.contains("65536 UTF-8 bytes"))
    );
    let literal = "x".repeat(65537);
    let source =
        format!(r#"#![pack(namespace = "limits")] fn run() {{ let value = "{literal}"; }}"#);
    assert!(
        errors(&analyze_one(&source))
            .iter()
            .any(|message| message.contains("65536 UTF-8 bytes"))
    );
}

#[test]
fn translations_preserve_substitution_order_styles_and_runtime_string_paths() {
    let analysis = analyze_one(
        r#"#![pack(namespace = "translated", minecraft = "26.2")]
global state S { score: number = 5 }
item Notice { base: Item::Paper, name: Text::translate("item.minecraft.paper") }
fn run() {
    let mut label = "Alice";
    let mut other = "Bob";
    chat!(Text::translate("chat.type.text", label, Text::translate("demo.points", "{gold}Points: {S.score}", other)));
    give_item(Notice);
    label = other;
    other = label;
    chat!(label + other);
}
"#,
    );
    let pack = compile(&analysis);
    let run = function(&pack, "translated/function/run");
    let line = run
        .lines()
        .find(|line| line.starts_with("tellraw @s "))
        .unwrap();
    let message: serde_json::Value =
        serde_json::from_str(line.strip_prefix("tellraw @s ").unwrap()).unwrap();
    let translation = &message["extra"][0];
    assert_eq!(translation["translate"], "chat.type.text");
    let args = &translation["with"];
    assert_eq!(args[0]["extra"][0]["nbt"], "p0_0_0[]");
    assert_eq!(args[0]["extra"][0]["interpret"], false);
    let inner = &args[1]["extra"][0];
    assert_eq!(inner["translate"], "demo.points");
    assert_eq!(inner["with"][0]["extra"][0]["color"], "gold");
    assert_eq!(inner["with"][0]["extra"][0]["text"], "Points: ");
    assert!(inner["with"][0]["extra"][1].get("score").is_some());
    assert_eq!(inner["with"][1]["extra"][0]["nbt"], "p0_1_0_1_0[]");
    assert!(run.contains("item.minecraft.paper"));
    for path in ["p0_0_0", "p0_1_0_1_0"] {
        let remove = format!("data remove storage translated:__program_text {path}");
        let (before, after) = run.split_once(line).unwrap();
        assert!(before.contains(&remove) && after.contains(&remove), "{run}");
    }
}

#[test]
fn translations_check_keys_execution_context_and_nesting() {
    let cases = [
        ("fn f() { chat!(Text::translate(\"\")); }", "nonblank"),
        (
            "fn f() { chat!(Text::translate()); }",
            "needs a quoted translation key",
        ),
        (
            "fn f() { chat!(Text::translate(key = \"demo.x\")); }",
            "arguments are positional",
        ),
        (
            "item Bad { base: Item::Paper, name: Text::translate(\"demo.x\", \"{player}\") }",
            "resource text cannot read runtime values",
        ),
    ];
    for (source, expected) in cases {
        let result = analyze_one(&format!("#![pack(namespace = \"translated\")] {source}"));
        assert!(
            errors(&result)
                .iter()
                .any(|message| message.contains(expected)),
            "{:#?}",
            errors(&result)
        );
    }
    let mut message = "\"leaf\"".to_owned();
    for _ in 0..=sand::advanced::compiler::TextPart::MAX_NESTING_DEPTH {
        message = format!("Text::translate(\"demo.key\", {message})");
    }
    let too_deep = analyze_one(&format!(
        "#![pack(namespace = \"translated\")] fn f() {{ broadcast!({message}); }}"
    ));
    assert!(
        errors(&too_deep)
            .iter()
            .any(|message| message.contains("component nesting limit"))
    );
}

#[test]
fn rich_chat_exports_native_hover_and_click_actions() {
    let analysis = analyze_one(
        r#"#![pack(namespace = "interactive", minecraft = "26.2")]
fn message() {
    chat!(Text::hover(Text::link("{aqua}Guide", "https://example.com/guide"), "Open the field guide"));
    chat!(Text::suggest("Next", "/trigger next"));
    chat!(Text::copy("Copy", "saved-code"));
}
"#,
    );
    let pack = compile(&analysis);
    let commands = function(&pack, "interactive/function/message");
    for expected in [
        r#""action":"show_text""#,
        r#""text":"Open the field guide""#,
        r#""action":"open_url""#,
        r#""url":"https://example.com/guide""#,
        r#""action":"suggest_command""#,
        r#""command":"/trigger next""#,
        r#""action":"copy_to_clipboard""#,
        r#""value":"saved-code""#,
    ] {
        assert!(
            commands.contains(expected),
            "missing {expected}: {commands}"
        );
    }
}

#[test]
fn rich_text_validates_click_payloads_and_display_channels() {
    for (code, expected) in [
        (
            r#"fn f() { chat!(Text::link("open", "file:///local")); }"#,
            "HTTP(S) URI",
        ),
        (
            r#"fn f() { chat!(Text::suggest("next", "/trigger\nnext")); }"#,
            "without control",
        ),
        (
            r#"fn f() { title!(Text::copy("copy", "code")); }"#,
            "supported in chat",
        ),
        (
            r#"item Bad { base: Item::Paper, name: Text::hover("name", "tooltip") }"#,
            "supported in chat",
        ),
        (
            r#"fn f() { chat!(Text::hover("label")); }"#,
            "takes a label and one other argument",
        ),
    ] {
        let analysis = analyze_one(&format!(r#"#![pack(namespace = "interactive")] {code}"#));
        let errors = errors(&analysis);
        assert!(
            errors.iter().any(|error| error.contains(expected)),
            "{errors:#?}"
        );
    }
}
