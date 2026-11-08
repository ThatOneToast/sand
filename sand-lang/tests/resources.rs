//! Data declarations: custom items, recipes, and registry tags.

mod support;

use sand_lang::{FileId, ide};
use support::*;

const PACK: &str = "#![pack(namespace = \"forge\", minecraft = \"26.2\")]\n";

fn pack(body: &str) -> String {
    format!("{PACK}{body}")
}

fn json(pack: &sand::advanced::compiler::CompiledPack, path: &str) -> serde_json::Value {
    serde_json::from_slice(&pack.resources[path]).unwrap_or_else(|_| panic!("missing {path}"))
}

const FORGE: &str = r##"
/// A sword forged from redstone.
item RubySword {
    base: Item::DiamondSword,
    name: "{red}{bold}Ruby Sword",
    lore: ["{gray}Forged in fire."],
    rarity: Rarity::Epic,
    unbreakable: true,
}

tag Ores: Item { values: [Item::IronOre, "#minecraft:coal_ores"] }
tag AllOres: Item { values: [Ores, Item::CopperOre] }

recipe RubySwordRecipe {
    shaped: [" R ", " R ", " S "],
    key: { R: Item::RedstoneBlock, S: Item::Stick },
    result: RubySword,
}

recipe Nugget {
    smelting: AllOres,
    result: Item::IronNugget,
    experience: 0.5,
    time: 100,
}

recipe RedDye { shapeless: [Item::Poppy, Item::Poppy], result: Item::RedDye, count: 2 }
recipe Bricks { stonecutting: Item::Stone, result: Item::StoneBricks }

fn starter() {
    give_item(RubySword);
    clear_item(RubySword);
}
"##;

#[test]
fn declarations_lower_to_the_same_resources_as_rust() {
    let analysis = analyze_one(&pack(FORGE));
    let pack = compile(&analysis);
    let shaped = json(&pack, "data/forge/recipe/ruby_sword_recipe.json");
    assert_eq!(shaped["type"], "minecraft:crafting_shaped");
    assert_eq!(shaped["key"]["R"], "minecraft:redstone_block");
    assert_eq!(
        shaped["result"]["components"]["minecraft:custom_data"]["forge:ruby_sword"],
        true
    );
    assert_eq!(shaped["result"]["components"]["minecraft:rarity"], "epic");
    let nugget = json(&pack, "data/forge/recipe/nugget.json");
    assert_eq!(nugget["type"], "minecraft:smelting");
    assert_eq!(nugget["ingredient"], "#forge:all_ores");
    assert_eq!(nugget["cookingtime"], 100);
    assert_eq!(
        json(&pack, "data/forge/recipe/red_dye.json")["result"]["count"],
        2
    );
    assert_eq!(
        json(&pack, "data/forge/recipe/bricks.json")["type"],
        "minecraft:stonecutting"
    );
    assert_eq!(
        json(&pack, "data/forge/tags/item/all_ores.json")["values"],
        serde_json::json!(["#forge:ores", "minecraft:copper_ore"])
    );
    let starter = function(&pack, "forge/function/starter");
    assert!(
        starter.contains("give @s minecraft:diamond_sword[custom_data={\"forge:ruby_sword\":1b}"),
        "{starter}"
    );
    assert!(
        starter.contains(
            "clear @s minecraft:diamond_sword[minecraft:custom_data~{\"forge:ruby_sword\":1b}]"
        ),
        "{starter}"
    );
}

#[test]
fn declaration_mistakes_are_explained() {
    let analysis = analyze_one(&pack(
        r#"
item Gem { base: Item::Emerald, nmae: "Gem" }
item Broken { name: "{Score.v} pts" }
recipe A { shaped: ["AB"], key: { A: Gem, C: Item::Stick }, result: Item::Stick }
recipe B { result: Item::Stick }
recipe C { smelting: Item::Sand, shapeless: [Item::Sand], result: Item::Glass }
recipe D { shapeless: [Item::Poppy], result: Gem, key: { A: Item::Stick } }
tag E: Block { values: [Item::Diamond] }
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "`item` has no field `nmae`",
        "`item` needs `base`",
        "`C` is not used in the pattern",
        "custom item `Gem` cannot be a recipe ingredient",
        "a recipe needs a type",
        "a recipe has exactly one type",
        "`key` does not apply to a `shapeless` recipe",
        "expected a `Block`, found an `Item`",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
    let helps: Vec<_> = analysis
        .diagnostics
        .iter()
        .filter_map(|d| d.help.clone())
        .collect();
    assert!(
        helps.contains(&"did you mean `name`?".to_string()),
        "{helps:?}"
    );
}

#[test]
fn declarations_complete_fields_and_typed_values_and_explain_them() {
    let text = pack(
        "item Gem { base: Item::Emerald }\nitem Draft {\n    \n    base: Item::,\n    rarity: ,\n}\nrecipe R {\n    result: ,\n}\n",
    );
    let analysis = analyze_one(&text);
    let at = |needle: &str, delta: usize| text.find(needle).unwrap() + delta;
    let labels = |offset: usize| -> Vec<String> {
        ide::completions(&analysis, FileId(0), offset)
            .into_iter()
            .map(|c| c.label)
            .collect()
    };
    let fields = labels(at("Draft {\n    \n", 12));
    for expected in ["base", "name", "lore", "rarity", "glint"] {
        assert!(
            fields.iter().any(|l| l == expected),
            "missing {expected} in {fields:?}"
        );
    }
    assert!(
        !fields.iter().any(|l| l == "shaped"),
        "recipe fields are not item fields"
    );
    assert!(labels(at("Item::,", 6)).iter().any(|l| l == "DiamondSword"));
    assert_eq!(
        labels(at("rarity: ,", 8)),
        vec![
            "Rarity::Common",
            "Rarity::Uncommon",
            "Rarity::Rare",
            "Rarity::Epic"
        ]
    );
    let result = labels(at("result: ,", 8));
    assert!(
        result.iter().any(|l| l == "Gem") && result.iter().any(|l| l == "Item::Diamond"),
        "custom and vanilla results"
    );

    let hover = ide::hover(&analysis, FileId(0), at("rarity:", 1)).unwrap();
    assert!(
        hover.markdown.contains("Name colour: `Rarity::Common`"),
        "{}",
        hover.markdown
    );
    let keyword = ide::hover(&analysis, FileId(0), at("recipe R", 1)).unwrap();
    assert!(
        keyword
            .markdown
            .contains("adds a crafting, cooking, or stonecutting recipe")
    );
}

#[test]
fn custom_item_hover_and_rename_follow_uses() {
    let text = pack(FORGE);
    let analysis = analyze_one(&text);
    let use_offset = text.find("give_item(RubySword)").unwrap() + 10;
    let hover = ide::hover(&analysis, FileId(0), use_offset).unwrap();
    assert!(
        hover.markdown.contains("item RubySword"),
        "{}",
        hover.markdown
    );
    assert!(
        hover.markdown.contains("Custom item `forge:ruby_sword`"),
        "{}",
        hover.markdown
    );
    assert!(hover.markdown.contains("A sword forged from redstone."));
    let edits = ide::rename(&analysis, FileId(0), use_offset, "Ruby").unwrap();
    assert_eq!(
        edits.len(),
        4,
        "declaration, recipe result, give, clear: {edits:?}"
    );
}
