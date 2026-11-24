mod support;
use support::*;

#[test]
fn equipment_uses_typed_slots_and_custom_item_identity() {
    let analysis = analyze_one(include_str!("fixtures/equipment.sand"));
    let pack = compile(&analysis);
    let commands = pack
        .resources
        .iter()
        .filter(|(name, _)| name.ends_with(".mcfunction"))
        .map(|(_, bytes)| std::str::from_utf8(bytes).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for expected in [
        "item replace entity @s armor.head with minecraft:iron_helmet[",
        "if items entity @s armor.head minecraft:iron_helmet[",
        "item replace entity @s weapon.offhand with minecraft:arrow 3",
        "item replace entity @s weapon.offhand with air",
    ] {
        assert!(commands.contains(expected), "missing {expected}");
    }
}

#[test]
fn equipment_rejects_ambiguous_slots_and_invalid_counts() {
    for (statement, expected) in [
        (
            "equip(Item::Stick, EquipmentSlot::Body);",
            "no equipment slot named",
        ),
        (
            "equip(Item::Stick, EquipmentSlot::Head, count = 100);",
            "expected a value from 1 to 99",
        ),
        (
            "if holding(Item::Stick, offhand = true, slot = EquipmentSlot::Head) { return; }",
            "choose either slot or offhand",
        ),
    ] {
        let analysis = analyze_one(&format!(
            "#![pack(namespace = \"equipment\")]\nfn try_it() {{ {statement} }}"
        ));
        assert!(
            errors(&analysis)
                .iter()
                .any(|error| error.contains(expected)),
            "{:?}",
            errors(&analysis)
        );
    }

    let analysis = analyze_one(
        "#![pack(namespace = \"equipment\")]\nfn try_it() { clear_equipment(EquipmentSlot::Head); }",
    );
    let mut program = serde_json::to_value(analysis.program.unwrap()).unwrap();
    let action = &mut program["modules"][0]["functions"][0]["body"][0]["action"];
    action["slot"] = "body".into();
    action["count"] = 1.into();
    let program = serde_json::from_value(program).unwrap();
    let issues = sand::advanced::compiler::Compiler::compile(&program).unwrap_err();
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("EquipmentSlot::Body")),
        "{issues:?}"
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("stack count requires an item")),
        "{issues:?}"
    );
}
