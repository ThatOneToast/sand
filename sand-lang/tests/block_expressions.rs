mod support;

use sand_lang::{FileId, ide};
use support::*;

#[test]
fn scoped_value_blocks_export_through_the_normal_expression_path() {
    compile(&analyze_one(include_str!(
        "fixtures/block_expressions.sand"
    )));
}

#[test]
fn value_block_type_inference_reuses_the_authored_local_definition() {
    let source = r#"#![pack(namespace = "blocks")]
fn message() -> string {
    let result = { let inner = "note"; inner };
    return result;
}
"#;
    let analysis = analyze_one(source);
    compile(&analysis);
    let tail = source.find("inner }").unwrap();
    let declared = source.find("inner =").unwrap();
    assert_eq!(
        ide::definition(&analysis, FileId(0), tail).unwrap().1.start,
        declared
    );
}

#[test]
fn value_blocks_require_a_tail_and_keep_control_transfers_scoped() {
    for (source, diagnostic) in [
        ("fn f() { let lost = { let n = 1; }; }", None),
        (
            "fn f() -> number { return { return 4; 5 }; }",
            Some("`return` cannot leave a value block"),
        ),
        (
            "fn f() { for n in 0..3 { let x = { continue; n }; } }",
            Some("a loop jump cannot leave a value block"),
        ),
        (
            "fn f() -> string { let output = { let inner = \"note\"; inner }; return inner; }",
            Some("cannot find `inner` in this scope"),
        ),
    ] {
        let analysis = analyze_one(&format!(r#"#![pack(namespace = "blocks")] {source}"#));
        assert!(analysis.has_errors(), "{source}");
        if let Some(expected) = diagnostic {
            let errors = errors(&analysis);
            assert!(
                errors.iter().any(|error| error.contains(expected)),
                "{errors:#?}"
            );
        }
    }
}
