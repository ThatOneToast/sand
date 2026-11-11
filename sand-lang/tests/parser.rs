//! Grammar coverage, error recovery, and robustness against partial input.

mod support;

use sand_lang::ast::{ItemKind, Stmt};
use sand_lang::parser::parse;
use support::*;

#[test]
fn keywords_only_match_whole_words() {
    let (file, errors) = parse(
        "fn f() {\n    format();\n    iffy();\n    forge();\n    raw_data();\n    as_is();\n    index.inner = 1;\n}\n",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let ItemKind::Fn(f) = &file.items[0].kind else {
        panic!()
    };
    let calls: Vec<_> = f
        .body
        .stmts
        .iter()
        .filter_map(|s| match s {
            Stmt::Call { path, .. } => Some(path.display()),
            _ => None,
        })
        .collect();
    assert_eq!(calls, vec!["format", "iffy", "forge", "raw_data", "as_is"]);
    assert!(matches!(f.body.stmts[5], Stmt::Assign { .. }));
}

#[test]
fn comments_docs_and_strings() {
    let (file, errors) = parse(
        r#"//! Pack docs.
/* outer /* nested */ still comment */
//// four slashes is an ordinary comment
/// Item docs.
/// Second line.
fn f() {
    raw!("say \"quoted\" \\ done");
}
"#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(file.docs, vec!["Pack docs."]);
    assert_eq!(file.items[0].docs, vec!["Item docs.", "Second line."]);
    let ItemKind::Fn(f) = &file.items[0].kind else {
        panic!()
    };
    let Stmt::Macro { name, args, .. } = &f.body.stmts[0] else {
        panic!()
    };
    assert_eq!(name.name, "raw");
    let sand_lang::ast::ArgValue::Expr(sand_lang::ast::Expr::Str { value, .. }) = &args[0].value
    else {
        panic!()
    };
    assert_eq!(value, r#"say "quoted" \ done"#);
}

#[test]
fn malformed_items_are_isolated() {
    let (file, errors) = parse("fn a() {}\nconst = 3;\nfn b() {}\n");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "expected identifier; found `=`");
    let names: Vec<_> = file
        .items
        .iter()
        .map(|item| item.name().map(|n| n.name.clone()))
        .collect();
    assert_eq!(names, vec![Some("a".into()), None, Some("b".into())]);
}

#[test]
fn unterminated_input_reports_without_panicking() {
    for text in [
        "/* never closed",
        "fn f() {",
        "player state S {",
        "\"",
        "#![pack(",
    ] {
        let (_, errors) = parse(text);
        assert!(!errors.is_empty(), "{text:?} should report an error");
    }
    let (file, errors) = parse("");
    assert!(file.items.is_empty() && errors.is_empty());
}

#[test]
fn every_prefix_of_a_real_pack_analyses_without_panicking() {
    let root = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/lang_encounters/pack.sand"
    ))
    .unwrap();
    let rewards = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/lang_encounters/rewards.sand"
    ))
    .unwrap();
    // Editors analyse every intermediate state while an author types.
    for end in (0..=root.len()).filter(|i| root.is_char_boundary(*i)) {
        let analysis = analyze(&[("pack.sand", &root[..end]), ("rewards.sand", &rewards)]);
        for offset in [0, end / 2, end] {
            let _ = sand_lang::ide::completions(&analysis, sand_lang::FileId(0), offset);
            let _ = sand_lang::ide::hover(&analysis, sand_lang::FileId(0), offset);
        }
    }
}

#[test]
fn the_shipped_example_compiles() {
    let root = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/lang_encounters/pack.sand"
    ));
    let analysis = sand_lang::analyze_path(root, &sand_lang::DiskFiles);
    let pack = compile(&analysis);
    assert!(
        pack.resources
            .contains_key("data/demo/function/rewards/grant.mcfunction")
    );
    assert!(
        pack.resources
            .contains_key("data/demo/function/encounter.flow/tick_active.mcfunction")
    );
}
