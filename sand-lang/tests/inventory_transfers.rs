mod support;

use support::*;

const SOURCE: &str = r#"
#![pack(namespace = "inventory_probe", minecraft = "26.2")]
inventory_transfer SortKit { from: 0, to: 9, amount: 2, success: moved, rejected: rejected }
fn moved() { chat!("Moved"); }
fn rejected() { chat!("Rejected"); }
fn move_kit() { transfer_inventory(SortKit); }
"#;

#[test]
fn inventory_transfers_register_native_operations_and_callbacks() {
    let analysis = analyze_one(SOURCE);
    let pack = compile(&analysis);
    let call = function(&pack, "inventory_probe/function/move_kit");
    let entry = call
        .lines()
        .find_map(|line| {
            line.strip_prefix("function ")
                .filter(|line| line.contains("inventory_transfer_move"))
        })
        .expect("calls the registered canonical move");
    let path = entry.strip_prefix("inventory_probe:").unwrap();
    assert!(function(&pack, &format!("inventory_probe/function/{path}")).contains("return"));
    for expected in [
        "function inventory_probe:moved",
        "function inventory_probe:rejected",
    ] {
        assert!(pack.resources.iter().any(|(path, bytes)| {
            path.ends_with(".mcfunction") && String::from_utf8_lossy(bytes).contains(expected)
        }));
    }
    let document = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&document, &Default::default()).unwrap();
    let recompiled = sand::advanced::compiler::Compiler::compile(&decoded).unwrap();
    assert_eq!(pack.resources, recompiled.resources);
}

#[test]
fn inventory_transfers_validate_slot_ranges_amounts_and_callbacks() {
    for (from, to, expected) in [
        ("to: 9", "to: 36", "from 0 to 35"),
        ("to: 9", "to: 0", "must be distinct"),
        ("amount: 2", "amount: 100", "from 1 to 99"),
        (
            "fn moved()",
            "fn moved(amount: number)",
            "expected a parameterless function",
        ),
    ] {
        let analysis = analyze_one(&SOURCE.replace(from, to));
        assert!(
            errors(&analysis)
                .iter()
                .any(|error| error.contains(expected)),
            "{:?}",
            errors(&analysis)
        );
    }
}
