mod support;
use support::*;

#[test]
fn typed_and_external_enchantments_reach_the_item_components() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "forge")]
item Sword {
    base: Item::DiamondSword,
    enchantments: [
        { enchantment: Enchantment::Sharpness, level: 5 },
        { enchantment: "other_pack:luminous", level: 1 },
    ],
}
fn deliver() { give_item(Sword); }
"#,
    );
    let pack = compile(&analysis);
    let command = function(&pack, "forge/function/deliver");
    assert!(command.contains("enchantments={"), "{command}");
    assert!(command.contains("\"minecraft:sharpness\":5"), "{command}");
    assert!(command.contains("\"other_pack:luminous\":1"), "{command}");
}

#[test]
fn vanilla_enchantment_matches_the_rust_identifier_wrapper() {
    let generated = sand::vanilla::Enchantment::Sharpness;
    let id: sand::registry::EnchantmentId = generated.into();
    let catalog = sand_version::registry::variants("26.2", "minecraft:enchantment").unwrap();
    assert_eq!(id.to_string(), catalog["Sharpness"]);
}

#[test]
fn invalid_enchantments_fail_at_both_authoring_boundaries() {
    let source = r#"
#![pack(namespace = "forge", minecraft = "26.2")]
item Sword { base: Item::DiamondSword, enchantments: [{ enchantment: Enchantment::Sharpness, level: 5 }] }
"#;
    let analysis = analyze_one(&source.replace("Sharpness", "NotAnEnchantment"));
    assert!(
        errors(&analysis)
            .iter()
            .any(|message| message.contains("no enchantment named"))
    );

    let analysis = analyze_one(source);
    let mut program = serde_json::to_value(analysis.program.unwrap()).unwrap();
    program["modules"][0]["items"][0]["enchantments"] =
        serde_json::json!({"minecraft:not_an_enchantment": 5});
    let program = serde_json::from_value(program).unwrap();
    let issues = sand::advanced::compiler::Compiler::compile(&program).unwrap_err();
    assert!(issues.iter().any(|issue| {
        issue
            .message
            .contains("no enchantment `minecraft:not_an_enchantment`")
    }));
}
