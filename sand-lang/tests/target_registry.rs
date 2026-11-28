//! Target-specific values and IDE suggestions must not follow the CLI's codegen version.
mod support;
use sand_lang::{FileId, ide};
use support::*;

fn pack(version: &str, body: &str) -> String {
    format!("#![pack(namespace = \"demo\", minecraft = \"{version}\")]\n{body}")
}

#[test]
fn typed_and_string_vanilla_values_follow_the_pack_target() {
    for value in [
        "Item::BlackConcreteSlab",
        "\"minecraft:black_concrete_slab\"",
    ] {
        let body = format!("fn reward() {{ give_item({value}); }}");
        let older = analyze_one(&pack("26.2", &body));
        assert!(older.has_errors(), "{value}: {:?}", errors(&older));
        assert!(
            errors(&older).iter().any(|error| error.contains("26.2")),
            "{:?}",
            errors(&older)
        );
        compile(&analyze_one(&pack("26.3", &body)));
    }
    compile(&analyze_one(&pack(
        "26.2",
        "fn reward() { give_item(\"custom:ruby\"); }",
    )));
}

#[test]
fn completion_only_offers_values_in_the_selected_target() {
    for (version, expected) in [("26.2", false), ("26.3", true)] {
        let text = pack(version, "fn reward() { give_item(Item::); }");
        let analysis = analyze_one(&text);
        let offset = text.find("Item::").unwrap() + "Item::".len();
        let items = ide::completions(&analysis, FileId(0), offset);
        assert_eq!(
            items.iter().any(|item| item.label == "BlackConcreteSlab"),
            expected,
            "{version}"
        );
        assert!(items.iter().any(|item| item.label == "Bread"));
    }
}

#[test]
fn portable_json_cannot_bypass_target_validation() {
    let source = pack(
        "26.3",
        "fn reward() { give_item(Item::BlackConcreteSlab); }",
    );
    let analysis = analyze_one(&source);
    let mut program = analysis.program.unwrap();
    program.target.minecraft = "26.2".into();
    let errors = sand::advanced::compiler::Compiler::compile(&program).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == "SAND_PROGRAM_TARGET"
                && error.message.contains("black_concrete_slab")),
        "{errors:?}"
    );
}

#[test]
fn resource_declarations_cannot_bypass_target_validation() {
    for body in [
        "item Future { base: Item::BlackConcreteSlab }",
        "recipe Future { shapeless: [Item::BlackConcreteSlab], result: Item::Bread }",
        "recipe Future { shapeless: [Item::Bread], result: Item::BlackConcreteSlab }",
        "tag Future: Item { values: [Item::BlackConcreteSlab] }",
    ] {
        let analysis = analyze_one(&pack("26.3", body));
        compile(&analysis);
        let mut program = analysis.program.unwrap();
        program.target.minecraft = "26.2".into();
        assert!(
            sand::advanced::compiler::Compiler::compile(&program).is_err(),
            "{body}"
        );
    }
    // Enchantments are their own registry, and tags are references to groups,
    // neither should be looked up as individual item identities.
    compile(&analyze_one(&pack(
        "26.2",
        r##"
item Sword { base: Item::DiamondSword, enchantments: { "minecraft:sharpness": 5 } }
tag Logs: Block { values: ["#minecraft:logs", Block::OakLog] }
tag Tools: Item { values: ["#minecraft:axes", Item::DiamondAxe] }
"##,
    )));
}

#[test]
fn target_aliases_select_the_same_verified_catalog() {
    compile(&analyze_one(&pack(
        "latest",
        "fn f() { give_item(Item::BlackConcreteSlab); }",
    )));
    compile(&analyze_one(&pack(
        "26.2.0",
        "fn f() { give_item(Item::Bread); }",
    )));
}

#[test]
fn capabilities_advertise_the_exported_vocabulary_and_verified_targets() {
    let capabilities = sand::advanced::compiler::Compiler::capabilities();
    for operation in [
        "data_get",
        "teleport",
        "add_experience",
        "push_frame",
        "set_time",
    ] {
        assert!(
            capabilities["operations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == operation),
            "missing {operation}"
        );
    }
    for event in ["mine_block", "craft_item", "killed_by"] {
        assert!(
            capabilities["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == event),
            "missing {event}"
        );
    }
    assert!(
        capabilities["targets"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "26.3")
    );
}

#[test]
fn portable_runtime_amounts_validate_scales_and_field_references() {
    fn replace_amount(value: &mut serde_json::Value, amount: &serde_json::Value) -> bool {
        if value["op"] == "damage" {
            value["amount"] = amount.clone();
            return true;
        }
        match value {
            serde_json::Value::Array(values) => {
                values.iter_mut().any(|value| replace_amount(value, amount))
            }
            serde_json::Value::Object(values) => values
                .values_mut()
                .any(|value| replace_amount(value, amount)),
            _ => false,
        }
    }
    let analysis = analyze_one(&pack("26.2", "fn hit(n: float) { damage(n); }"));
    compile(&analysis);
    let original = serde_json::to_value(analysis.program.unwrap()).unwrap();
    fn find_amount(value: &serde_json::Value) -> Option<&serde_json::Value> {
        if value["op"] == "damage" {
            return Some(&value["amount"]);
        }
        match value {
            serde_json::Value::Array(values) => values.iter().find_map(find_amount),
            serde_json::Value::Object(values) => values.values().find_map(find_amount),
            _ => None,
        }
    }
    let mut amounts = vec![
        serde_json::json!({"score": {"state": "demo:missing", "field": "missing"}, "scale": 1000.0}),
        serde_json::json!(-1.0),
        serde_json::json!(1000001.0),
    ];
    for scale in [0.0, -1.0, 1e-320] {
        let mut amount = find_amount(&original).unwrap().clone();
        amount["scale"] = serde_json::json!(scale);
        amounts.push(amount);
    }
    for amount in amounts {
        let mut value = original.clone();
        assert!(replace_amount(&mut value, &amount));
        let program = serde_json::from_value(value).unwrap();
        assert!(
            sand::advanced::compiler::Compiler::compile(&program).is_err(),
            "accepted {amount}"
        );
    }
}
