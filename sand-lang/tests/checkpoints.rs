//! Portable player checkpoints use the shared ownership and transaction runtime.
mod support;
use support::*;

const PACK: &str = r#"
#![pack(namespace = "journal", minecraft = "26.2")]
player state Live { value: number }
player state Saved { valid: bool, #[clamp(0..=100)] value: number }

checkpoint Snapshot {
    valid: Saved.valid,
    fields: [{ live: Live.value, saved: Saved.value }],
    before_restore: prepare,
    after_restore: restored,
}

fn prepare() { Live.value = 10; }
fn restored() { Live.value += 1; }

#[player]
fn operate() {
    save_checkpoint(Snapshot);
    restore_checkpoint(Snapshot);
    clear_checkpoint(Snapshot);
}
"#;

#[test]
fn checkpoints_export_shared_transactions_and_portable_round_trip() {
    let analysis = analyze_one(PACK);
    let pack = compile(&analysis);
    let call = function(&pack, "journal/function/operate");
    for name in ["save", "restore", "clear"] {
        assert!(call.contains(&format!("/checkpoint_{name}")), "{call}");
    }
    let bodies = pack
        .resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, contents)| std::str::from_utf8(contents).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(bodies.contains("function journal:prepare"));
    assert!(bodies.contains("function journal:restored"));
    assert!(bodies.contains("@a[nbt={UUID:$(owner)},limit=1]"));
    assert!(bodies.contains("matches 0..100 run return 0"));
    let portable = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&portable, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn checkpoints_validate_storage_scope_types_and_projection() {
    for (old, new) in [
        ("valid: bool", "valid: number"),
        ("valid: bool", "valid: bool = true"),
        ("player state Saved", "global state Saved"),
        ("saved: Saved.value", "saved: Live.value"),
        ("[{ live: Live.value, saved: Saved.value }]", "[]"),
    ] {
        let analysis = analyze_one(&PACK.replace(old, new));
        assert!(
            analysis.has_errors() || analysis.compile().is_err(),
            "{new}"
        );
    }
}
