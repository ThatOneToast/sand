//! Editor queries: hover, navigation, rename, completion, outline, hints,
//! and parser recovery that keeps them working while code is incomplete.

mod support;

use sand_lang::ide::{self, CompletionKind};
use sand_lang::{Analysis, DefKind, FileId};
use support::*;

const ROOT_TEXT: &str = r#"//! Arena.
#![pack(namespace = "arena", minecraft = "26.2")]

mod rewards;

use rewards::grant;

/// Energy cap.
const MAX: number = 100;

/// Where a player is.
enum Phase { Idle, Fight }

/// Per-player progress.
player state Hero {
    phase: Phase,
    /// Spendable energy.
    #[clamp(0..=MAX)]
    energy: number = MAX,
}

#[tick]
fn tick() {
    for player in players() {
        grant();
    }
}
"#;

const REWARDS_TEXT: &str = r#"use crate::Hero;

/// Give back some energy.
pub fn grant() {
    Hero.energy += 5;
}
"#;

fn workspace() -> Analysis {
    analyze(&[("pack.sand", ROOT_TEXT), ("rewards.sand", REWARDS_TEXT)])
}

fn offset(text: &str, needle: &str, nth: usize) -> usize {
    text.match_indices(needle)
        .nth(nth)
        .unwrap_or_else(|| panic!("{needle:?} #{nth} not found"))
        .0
}

const ROOT: FileId = FileId(0);
const REWARDS: FileId = FileId(1);

#[test]
fn hover_shows_signature_inferred_context_identity_and_docs() {
    let analysis = workspace();
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));

    let at = offset(REWARDS_TEXT, "grant", 0);
    let hover = ide::hover(&analysis, REWARDS, at).unwrap();
    assert!(
        hover.markdown.contains("pub fn grant()"),
        "{}",
        hover.markdown
    );
    assert!(hover.markdown.contains("Function `arena:rewards/grant`"));
    assert!(
        hover
            .markdown
            .contains("Runs as a **player** because it writes player state `Hero.energy`"),
        "{}",
        hover.markdown
    );
    assert!(hover.markdown.contains("Give back some energy."));

    let field = ide::hover(&analysis, REWARDS, offset(REWARDS_TEXT, "energy +=", 0)).unwrap();
    assert!(
        field.markdown.contains("energy: number = 100"),
        "{}",
        field.markdown
    );
    assert!(field.markdown.contains("clamped to `0..=100`"));
    assert!(field.markdown.contains("Spendable energy."));

    let tick = ide::hover(&analysis, ROOT, offset(ROOT_TEXT, "fn tick", 0) + 3).unwrap();
    assert!(
        tick.markdown.contains("Runs on the **server**"),
        "{}",
        tick.markdown
    );
    assert!(tick.markdown.contains("Entry point: #[tick]"));
}

#[test]
fn definition_crosses_files_and_modules_open_their_file() {
    let analysis = workspace();
    let call = offset(ROOT_TEXT, "grant();", 0);
    let (file, span) = ide::definition(&analysis, ROOT, call).unwrap();
    assert_eq!(file, REWARDS);
    assert_eq!(&REWARDS_TEXT[span.start..span.end], "grant");

    let module = offset(ROOT_TEXT, "rewards::grant", 0);
    assert_eq!(
        ide::definition(&analysis, ROOT, module),
        Some((REWARDS, sand_lang::ast::Span::new(0, 0)))
    );

    let state = offset(REWARDS_TEXT, "Hero.energy", 0);
    let (file, span) = ide::definition(&analysis, REWARDS, state).unwrap();
    assert_eq!((file, &ROOT_TEXT[span.start..span.end]), (ROOT, "Hero"));
}

#[test]
fn references_and_rename_cover_every_use() {
    let analysis = workspace();
    let at = offset(ROOT_TEXT, "MAX", 0);
    let refs = ide::references(&analysis, ROOT, at, true);
    assert_eq!(refs.len(), 3, "declaration, clamp bound, and default");

    let edits = ide::rename(
        &analysis,
        REWARDS,
        offset(REWARDS_TEXT, "Hero", 1),
        "Champion",
    )
    .unwrap();
    let files: Vec<_> = edits.iter().map(|e| e.file).collect();
    assert_eq!(edits.len(), 3, "{edits:?}");
    assert!(files.contains(&ROOT) && files.contains(&REWARDS));

    assert!(ide::rename(&analysis, ROOT, at, "fn").is_err());
    assert!(
        ide::prepare_rename(&analysis, ROOT, offset(ROOT_TEXT, "rewards", 0))
            .unwrap_err()
            .contains("rename the file")
    );
}

#[test]
fn completion_is_context_aware() {
    let analysis = workspace();
    let labels =
        |items: Vec<ide::Completion>| items.into_iter().map(|c| c.label).collect::<Vec<_>>();

    // Fields after `State.`
    let after_dot = offset(REWARDS_TEXT, "energy +=", 0);
    let fields = labels(ide::completions(&analysis, REWARDS, after_dot));
    assert_eq!(fields, vec!["phase", "energy"]);

    // Module members after `path::`, honouring privacy.
    let after_path = offset(ROOT_TEXT, "grant;", 0);
    let members = labels(ide::completions(&analysis, ROOT, after_path));
    assert_eq!(members, vec!["grant"]);

    // Statement position: items, statements, and builtins, but not values.
    let in_body = offset(ROOT_TEXT, "grant();", 0);
    let items = ide::completions(&analysis, ROOT, in_body);
    let item_labels = labels(items.clone());
    for expected in [
        "Hero",
        "tick",
        "grant",
        "rewards",
        "let",
        "for player in players()",
        "continue",
    ] {
        assert!(
            item_labels.iter().any(|l| l == expected),
            "missing {expected} in {item_labels:?}"
        );
    }
    let grant = items.iter().find(|c| c.label == "grant").unwrap();
    assert_eq!(grant.kind, CompletionKind::Function);
    assert_eq!(grant.insert.as_deref(), Some("grant()"));
    // Expression position: values such as constants.
    let value = labels(ide::completions(
        &analysis,
        ROOT,
        offset(ROOT_TEXT, "MAX,", 0),
    ));
    assert!(value.iter().any(|l| l == "MAX"), "{value:?}");
    assert!(!value.iter().any(|l| l == "let"), "{value:?}");

    // Attribute names after `#[`.
    let attr = offset(ROOT_TEXT, "tick]", 0);
    let attrs = labels(ide::completions(&analysis, ROOT, attr));
    // Only attributes that apply to a function; `clamp` is for State fields.
    assert!(attrs.contains(&"tag".to_string()) && attrs.contains(&"tick".to_string()));
    assert!(!attrs.contains(&"clamp".to_string()), "{attrs:?}");
}

#[test]
fn enum_variants_complete_after_the_enum_path() {
    let text = "#![pack(namespace = \"x\")]\nenum Phase { Idle, Fight }\nplayer state H { p: Phase }\nfn f() { H.p = Phase::F; }\n";
    let analysis = analyze_one(text);
    let at = offset(text, "Phase::F", 0) + "Phase::".len();
    let variants: Vec<_> = ide::completions(&analysis, ROOT, at)
        .into_iter()
        .map(|c| c.label)
        .collect();
    assert_eq!(variants, vec!["Idle", "Fight"]);
}

#[test]
fn outline_nests_fields_variants_and_inline_modules() {
    let text = "#![pack(namespace = \"x\")]\nenum P { A, B }\nmod inner {\n    pub fn f() {}\n}\nglobal state G { a: number, b: number }\n";
    let analysis = analyze_one(text);
    let symbols = ide::document_symbols(&analysis, ROOT);
    let summary: Vec<_> = symbols
        .iter()
        .map(|s| (s.name.as_str(), s.kind, s.children.len()))
        .collect();
    assert_eq!(
        summary,
        vec![
            ("P", DefKind::Enum, 2),
            ("inner", DefKind::Module, 1),
            ("G", DefKind::State, 2)
        ]
    );
}

#[test]
fn inlay_hints_show_inferred_contexts_after_signatures() {
    let analysis = workspace();
    let hints = ide::inlay_hints(&analysis, REWARDS);
    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0].label, ": player");
    assert_eq!(
        &REWARDS_TEXT[..hints[0].offset],
        &REWARDS_TEXT[..offset(REWARDS_TEXT, "() {", 0) + 2]
    );
}

#[test]
fn recovery_keeps_the_rest_of_the_file_analysable() {
    let text = r#"#![pack(namespace = "x")]
player state H { hp: number }
fn broken( {
}
fn ok() {
    H.hp += ;
    H.hp = 3;
}
"#;
    let analysis = analyze_one(text);
    let errors = errors(&analysis);
    assert_eq!(
        errors,
        vec![
            "expected `)` or identifier; found `{`",
            "expected expression; found `;`"
        ]
    );
    // Hover and definition still work on the healthy statement.
    let at = offset(text, "hp = 3", 0);
    assert!(
        ide::hover(&analysis, ROOT, at)
            .unwrap()
            .markdown
            .contains("hp: number")
    );
    let symbols: Vec<_> = ide::document_symbols(&analysis, ROOT)
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(symbols, vec!["H", "ok"]);
}
