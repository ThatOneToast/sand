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
