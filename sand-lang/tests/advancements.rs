mod support;

use support::*;

const MILESTONE: &str = r#"
advancement Scout {
    display: {
        icon: Item::Spyglass,
        title: "{gold}Scout",
        description: Text::translate("item.minecraft.spyglass"),
        background: "minecraft:gui/advancements/backgrounds/adventure",
    },
    criteria: {
        find_paper: { inventory_item: Item::Paper },
        place_stone: { place_block: Block::Stone },
        hunt_pig: { kill_entity: Entity::Pig },
    },
    require_any: true,
    experience: 5,
}
"#;

#[test]
fn advancements_export_native_display_requirements_and_typed_triggers() {
    for target in ["26.2", "26.3"] {
        let source =
            format!(r#"#![pack(namespace = "milestone", minecraft = "{target}")]{MILESTONE}"#);
        let pack = compile(&analyze_one(&source));
        let resource: serde_json::Value =
            serde_json::from_slice(&pack.resources["data/milestone/advancement/scout.json"])
                .unwrap();
        assert_eq!(resource["display"]["title"]["extra"][0]["text"], "Scout");
        assert_eq!(
            resource["display"]["description"]["extra"][0]["translate"],
            "item.minecraft.spyglass"
        );
        assert_eq!(
            resource["criteria"]["find_paper"]["trigger"],
            "minecraft:inventory_changed"
        );
        assert_eq!(
            resource["criteria"]["place_stone"]["trigger"],
            "minecraft:placed_block"
        );
        assert_eq!(
            resource["criteria"]["hunt_pig"]["trigger"],
            "minecraft:player_killed_entity"
        );
        assert_eq!(
            resource["requirements"],
            serde_json::json!([["find_paper", "hunt_pig", "place_stone"]])
        );
        assert_eq!(resource["rewards"]["experience"], 5);
    }
}

#[test]
fn advancement_requires_each_named_criterion_by_default() {
    let source = format!(
        r#"#![pack(namespace = "milestone", minecraft = "26.2")]{}"#,
        MILESTONE.replace("    require_any: true,\n", "")
    );
    let pack = compile(&analyze_one(&source));
    let resource: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/milestone/advancement/scout.json"]).unwrap();
    assert_eq!(
        resource["requirements"],
        serde_json::json!([["find_paper"], ["hunt_pig"], ["place_stone"]])
    );
}

#[test]
fn advancement_declarations_validate_trigger_names_types_and_display() {
    for (source, want) in [
        (
            "advancement Empty { criteria: {} }",
            "needs at least one criterion",
        ),
        (
            "advancement Invalid { criteria: { a: { place_block: Item::Paper } } }",
            "expected a `Block`",
        ),
        (
            "advancement Invalid { criteria: { a: { inventory_item: Item::Paper, kill_entity: Entity::Pig } } }",
            "exactly one trigger",
        ),
        (
            "advancement Invalid { criteria: { a: { inventory_item: Item::Paper }, a: { kill_entity: Entity::Pig } } }",
            "duplicate criterion",
        ),
        (
            r#"advancement Invalid { criteria: { a: { inventory_item: Item::Paper } }, display: { icon: Item::Paper, title: "{player}", description: "Desc" } }"#,
            "resource text cannot read runtime values",
        ),
    ] {
        let errors = errors(&analyze_one(&format!(
            r#"#![pack(namespace = "milestone")] {source}"#
        )));
        assert!(
            errors.iter().any(|error| error.contains(want)),
            "{errors:#?}"
        );
    }
}
