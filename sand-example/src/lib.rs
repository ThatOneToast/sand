//! # hello_world
//!
//! A minimal example datapack built with [Sand](https://crates.io/crates/sand),
//! targeting Minecraft 1.21.11.
//!
//! This crate also serves as the primary integration test for the Sand
//! workspace — every module exercises a different part of the pipeline.

pub mod join_example;
pub use join_example::*;

use sand_core::mcfunction;
use sand_macros::{component, function, run_fn};

// ── Datapack functions ────────────────────────────────────────────────────────

/// Welcomes all online players to the world.
///
/// Registered automatically as `<namespace>:hello_world` via inventory.
#[function]
pub fn hello_world() {
    mcfunction! {
        r#"tellraw @a {"text":"Welcome to the world!","color":"gold","bold":true}"#;
        "playsound minecraft:ui.toast.challenge_complete master @a ~ ~ ~ 1 1";
    }
}

/// Called every game tick. Placeholder for per-tick logic.
#[function]
pub fn tick() {
    mcfunction! {
        "scoreboard players add @a playtime 1";
    }
}

/// Demonstrates `run_fn!` — calls an inline function via execute.
#[function]
pub fn execute_example() {
    use sand_core::cmd::{Execute, Selector};
    Execute::new()
        .as_(Selector::all_players())
        .run(run_fn!("hello_world:greet_inline" {
            "say Welcome from the inline function!";
        }));
}

// ── Components ───────────────────────────────────────────────────────────────

/// Fires `hello_world:hello_world` once per player on their first tick alive.
#[component]
pub fn player_join_advancement() -> sand_core::Advancement {
    use sand_core::{Advancement, AdvancementRewards, AdvancementTrigger, Criterion};
    Advancement::new("hello_world:player_join".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
        .rewards(AdvancementRewards::new().function("hello_world:hello_world"))
}

// ── Export hook ───────────────────────────────────────────────────────────────

/// Invoked by the generated `sand_export` binary.
///
/// Calling this function from the binary forces the linker to include this
/// library's object file, which is required for `inventory::submit!`
/// constructor functions to run before `main`.
#[doc(hidden)]
pub fn __sand_export(namespace: &str) {
    println!("{}", sand_core::export_components_json(namespace));
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sand_core::inventory;
    use sand_core::{DatapackComponent, FunctionDescriptor, McVersion, PackNamespace};

    // ── mcfunction! macro ─────────────────────────────────────────────────────

    #[test]
    fn mcfunction_macro_single_command() {
        let cmds = sand_core::mcfunction!["say hello"];
        assert_eq!(cmds, vec!["say hello".to_string()]);
    }

    #[test]
    fn mcfunction_macro_multiple_commands() {
        let cmds = sand_core::mcfunction![
            "say hello";
            "give @a diamond 1";
            r#"tellraw @a {"text":"hi"}"#;
        ];
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[1], "give @a diamond 1");
    }

    // ── #[function] macro ─────────────────────────────────────────────────────

    #[test]
    fn function_macro_returns_commands() {
        let cmds = hello_world();
        assert_eq!(cmds.len(), 2);
        assert!(cmds[0].contains("tellraw"));
        assert!(cmds[1].contains("playsound"));
    }

    #[test]
    fn function_macro_inventory_registration() {
        // Verify that #[function]-annotated functions are registered.
        let paths: Vec<&str> = inventory::iter::<FunctionDescriptor>()
            .map(|d| d.path)
            .collect();

        assert!(
            paths.contains(&"hello_world"),
            "hello_world not found in inventory: {paths:?}"
        );
        assert!(
            paths.contains(&"tick"),
            "tick not found in inventory: {paths:?}"
        );
    }

    // ── run_fn! macro ─────────────────────────────────────────────────────────

    #[test]
    fn run_fn_produces_execute_function_command() {
        let cmds = execute_example();
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            "execute as @a run function hello_world:greet_inline"
        );
    }

    #[test]
    fn run_fn_registers_inline_function() {
        let paths: Vec<&str> = inventory::iter::<FunctionDescriptor>()
            .map(|d| d.path)
            .collect();
        assert!(
            paths.contains(&"greet_inline"),
            "greet_inline not found in inventory: {paths:?}"
        );
    }

    #[test]
    fn run_fn_inline_body_commands_correct() {
        let descriptor = inventory::iter::<FunctionDescriptor>()
            .find(|d| d.path == "greet_inline")
            .expect("greet_inline descriptor not registered");
        let cmds = (descriptor.make)();
        assert_eq!(cmds, vec!["say Welcome from the inline function!"]);
    }

    #[test]
    fn function_descriptor_commands_correct() {
        let descriptor = inventory::iter::<FunctionDescriptor>()
            .find(|d| d.path == "hello_world")
            .expect("hello_world descriptor not registered");

        let commands = (descriptor.make)();
        assert_eq!(commands.len(), 2);
        assert!(commands[0].contains("tellraw"));
    }

    // ── ResourceLocation ──────────────────────────────────────────────────────

    #[test]
    fn resource_location_roundtrip() {
        use sand_core::ResourceLocation;
        let loc: ResourceLocation = "hello_world:hello_world".parse().unwrap();
        assert_eq!(loc.namespace(), "hello_world");
        assert_eq!(loc.path(), "hello_world");
        assert_eq!(loc.to_string(), "hello_world:hello_world");
    }

    #[test]
    fn resource_location_rejects_uppercase() {
        use sand_core::ResourceLocation;
        assert!(ResourceLocation::new("Hello", "world").is_err());
        assert!(ResourceLocation::new("hello", "World").is_err());
    }

    #[test]
    fn resource_location_serde() {
        use sand_core::ResourceLocation;
        let loc = ResourceLocation::minecraft("stone").unwrap();
        let json = serde_json::to_string(&loc).unwrap();
        assert_eq!(json, r#""minecraft:stone""#);
        let back: ResourceLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(loc, back);
    }

    // ── McVersion ─────────────────────────────────────────────────────────────

    #[test]
    fn mc_version_parse_and_order() {
        let v1: McVersion = "1.20.1".parse().unwrap();
        let v2: McVersion = "1.21.4".parse().unwrap();
        assert!(v1 < v2);
        assert_eq!(v2.to_string(), "1.21.4");
    }

    // ── PackNamespace ─────────────────────────────────────────────────────────

    #[test]
    fn pack_namespace_valid() {
        let ns = PackNamespace::new("hello_world").unwrap();
        assert_eq!(ns.as_str(), "hello_world");
    }

    // ── Generated registry types ──────────────────────────────────────────────

    #[test]
    fn generated_item_resource_location() {
        use sand_core::generated::Item;
        assert_eq!(Item::Air.resource_location(), "minecraft:air");
        assert_eq!(Item::OakLog.to_string(), "minecraft:oak_log");
    }

    #[test]
    fn generated_block_resource_location() {
        use sand_core::generated::Block;
        assert_eq!(Block::Air.resource_location(), "minecraft:air");
        assert_eq!(Block::Bedrock.resource_location(), "minecraft:bedrock");
    }

    #[test]
    fn generated_entity_type_exists() {
        use sand_core::generated::EntityType;
        assert_eq!(EntityType::Zombie.resource_location(), "minecraft:zombie");
        assert_eq!(EntityType::Creeper.resource_location(), "minecraft:creeper");
    }

    #[test]
    fn generated_sound_event_exists() {
        use sand_core::generated::SoundEvent;
        assert!(
            SoundEvent::EntityPlayerDeath
                .resource_location()
                .starts_with("minecraft:")
        );
    }

    // ── Block state property types ────────────────────────────────────────────

    #[test]
    fn block_state_facing_as_str() {
        use sand_core::block_states::Facing2;
        assert_eq!(Facing2::North.as_str(), "north");
        assert_eq!(Facing2::East.as_str(), "east");
    }

    #[test]
    fn oak_door_properties_struct_exists() {
        use sand_core::block_states::{Facing2, Half1, Hinge, OakDoorProperties};
        let props = OakDoorProperties {
            facing: Facing2::North,
            half: Half1::Lower,
            hinge: Hinge::Left,
            open: false,
            powered: false,
        };
        assert_eq!(props.facing.as_str(), "north");
        assert!(!props.open);
    }

    // ── DatapackComponent trait ───────────────────────────────────────────────

    #[test]
    fn player_join_advancement_component() {
        let adv = player_join_advancement();
        assert_eq!(
            adv.resource_location().to_string(),
            "hello_world:player_join"
        );
        let json = adv.to_json();
        assert_eq!(
            json["rewards"]["function"].as_str().unwrap(),
            "hello_world:hello_world"
        );
        assert_eq!(
            json["criteria"]["tick"]["trigger"].as_str().unwrap(),
            "minecraft:tick"
        );
    }

    #[test]
    fn component_macro_inventory_registration() {
        use sand_core::ComponentFactory;
        let count = sand_core::inventory::iter::<ComponentFactory>().count();
        assert!(
            count >= 1,
            "expected at least 1 #[component] registration, got {count}"
        );
    }

    // ── Tag component ─────────────────────────────────────────────────────────

    #[test]
    fn tag_json_output() {
        use sand_core::Tag;
        use sand_core::generated::Block;
        let tag = Tag::new("hello_world:oak_things".parse().unwrap())
            .entry(Block::OakLog)
            .entry(Block::OakWood)
            .tag_ref("minecraft:logs");
        let json = tag.to_json();
        assert_eq!(json["replace"], false);
        let values = json["values"].as_array().unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].as_str().unwrap(), "minecraft:oak_log");
        assert_eq!(values[2].as_str().unwrap(), "#minecraft:logs");
    }

    // ── McFunction component ──────────────────────────────────────────────────

    #[test]
    fn mc_function_text_content() {
        use sand_core::{ComponentContent, McFunction};
        let func = McFunction::new("hello_world:greet".parse().unwrap())
            .command("say Hello")
            .command("give @a minecraft:diamond 1");
        assert_eq!(func.resource_location().to_string(), "hello_world:greet");
        match func.content() {
            ComponentContent::Text(t) => {
                assert_eq!(t, "say Hello\ngive @a minecraft:diamond 1");
            }
            ComponentContent::Json(_) => panic!("expected Text content"),
        }
    }

    // ── ShapedRecipe component ────────────────────────────────────────────────

    #[test]
    fn shaped_recipe_json_output() {
        use sand_core::{Ingredient, RecipeResult, ShapedRecipe};
        let recipe = ShapedRecipe::new("hello_world:diamond_sword".parse().unwrap())
            .pattern(["#", "D", "D"])
            .key('#', Ingredient::item("minecraft:stick"))
            .key('D', Ingredient::item("minecraft:diamond"))
            .result(RecipeResult::new("minecraft:diamond_sword", 1))
            .category("equipment");
        let json = recipe.to_json();
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:crafting_shaped");
        assert_eq!(json["category"].as_str().unwrap(), "equipment");
        assert_eq!(
            json["key"]["D"]["item"].as_str().unwrap(),
            "minecraft:diamond"
        );
        assert_eq!(
            json["result"]["id"].as_str().unwrap(),
            "minecraft:diamond_sword"
        );
    }

    // ── Advancement component ─────────────────────────────────────────────────

    #[test]
    fn advancement_json_output() {
        use sand_core::{
            Advancement, AdvancementDisplay, AdvancementFrame, AdvancementIcon, AdvancementRewards,
            AdvancementTrigger, Criterion,
        };
        let adv = Advancement::new("hello_world:kill_zombie".parse().unwrap())
            .display(
                AdvancementDisplay::new(
                    AdvancementIcon::new("minecraft:diamond_sword"),
                    "Zombie Slayer",
                    "Kill your first zombie",
                )
                .frame(AdvancementFrame::Challenge),
            )
            .criterion(
                "kill",
                Criterion::new(AdvancementTrigger::PlayerKilledEntity {
                    entity: Some(serde_json::json!({"type": "minecraft:zombie"})),
                    killing_blow: None,
                }),
            )
            .rewards(AdvancementRewards::new().experience(100));
        let json = adv.to_json();
        assert_eq!(json["display"]["frame"].as_str().unwrap(), "challenge");
        assert_eq!(
            json["criteria"]["kill"]["trigger"].as_str().unwrap(),
            "minecraft:player_killed_entity"
        );
        assert_eq!(json["rewards"]["experience"].as_i64().unwrap(), 100);
    }

    // ── LootTable component ───────────────────────────────────────────────────

    #[test]
    fn loot_table_json_output() {
        use sand_core::{LootEntry, LootPool, LootTable, LootTableType, NumberProvider};
        let table = LootTable::new("hello_world:chest_loot".parse().unwrap())
            .loot_type(LootTableType::Chest)
            .pool(
                LootPool::new()
                    .rolls(NumberProvider::Uniform { min: 1.0, max: 3.0 })
                    .entry(LootEntry::item("minecraft:diamond"))
                    .entry(LootEntry::item("minecraft:gold_ingot")),
            );
        let json = table.to_json();
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:chest");
        let pools = json["pools"].as_array().unwrap();
        assert_eq!(pools.len(), 1);
        assert_eq!(
            pools[0]["rolls"]["type"].as_str().unwrap(),
            "minecraft:uniform"
        );
        let entries = pools[0]["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["name"].as_str().unwrap(), "minecraft:diamond");
    }

    // ── Predicate component ───────────────────────────────────────────────────

    #[test]
    fn predicate_json_output() {
        use sand_core::{LootCondition, Predicate};
        let pred = Predicate::new(
            "hello_world:rare_drop".parse().unwrap(),
            LootCondition::RandomChance { chance: 0.1 },
        );
        let json = pred.to_json();
        assert_eq!(
            json["condition"].as_str().unwrap(),
            "minecraft:random_chance"
        );
        assert!((json["chance"].as_f64().unwrap() - 0.1).abs() < f64::EPSILON);
    }

    // ── ItemModifier component ────────────────────────────────────────────────

    #[test]
    fn item_modifier_single_function() {
        use sand_core::{ItemModifier, LootFunction, NumberProvider};
        let modifier = ItemModifier::new("hello_world:double_count".parse().unwrap()).function(
            LootFunction::SetCount {
                count: NumberProvider::Constant(2.0),
                add: false,
            },
        );
        let json = modifier.to_json();
        // Single function serializes as an object, not an array.
        assert_eq!(json["function"].as_str().unwrap(), "minecraft:set_count");
        assert_eq!(json["count"].as_f64().unwrap(), 2.0);
    }

    #[test]
    fn item_modifier_multiple_functions_serializes_as_array() {
        use sand_core::{ItemModifier, LootFunction, NumberProvider};
        let modifier = ItemModifier::new("hello_world:buff".parse().unwrap())
            .function(LootFunction::SetCount {
                count: NumberProvider::Constant(3.0),
                add: false,
            })
            .function(LootFunction::SetDamage {
                damage: NumberProvider::Uniform { min: 0.0, max: 0.5 },
                add: false,
            });
        let json = modifier.to_json();
        let arr = json
            .as_array()
            .expect("multiple functions should be an array");
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["function"].as_str().unwrap(), "minecraft:set_count");
        assert_eq!(arr[1]["function"].as_str().unwrap(), "minecraft:set_damage");
    }

    // ── Tag edge cases ────────────────────────────────────────────────────────

    #[test]
    fn tag_replace_flag() {
        use sand_core::Tag;
        let tag = Tag::new("hello_world:replaceable".parse().unwrap())
            .replace(true)
            .entry("minecraft:stone");
        let json = tag.to_json();
        assert_eq!(json["replace"], true);
    }

    #[test]
    fn tag_struct_literal_construction() {
        use sand_core::{DatapackComponent, Tag};
        let tag = Tag {
            location: "hello_world:manual".parse().unwrap(),
            replace: false,
            values: vec![
                "minecraft:oak_log".to_string(),
                "#minecraft:logs".to_string(),
            ],
        };
        let json = tag.to_json();
        assert_eq!(json["values"].as_array().unwrap().len(), 2);
        assert_eq!(tag.resource_location().to_string(), "hello_world:manual");
    }

    // ── McFunction edge cases ─────────────────────────────────────────────────

    #[test]
    fn mc_function_bulk_commands() {
        use sand_core::McFunction;
        let func = McFunction::new("hello_world:bulk".parse().unwrap()).commands([
            "say one",
            "say two",
            "say three",
        ]);
        let json = func.to_json();
        let arr = json.as_array().expect("to_json should be an array");
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_str().unwrap(), "say one");
    }

    #[test]
    fn mc_function_struct_literal() {
        use sand_core::{ComponentContent, McFunction};
        let func = McFunction {
            location: "hello_world:manual".parse().unwrap(),
            commands: vec!["say hello".into(), "say world".into()],
        };
        match func.content() {
            ComponentContent::Text(t) => assert_eq!(t, "say hello\nsay world"),
            ComponentContent::Json(_) => panic!("expected Text"),
        }
    }

    // ── Recipe — all remaining types ──────────────────────────────────────────

    #[test]
    fn shapeless_recipe_json_output() {
        use sand_core::{Ingredient, RecipeResult, ShapelessRecipe};
        let recipe = ShapelessRecipe::new("hello_world:fire_charge".parse().unwrap())
            .ingredient(Ingredient::item("minecraft:blaze_powder"))
            .ingredient(Ingredient::item("minecraft:coal"))
            .ingredient(Ingredient::item("minecraft:gunpowder"))
            .result(RecipeResult::new("minecraft:fire_charge", 3));
        let json = recipe.to_json();
        assert_eq!(
            json["type"].as_str().unwrap(),
            "minecraft:crafting_shapeless"
        );
        let ings = json["ingredients"].as_array().unwrap();
        assert_eq!(ings.len(), 3);
        assert_eq!(ings[0]["item"].as_str().unwrap(), "minecraft:blaze_powder");
        assert_eq!(json["result"]["count"].as_u64().unwrap(), 3);
    }

    #[test]
    fn cooking_recipe_smelting_json_output() {
        use sand_core::{CookingRecipe, CookingType, Ingredient, RecipeResult};
        let recipe = CookingRecipe::new(
            "hello_world:iron_smelting".parse().unwrap(),
            CookingType::Smelting,
        )
        .ingredient(Ingredient::item("minecraft:iron_ore"))
        .result(RecipeResult::new("minecraft:iron_ingot", 1))
        .experience(0.7)
        .cooking_time(200);
        let json = recipe.to_json();
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:smelting");
        assert_eq!(
            json["ingredient"]["item"].as_str().unwrap(),
            "minecraft:iron_ore"
        );
        assert!((json["experience"].as_f64().unwrap() - 0.7).abs() < 0.001);
        assert_eq!(json["cookingtime"].as_u64().unwrap(), 200);
    }

    #[test]
    fn cooking_recipe_blasting_type_str() {
        use sand_core::{CookingRecipe, CookingType, Ingredient, RecipeResult};
        let recipe = CookingRecipe::new(
            "hello_world:gold_blasting".parse().unwrap(),
            CookingType::Blasting,
        )
        .ingredient(Ingredient::tag("minecraft:gold_ores"))
        .result(RecipeResult::new("minecraft:gold_ingot", 1));
        let json = recipe.to_json();
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:blasting");
        // tag ingredient serializes as {"tag": "..."}
        assert_eq!(
            json["ingredient"]["tag"].as_str().unwrap(),
            "minecraft:gold_ores"
        );
    }

    #[test]
    fn stonecutting_recipe_json_output() {
        use sand_core::{Ingredient, RecipeResult, StonecuttingRecipe};
        let recipe = StonecuttingRecipe::new("hello_world:stone_slab".parse().unwrap())
            .ingredient(Ingredient::item("minecraft:stone"))
            .result(RecipeResult::new("minecraft:stone_slab", 2))
            .count(2);
        let json = recipe.to_json();
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:stonecutting");
        assert_eq!(json["count"].as_u64().unwrap(), 2);
        assert_eq!(
            json["ingredient"]["item"].as_str().unwrap(),
            "minecraft:stone"
        );
    }

    #[test]
    fn smithing_transform_recipe_json_output() {
        use sand_core::{Ingredient, RecipeResult, SmithingTransformRecipe};
        let recipe = SmithingTransformRecipe::new("hello_world:netherite_sword".parse().unwrap())
            .template(Ingredient::item(
                "minecraft:netherite_upgrade_smithing_template",
            ))
            .base(Ingredient::item("minecraft:diamond_sword"))
            .addition(Ingredient::item("minecraft:netherite_ingot"))
            .result(RecipeResult::new("minecraft:netherite_sword", 1));
        let json = recipe.to_json();
        assert_eq!(
            json["type"].as_str().unwrap(),
            "minecraft:smithing_transform"
        );
        assert_eq!(
            json["base"]["item"].as_str().unwrap(),
            "minecraft:diamond_sword"
        );
        assert_eq!(
            json["result"]["id"].as_str().unwrap(),
            "minecraft:netherite_sword"
        );
    }

    #[test]
    fn smithing_trim_recipe_json_output() {
        use sand_core::{Ingredient, SmithingTrimRecipe};
        let recipe = SmithingTrimRecipe::new("hello_world:armor_trim".parse().unwrap())
            .template(Ingredient::tag("minecraft:trim_templates"))
            .base(Ingredient::tag("minecraft:trimmable_armor"))
            .addition(Ingredient::tag("minecraft:trim_materials"));
        let json = recipe.to_json();
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:smithing_trim");
        assert_eq!(
            json["template"]["tag"].as_str().unwrap(),
            "minecraft:trim_templates"
        );
    }

    // ── Advancement — additional triggers and fields ───────────────────────────

    #[test]
    fn advancement_tick_trigger() {
        use sand_core::{Advancement, AdvancementTrigger, Criterion};
        let adv = Advancement::new("hello_world:tick_test".parse().unwrap())
            .criterion("tick", Criterion::new(AdvancementTrigger::Tick));
        let json = adv.to_json();
        assert_eq!(
            json["criteria"]["tick"]["trigger"].as_str().unwrap(),
            "minecraft:tick"
        );
        // Tick has no conditions key.
        assert!(json["criteria"]["tick"]["conditions"].is_null());
    }

    #[test]
    fn advancement_parent_and_requirements() {
        use sand_core::{Advancement, AdvancementTrigger, Criterion};
        let adv = Advancement::new("hello_world:child".parse().unwrap())
            .parent("hello_world:root")
            .criterion("a", Criterion::new(AdvancementTrigger::Tick))
            .criterion("b", Criterion::new(AdvancementTrigger::Impossible))
            .requirements(vec![vec!["a".into()], vec!["b".into()]]);
        let json = adv.to_json();
        assert_eq!(json["parent"].as_str().unwrap(), "hello_world:root");
        let reqs = json["requirements"].as_array().unwrap();
        assert_eq!(reqs.len(), 2);
    }

    #[test]
    fn advancement_rewards_all_fields() {
        use sand_core::{Advancement, AdvancementRewards, AdvancementTrigger, Criterion};
        let adv = Advancement::new("hello_world:rewarded".parse().unwrap())
            .criterion("t", Criterion::new(AdvancementTrigger::Tick))
            .rewards(
                AdvancementRewards::new()
                    .recipe("hello_world:special_recipe")
                    .loot("hello_world:bonus_chest")
                    .experience(500)
                    .function("hello_world:on_complete"),
            );
        let json = adv.to_json();
        assert_eq!(json["rewards"]["experience"].as_i64().unwrap(), 500);
        assert_eq!(
            json["rewards"]["function"].as_str().unwrap(),
            "hello_world:on_complete"
        );
        assert_eq!(
            json["rewards"]["recipes"][0].as_str().unwrap(),
            "hello_world:special_recipe"
        );
        assert_eq!(
            json["rewards"]["loot"][0].as_str().unwrap(),
            "hello_world:bonus_chest"
        );
    }

    #[test]
    fn advancement_sends_telemetry_data() {
        use sand_core::{Advancement, AdvancementTrigger, Criterion};
        let adv = Advancement::new("hello_world:telemetry".parse().unwrap())
            .criterion("t", Criterion::new(AdvancementTrigger::Tick))
            .sends_telemetry_data(true);
        let json = adv.to_json();
        assert!(json["sends_telemetry_data"].as_bool().unwrap_or(false));
    }

    #[test]
    fn advancement_recipe_unlocked_trigger() {
        use sand_core::{Advancement, AdvancementTrigger, Criterion};
        let adv = Advancement::new("hello_world:recipe_test".parse().unwrap()).criterion(
            "unlocked",
            Criterion::new(AdvancementTrigger::RecipeUnlocked {
                recipe: "hello_world:diamond_sword".into(),
            }),
        );
        let json = adv.to_json();
        assert_eq!(
            json["criteria"]["unlocked"]["trigger"].as_str().unwrap(),
            "minecraft:recipe_unlocked"
        );
        assert_eq!(
            json["criteria"]["unlocked"]["conditions"]["recipe"]
                .as_str()
                .unwrap(),
            "hello_world:diamond_sword"
        );
    }

    #[test]
    fn advancement_custom_trigger_escape_hatch() {
        use sand_core::{Advancement, AdvancementTrigger, Criterion};
        let adv = Advancement::new("hello_world:mod_adv".parse().unwrap()).criterion(
            "mod_thing",
            Criterion::new(AdvancementTrigger::Custom {
                trigger: "mymod:do_thing".into(),
                conditions: Some(serde_json::json!({"count": 5})),
            }),
        );
        let json = adv.to_json();
        assert_eq!(
            json["criteria"]["mod_thing"]["trigger"].as_str().unwrap(),
            "mymod:do_thing"
        );
        assert_eq!(
            json["criteria"]["mod_thing"]["conditions"]["count"]
                .as_i64()
                .unwrap(),
            5
        );
    }

    // ── LootTable — extended coverage ─────────────────────────────────────────

    #[test]
    fn loot_table_bonus_rolls() {
        use sand_core::{LootEntry, LootPool, LootTable, NumberProvider};
        let table = LootTable::new("hello_world:bonus_test".parse().unwrap()).pool(
            LootPool::new()
                .rolls(1)
                .bonus_rolls(NumberProvider::Uniform { min: 0.0, max: 1.0 })
                .entry(LootEntry::item("minecraft:diamond")),
        );
        let json = table.to_json();
        let pool = &json["pools"][0];
        assert_eq!(
            pool["bonus_rolls"]["type"].as_str().unwrap(),
            "minecraft:uniform"
        );
    }

    #[test]
    fn loot_table_multiple_pools() {
        use sand_core::{LootEntry, LootPool, LootTable};
        let table = LootTable::new("hello_world:two_pools".parse().unwrap())
            .pool(
                LootPool::new()
                    .rolls(1)
                    .entry(LootEntry::item("minecraft:diamond")),
            )
            .pool(
                LootPool::new()
                    .rolls(2)
                    .entry(LootEntry::item("minecraft:gold_ingot")),
            );
        let json = table.to_json();
        assert_eq!(json["pools"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn loot_entry_types() {
        use sand_core::{LootEntry, LootPool, LootTable};
        let table = LootTable::new("hello_world:entry_types".parse().unwrap()).pool(
            LootPool::new()
                .rolls(1)
                .entry(LootEntry::tag("minecraft:logs"))
                .entry(LootEntry::loot_table("minecraft:chests/simple_dungeon"))
                .entry(LootEntry::empty()),
        );
        let json = table.to_json();
        let entries = json["pools"][0]["entries"].as_array().unwrap();
        assert_eq!(entries[0]["type"].as_str().unwrap(), "minecraft:tag");
        assert_eq!(entries[0]["name"].as_str().unwrap(), "minecraft:logs");
        assert_eq!(entries[1]["type"].as_str().unwrap(), "minecraft:loot_table");
        assert_eq!(entries[2]["type"].as_str().unwrap(), "minecraft:empty");
    }

    #[test]
    fn loot_condition_composed() {
        use sand_core::{LootCondition, LootEntry, LootPool, LootTable};
        let table = LootTable::new("hello_world:conditional".parse().unwrap()).pool(
            LootPool::new()
                .rolls(1)
                .entry(LootEntry::item("minecraft:diamond"))
                .condition(LootCondition::AllOf {
                    terms: vec![
                        LootCondition::KilledByPlayer,
                        LootCondition::RandomChance { chance: 0.5 },
                    ],
                }),
        );
        let json = table.to_json();
        let cond = &json["pools"][0]["conditions"][0];
        assert_eq!(cond["condition"].as_str().unwrap(), "minecraft:all_of");
        let terms = cond["terms"].as_array().unwrap();
        assert_eq!(terms.len(), 2);
        assert_eq!(
            terms[0]["condition"].as_str().unwrap(),
            "minecraft:killed_by_player"
        );
    }

    #[test]
    fn loot_condition_inverted() {
        use sand_core::{LootCondition, Predicate};
        let pred = Predicate::new(
            "hello_world:not_raining".parse().unwrap(),
            LootCondition::Inverted {
                term: Box::new(LootCondition::WeatherCheck {
                    raining: Some(true),
                    thundering: None,
                }),
            },
        );
        let json = pred.to_json();
        assert_eq!(json["condition"].as_str().unwrap(), "minecraft:inverted");
        assert_eq!(
            json["term"]["condition"].as_str().unwrap(),
            "minecraft:weather_check"
        );
        assert!(json["term"]["raining"].as_bool().unwrap_or(false));
    }

    #[test]
    fn loot_condition_custom_escape_hatch() {
        use sand_core::{LootCondition, Predicate};
        let pred = Predicate::new(
            "hello_world:mod_condition".parse().unwrap(),
            LootCondition::Custom {
                condition: "mymod:special_condition".into(),
                data: serde_json::json!({"level": 10}),
            },
        );
        let json = pred.to_json();
        assert_eq!(
            json["condition"].as_str().unwrap(),
            "mymod:special_condition"
        );
        assert_eq!(json["level"].as_i64().unwrap(), 10);
    }

    #[test]
    fn loot_function_set_name() {
        use sand_core::{ItemModifier, LootFunction};
        let modifier = ItemModifier::new("hello_world:named_item".parse().unwrap()).function(
            LootFunction::SetName {
                name: serde_json::json!({"text": "Legendary Sword", "color": "gold"}),
                entity: None,
            },
        );
        let json = modifier.to_json();
        assert_eq!(json["function"].as_str().unwrap(), "minecraft:set_name");
        assert_eq!(json["name"]["color"].as_str().unwrap(), "gold");
    }

    #[test]
    fn loot_function_custom_escape_hatch() {
        use sand_core::{ItemModifier, LootFunction};
        let modifier = ItemModifier::new("hello_world:mod_func".parse().unwrap()).function(
            LootFunction::Custom {
                function: "mymod:apply_buff".into(),
                data: serde_json::json!({"power": 5}),
            },
        );
        let json = modifier.to_json();
        assert_eq!(json["function"].as_str().unwrap(), "mymod:apply_buff");
        assert_eq!(json["power"].as_i64().unwrap(), 5);
    }

    #[test]
    fn number_provider_binomial() {
        use sand_core::{LootEntry, LootPool, LootTable, NumberProvider};
        let table = LootTable::new("hello_world:binomial_rolls".parse().unwrap()).pool(
            LootPool::new()
                .rolls(NumberProvider::Binomial { n: 5, p: 0.4 })
                .entry(LootEntry::item("minecraft:emerald")),
        );
        let json = table.to_json();
        let rolls = &json["pools"][0]["rolls"];
        assert_eq!(rolls["type"].as_str().unwrap(), "minecraft:binomial");
        assert_eq!(rolls["n"].as_i64().unwrap(), 5);
        assert!((rolls["p"].as_f64().unwrap() - 0.4).abs() < f64::EPSILON);
    }

    // ── Predicate — nested conditions ─────────────────────────────────────────

    #[test]
    fn predicate_any_of_nested() {
        use sand_core::{LootCondition, Predicate};
        let pred = Predicate::new(
            "hello_world:day_or_rain".parse().unwrap(),
            LootCondition::AnyOf {
                terms: vec![
                    LootCondition::TimeCheck {
                        value: serde_json::json!({"min": 0, "max": 12000}),
                        period: None,
                    },
                    LootCondition::WeatherCheck {
                        raining: Some(true),
                        thundering: None,
                    },
                ],
            },
        );
        let json = pred.to_json();
        assert_eq!(json["condition"].as_str().unwrap(), "minecraft:any_of");
        let terms = json["terms"].as_array().unwrap();
        assert_eq!(terms.len(), 2);
        assert_eq!(
            terms[1]["condition"].as_str().unwrap(),
            "minecraft:weather_check"
        );
    }

    // ── Struct literal construction ───────────────────────────────────────────

    #[test]
    fn advancement_struct_literal() {
        use sand_core::{Advancement, AdvancementTrigger, Criterion, DatapackComponent};
        use std::collections::HashMap;
        let mut criteria = HashMap::new();
        criteria.insert(
            "t".to_string(),
            Criterion {
                trigger: AdvancementTrigger::Tick,
            },
        );
        let adv = Advancement {
            location: "hello_world:manual_adv".parse().unwrap(),
            parent: None,
            display: None,
            criteria,
            requirements: None,
            rewards: None,
            sends_telemetry_data: false,
        };
        let json = adv.to_json();
        assert_eq!(
            adv.resource_location().to_string(),
            "hello_world:manual_adv"
        );
        assert_eq!(
            json["criteria"]["t"]["trigger"].as_str().unwrap(),
            "minecraft:tick"
        );
    }

    #[test]
    fn loot_table_struct_literal() {
        use sand_core::{DatapackComponent, LootTable};
        let table = LootTable {
            location: "hello_world:manual_table".parse().unwrap(),
            loot_type: None,
            random_sequence: None,
            pools: vec![],
            functions: vec![],
            conditions: vec![],
        };
        assert_eq!(
            table.resource_location().to_string(),
            "hello_world:manual_table"
        );
        assert!(table.to_json().as_object().unwrap().is_empty());
    }
}
