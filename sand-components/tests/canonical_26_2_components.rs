//! Canonical Minecraft Java 26.2 datapack-component fixture (Phase 4c).
//!
//! Proves that `sand-components`' typed builders produce exact, valid JSON
//! for every currently supported registry/component family, by comparing
//! the full parsed [`serde_json::Value`] against an inline `json!{...}`
//! expected value (never `.contains`).
//!
//! Each test's doc comment cites the Minecraft Wiki page consulted while
//! writing it (checked 2026-07-19). Families that exist in the Minecraft
//! 26.2 datapack format but are **not** owned by this crate are noted at the
//! bottom of this file instead of being invented here:
//!
//! - `pack.mcmeta` — owned by `sand-core`/`sand-cli` (see
//!   `sand-cli/src/pack_format.rs`, `sand-cli/src/build/*`), not
//!   `sand-components`.
//! - Standalone "custom item registry" JSON files — Minecraft 26.2 has no
//!   such datapack registry; `sand-components` models item components as
//!   data embedded in recipe results / item stacks
//!   ([`CustomItem`]/[`ItemStack`]/[`RecipeResult::from_custom_item`]), which
//!   this file covers via [`custom_item_components`].

use std::collections::HashMap;

use sand_components::TagId;
use sand_components::dialog::{Dialog, DialogAction, DialogBody, DialogButton};
use sand_components::item::stack::ItemStack;
use sand_components::item_modifier::ItemModifier;
use sand_components::loot_table::{
    LootCondition, LootEntry, LootFunction, LootPool, LootTable, LootTableType, NumberProvider,
};
use sand_components::predicate::Predicate;
use sand_components::predicates::{DamagePredicate, EntityPredicate, FloatRange};
use sand_components::recipe::{
    CookingRecipe, CookingType, Ingredient, RecipeResult, ShapedRecipe, ShapelessRecipe,
    SmithingTransformRecipe, SmithingTrimRecipe, StonecuttingRecipe,
};
use sand_components::registry::{EnchantmentId, ItemId};
use sand_components::tag::{Tag, TagEntry, TypedTag};
use sand_components::worldgen::Biome;
use sand_components::worldgen::biome::BiomeEffects;
use sand_components::worldgen::placed_feature::PlacedFeature;
use sand_components::{
    Advancement, AdvancementDisplay, AdvancementFrame, AdvancementIcon, AdvancementRewards,
    AdvancementTrigger, AttributeModifier, AttributeOperation, AttributeType, ComponentContent,
    Criterion, CustomItem, DatapackComponent, Enchantment, EnchantmentCost, EnchantmentEntry,
    EquipmentSlot, EquipmentSlotGroup, EquippableProperties, FoodProperties, ItemComponent,
    ResourceLocation,
};

fn id(path: &str) -> ResourceLocation {
    ResourceLocation::new("canon", path).unwrap()
}

fn content_json(component: &dyn DatapackComponent) -> serde_json::Value {
    match component.try_content().expect("component should validate") {
        ComponentContent::Json(v) => v,
        ComponentContent::Text(_) => panic!("expected JSON content"),
    }
}

// ── Advancements ────────────────────────────────────────────────────────────
// Wiki: "Advancement definition" (https://minecraft.wiki/w/Advancement_definition), checked 2026-07-19.

/// Full advancement: display (icon/title/description/background/frame),
/// a criterion combining an entity kill-predicate trigger with a damage
/// sub-predicate, a parent, and rewards (recipes/loot/experience/function).
#[test]
fn canonical_26_2_advancement_display_icon_criteria_rewards_parent() {
    let build = || {
        Advancement::new(id("boss/dragon_slayer"))
            .parent("canon:boss/root")
            .display(
                AdvancementDisplay::new(
                    AdvancementIcon::new("minecraft:dragon_head"),
                    "Dragon Slayer",
                    "Defeat the Ender Dragon",
                )
                .frame(AdvancementFrame::Challenge)
                .background("minecraft:textures/block/end_stone.png")
                .show_toast(true)
                .announce_to_chat(true)
                .hidden(false),
            )
            .criterion(
                "killed_dragon",
                Criterion::new(AdvancementTrigger::PlayerKilledEntity {
                    entity: Some(EntityPredicate::type_("minecraft:ender_dragon")),
                    killing_blow: Some(DamagePredicate::new().dealt(FloatRange::at_least(1.0))),
                }),
            )
            .rewards(
                AdvancementRewards::new()
                    .experience(1000)
                    .loot("canon:boss/dragon_reward")
                    .recipe("canon:elytra_repair")
                    .function("canon:on_dragon_slain"),
            )
    };

    let expected = serde_json::json!({
        "parent": "canon:boss/root",
        "display": {
            "icon": {"id": "minecraft:dragon_head"},
            "title": "Dragon Slayer",
            "description": "Defeat the Ender Dragon",
            "background": "minecraft:textures/block/end_stone.png",
            "frame": "challenge",
            "show_toast": true,
            "announce_to_chat": true,
            "hidden": false
        },
        "criteria": {
            "killed_dragon": {
                "trigger": "minecraft:player_killed_entity",
                "conditions": {
                    "entity": [
                        {
                            "condition": "minecraft:entity_properties",
                            "entity": "this",
                            "predicate": {"minecraft:entity_type": "minecraft:ender_dragon"}
                        }
                    ],
                    "killing_blow": {"dealt": {"min": 1.0}}
                }
            }
        },
        "requirements": [["killed_dragon"]],
        "rewards": {
            "recipes": ["canon:elytra_repair"],
            "loot": ["canon:boss/dragon_reward"],
            "experience": 1000,
            "function": "canon:on_dragon_slain"
        }
    });

    let adv = build();
    assert_eq!(adv.to_json(), expected);
    assert_eq!(content_json(&adv), expected);

    // Deterministic key ordering: two independently-built advancements must
    // serialize byte-identically (sand-components' JSON output routes
    // through serde_json::Value/Map, which sorts keys; this asserts the
    // promise holds for a real multi-field advancement).
    let first = serde_json::to_string(&build().to_json()).unwrap();
    let second = serde_json::to_string(&build().to_json()).unwrap();
    assert_eq!(first, second);
}

/// Minecraft 26.2 namespaces advancement entity sub-predicate keys
/// (`minecraft:entity_type`) while 1.21.4 uses the historical unnamespaced
/// `type` key inside a `minecraft:entity_properties` loot condition — a
/// genuine `VersionCaps`-gated rendering branch
/// (`AdvancementSchemaFamily::{LocationConditionItemComponents,NamespacedEntityPredicates}`
/// in `sand-components/src/advancement/mod.rs`).
///
/// Wiki: "Advancement definition" (https://minecraft.wiki/w/Advancement_definition), checked 2026-07-19.
#[test]
fn compat_1_21_4_advancement_entity_predicate_schema_is_unnamespaced() {
    let trigger = AdvancementTrigger::PlayerKilledEntity {
        entity: Some(EntityPredicate::type_("minecraft:ender_dragon")),
        killing_blow: None,
    };

    let caps_1_21_4 = sand_version::VersionCaps::from_profile_flags(
        "1.21.4", false, false, true, true, true, true, true, true,
    );

    let legacy = trigger.render_for(Some(&caps_1_21_4)).unwrap();
    assert_eq!(
        legacy,
        serde_json::json!({
            "trigger": "minecraft:player_killed_entity",
            "conditions": {
                "entity": [
                    {
                        "condition": "minecraft:entity_properties",
                        "entity": "this",
                        "predicate": {"type": "minecraft:ender_dragon"}
                    }
                ]
            }
        })
    );

    // The unprofiled/26.2 compatibility path namespaces the same key.
    let modern = trigger.render_for(None).unwrap();
    assert_eq!(
        modern["conditions"]["entity"][0]["predicate"]["minecraft:entity_type"],
        "minecraft:ender_dragon"
    );
    assert!(
        modern["conditions"]["entity"][0]["predicate"]
            .get("type")
            .is_none()
    );
}

// ── Predicates ──────────────────────────────────────────────────────────────
// Wiki: "Predicate" (https://minecraft.wiki/w/Predicate), checked 2026-07-19.

/// A standalone predicate file composing boolean logic (`all_of`/`inverted`),
/// probability, entity-score, and weather `LootCondition` variants.
#[test]
fn canonical_26_2_predicate_loot_condition_composition() {
    let mut scores = HashMap::new();
    scores.insert("looting".to_string(), serde_json::json!({"min": 1}));

    let predicate = Predicate::new(
        id("conditions/dragon_drop"),
        LootCondition::AllOf {
            terms: vec![
                LootCondition::RandomChance { chance: 0.25 },
                LootCondition::KilledByPlayer,
                LootCondition::Inverted {
                    term: Box::new(LootCondition::SurvivesExplosion),
                },
                LootCondition::EntityScores {
                    entity: "this".to_string(),
                    scores,
                },
                LootCondition::WeatherCheck {
                    raining: Some(true),
                    thundering: None,
                },
            ],
        },
    );

    let expected = serde_json::json!({
        "condition": "minecraft:all_of",
        "terms": [
            {"condition": "minecraft:random_chance", "chance": 0.25},
            {"condition": "minecraft:killed_by_player"},
            {"condition": "minecraft:inverted", "term": {"condition": "minecraft:survives_explosion"}},
            {"condition": "minecraft:entity_scores", "entity": "this", "scores": {"looting": {"min": 1}}},
            {"condition": "minecraft:weather_check", "raining": true}
        ]
    });

    assert_eq!(predicate.to_json(), expected);
    assert_eq!(content_json(&predicate), expected);
    assert_eq!(
        predicate.resource_location().to_string(),
        "canon:conditions/dragon_drop"
    );
    assert_eq!(predicate.component_dir(), "predicate");
}

// ── Recipes ─────────────────────────────────────────────────────────────────
// Wiki: "Recipe" (https://minecraft.wiki/w/Recipe), checked 2026-07-19.

/// Shaped crafting recipe: 3x3 pattern with an item key, a tag key, category,
/// group, and a disabled unlock notification.
#[test]
fn canonical_26_2_recipe_shaped() {
    let shaped = ShapedRecipe::new(id("relics/phoenix_feather"))
        .pattern(["FGF", "GPG", "FGF"])
        .key('F', Ingredient::item("minecraft:feather"))
        .key('G', Ingredient::tag("minecraft:logs"))
        .key('P', Ingredient::item("minecraft:blaze_powder"))
        .category("misc")
        .group("phoenix")
        .show_notification(false)
        .result(RecipeResult::item(
            ItemId::minecraft("firework_star").unwrap(),
            1,
        ));

    let expected = serde_json::json!({
        "type": "minecraft:crafting_shaped",
        "category": "misc",
        "group": "phoenix",
        "pattern": ["FGF", "GPG", "FGF"],
        "key": {
            "F": "minecraft:feather",
            "G": "#minecraft:logs",
            "P": "minecraft:blaze_powder"
        },
        "result": {"id": "minecraft:firework_star", "count": 1},
        "show_notification": false
    });

    assert_eq!(shaped.to_json(), expected);
    assert_eq!(content_json(&shaped), expected);
    assert_eq!(shaped.component_dir(), "recipe");

    // Deterministic serialization: two independently-built recipes match byte-for-byte.
    let a = serde_json::to_string(&shaped.to_json()).unwrap();
    let b = serde_json::to_string(
        &ShapedRecipe::new(id("relics/phoenix_feather"))
            .pattern(["FGF", "GPG", "FGF"])
            .key('F', Ingredient::item("minecraft:feather"))
            .key('G', Ingredient::tag("minecraft:logs"))
            .key('P', Ingredient::item("minecraft:blaze_powder"))
            .category("misc")
            .group("phoenix")
            .show_notification(false)
            .result(RecipeResult::item(
                ItemId::minecraft("firework_star").unwrap(),
                1,
            ))
            .to_json(),
    )
    .unwrap();
    assert_eq!(a, b);
}

/// Shapeless crafting recipe with mixed item/alternatives ingredients.
#[test]
fn canonical_26_2_recipe_shapeless() {
    let shapeless = ShapelessRecipe::new(id("elixirs/regeneration"))
        .ingredient(Ingredient::item("minecraft:golden_carrot"))
        .ingredient(Ingredient::alternatives([
            Ingredient::item("minecraft:ghast_tear"),
            Ingredient::item("minecraft:phantom_membrane"),
        ]))
        .category("misc")
        .result(RecipeResult::item(ItemId::minecraft("potion").unwrap(), 1));

    let expected = serde_json::json!({
        "type": "minecraft:crafting_shapeless",
        "category": "misc",
        "ingredients": [
            "minecraft:golden_carrot",
            ["minecraft:ghast_tear", "minecraft:phantom_membrane"]
        ],
        "result": {"id": "minecraft:potion", "count": 1}
    });

    assert_eq!(shapeless.to_json(), expected);
    assert_eq!(content_json(&shapeless), expected);
}

/// Cooking (smelting) recipe with category, group, experience, and cook time.
#[test]
fn canonical_26_2_recipe_cooking_smelting() {
    let smelting = CookingRecipe::new(id("smelting/dragon_scale"), CookingType::Smelting)
        .ingredient(Ingredient::item("minecraft:phantom_membrane"))
        .result(RecipeResult::item(ItemId::minecraft("scute").unwrap(), 1))
        .category("misc")
        .group("dragon")
        .experience(0.35)
        .cooking_time(400);

    let expected = serde_json::json!({
        "type": "minecraft:smelting",
        "category": "misc",
        "group": "dragon",
        "ingredient": "minecraft:phantom_membrane",
        "result": {"id": "minecraft:scute", "count": 1},
        // `experience` is stored as f32 internally; 0.35f32 round-trips
        // through f64 JSON as 0.3499999940395355, not the shorter literal.
        "experience": 0.3499999940395355,
        "cookingtime": 400
    });

    assert_eq!(smelting.to_json(), expected);
    assert_eq!(content_json(&smelting), expected);
}

/// Stonecutting recipe.
#[test]
fn canonical_26_2_recipe_stonecutting() {
    let stonecutting = StonecuttingRecipe::new(id("blocks/polished_endstone"))
        .ingredient(Ingredient::item("minecraft:end_stone"))
        .result(RecipeResult::item(
            ItemId::minecraft("end_stone_bricks").unwrap(),
            1,
        ))
        .count(1)
        .group("endstone");

    let expected = serde_json::json!({
        "type": "minecraft:stonecutting",
        "group": "endstone",
        "ingredient": "minecraft:end_stone",
        "result": {"id": "minecraft:end_stone_bricks", "count": 1},
        "count": 1
    });

    assert_eq!(stonecutting.to_json(), expected);
    assert_eq!(content_json(&stonecutting), expected);
}

/// Smithing transform recipe (template/base/addition -> result), the
/// currently-supported smithing variant alongside smithing trim.
#[test]
fn canonical_26_2_recipe_smithing_transform() {
    let transform = SmithingTransformRecipe::new(id("smithing/netherite_relic"))
        .template(Ingredient::item(
            "minecraft:netherite_upgrade_smithing_template",
        ))
        .base(Ingredient::item("minecraft:diamond_sword"))
        .addition(Ingredient::item("minecraft:netherite_ingot"))
        .group("netherite")
        .result(RecipeResult::item(
            ItemId::minecraft("netherite_sword").unwrap(),
            1,
        ));

    let expected = serde_json::json!({
        "type": "minecraft:smithing_transform",
        "group": "netherite",
        "template": "minecraft:netherite_upgrade_smithing_template",
        "base": "minecraft:diamond_sword",
        "addition": "minecraft:netherite_ingot",
        "result": {"id": "minecraft:netherite_sword", "count": 1}
    });

    assert_eq!(transform.to_json(), expected);
    assert_eq!(content_json(&transform), expected);
}

/// Smithing trim recipe (no `result` field — trims are applied in-place).
#[test]
fn canonical_26_2_recipe_smithing_trim() {
    let trim = SmithingTrimRecipe::new(id("smithing/sentry_trim"))
        .template(Ingredient::item(
            "minecraft:sentry_armor_trim_smithing_template",
        ))
        .base(Ingredient::item("minecraft:diamond_chestplate"))
        .addition(Ingredient::item("minecraft:amethyst_shard"));

    let expected = serde_json::json!({
        "type": "minecraft:smithing_trim",
        "template": "minecraft:sentry_armor_trim_smithing_template",
        "base": "minecraft:diamond_chestplate",
        "addition": "minecraft:amethyst_shard"
    });

    assert_eq!(trim.to_json(), expected);
    assert_eq!(content_json(&trim), expected);
}

// ── Loot tables ─────────────────────────────────────────────────────────────
// Wiki: "Loot table" (https://minecraft.wiki/w/Loot_table), checked 2026-07-19.

/// Chest loot table: pool composition (rolls/bonus_rolls), an `alternatives`
/// entry nesting a conditioned+functioned item entry and a plain item entry,
/// a pool-level condition, and a pool-level function.
#[test]
fn canonical_26_2_loot_table_pool_entry_function_condition_composition() {
    let build = || {
        LootTable::new(id("chests/dragon_hoard"))
            .loot_type(LootTableType::Chest)
            .random_sequence("canon:chests/dragon_hoard")
            .pool(
                LootPool::new()
                    .rolls(NumberProvider::Uniform { min: 1.0, max: 3.0 })
                    .bonus_rolls(NumberProvider::Constant(1.0))
                    .entry(LootEntry::Alternatives {
                        children: vec![
                            LootEntry::Item {
                                name: "minecraft:diamond".to_string(),
                                weight: Some(5),
                                quality: None,
                                functions: vec![LootFunction::SetCount {
                                    count: NumberProvider::Uniform { min: 1.0, max: 3.0 },
                                    add: false,
                                }],
                                conditions: vec![LootCondition::RandomChance { chance: 0.5 }],
                            },
                            LootEntry::item("minecraft:iron_ingot"),
                        ],
                        conditions: vec![],
                    })
                    .condition(LootCondition::KilledByPlayer)
                    .function(LootFunction::EnchantWithLevels {
                        levels: NumberProvider::Constant(30.0),
                        options: Some("#minecraft:on_random_loot".to_string()),
                    }),
            )
    };

    let expected = serde_json::json!({
        "type": "minecraft:chest",
        "random_sequence": "canon:chests/dragon_hoard",
        "pools": [
            {
                "rolls": {"type": "minecraft:uniform", "min": 1.0, "max": 3.0},
                "bonus_rolls": 1.0,
                "entries": [
                    {
                        "type": "minecraft:alternatives",
                        "children": [
                            {
                                "type": "minecraft:item",
                                "name": "minecraft:diamond",
                                "weight": 5,
                                "functions": [
                                    {
                                        "function": "minecraft:set_count",
                                        "count": {"type": "minecraft:uniform", "min": 1.0, "max": 3.0},
                                        "add": false
                                    }
                                ],
                                "conditions": [
                                    {"condition": "minecraft:random_chance", "chance": 0.5}
                                ]
                            },
                            {"type": "minecraft:item", "name": "minecraft:iron_ingot"}
                        ]
                    }
                ],
                "conditions": [
                    {"condition": "minecraft:killed_by_player"}
                ],
                "functions": [
                    {
                        "function": "minecraft:enchant_with_levels",
                        "levels": 30.0,
                        "options": "#minecraft:on_random_loot"
                    }
                ]
            }
        ]
    });

    let table = build();
    assert_eq!(table.to_json(), expected);
    assert_eq!(content_json(&table), expected);
    assert_eq!(table.component_dir(), "loot_table");

    // Deterministic key ordering: two independently-built loot tables with
    // the same nested pool/entry/function/condition composition must
    // serialize byte-identically.
    let first = serde_json::to_string(&build().to_json()).unwrap();
    let second = serde_json::to_string(&build().to_json()).unwrap();
    assert_eq!(first, second);
}

// ── Item modifiers ──────────────────────────────────────────────────────────
// Wiki: "Item modifier" (https://minecraft.wiki/w/Item_modifier), checked 2026-07-19.

/// Multi-function item modifier (looting-enchant bonus followed by explosion decay).
#[test]
fn canonical_26_2_item_modifier_multiple_functions() {
    let modifier = ItemModifier::new(id("modifiers/dragon_scale_bonus"))
        .function(LootFunction::LootingEnchant {
            count: NumberProvider::Uniform { min: 0.0, max: 2.0 },
            limit: Some(5),
        })
        .function(LootFunction::ExplosionDecay);

    let expected = serde_json::json!([
        {
            "function": "minecraft:looting_enchant",
            "count": {"type": "minecraft:uniform", "min": 0.0, "max": 2.0},
            "limit": 5
        },
        {"function": "minecraft:explosion_decay"}
    ]);

    assert_eq!(modifier.to_json(), expected);
    assert_eq!(content_json(&modifier), expected);
    assert_eq!(modifier.component_dir(), "item_modifier");
}

// ── Tags ────────────────────────────────────────────────────────────────────
// Wiki: "Tag (Java Edition)" (https://minecraft.wiki/w/Tag_(Java_Edition)), checked 2026-07-19.

/// Function tag (legacy [`Tag`] builder, still the correct API for function
/// tags — [`TypedTag`] is scoped to item/block/entity_type registries).
#[test]
fn canonical_26_2_function_tag() {
    let tag = Tag::new(id("function/deploy_all"))
        .entry("canon:deploy/step_one")
        .entry("canon:deploy/step_two")
        .tag_ref("canon:deploy/extensions");

    let expected = serde_json::json!({
        "replace": false,
        "values": [
            "canon:deploy/step_one",
            "canon:deploy/step_two",
            "#canon:deploy/extensions"
        ]
    });

    assert_eq!(tag.to_json(), expected);
    assert_eq!(tag.component_dir(), "tags");
}

/// Item tag ([`TypedTag`]) with required entries, an optional entry, and a
/// required tag reference.
#[test]
fn canonical_26_2_item_tag() {
    let tag = TypedTag::new(TagId::<ItemId>::minecraft("dragon_materials").unwrap())
        .entry(ItemId::minecraft("dragon_breath").unwrap())
        .optional_entry(ItemId::minecraft("elytra").unwrap())
        .with_entry(TagEntry::value(
            ItemId::minecraft("phantom_membrane").unwrap(),
        ));

    let expected = serde_json::json!({
        "replace": false,
        "values": [
            "minecraft:dragon_breath",
            {"id": "minecraft:elytra", "required": false},
            "minecraft:phantom_membrane"
        ]
    });

    assert_eq!(tag.to_json(), expected);
    assert_eq!(tag.component_dir(), "tags/item");
}

// ── Custom items with item components ───────────────────────────────────────
// Wiki: "Data component format" (https://minecraft.wiki/w/Data_component_format), checked 2026-07-19.

/// A `CustomItem` carrying attributes, food, enchantments, custom_data, and
/// equippable components, converted into a component-bearing recipe result
/// via [`RecipeResult::from_custom_item`] — the JSON shape these components
/// actually take on disk (Minecraft 26.2 has no standalone custom-item
/// registry file; components are embedded in recipe results / item stacks).
#[test]
fn custom_item_components_attributes_food_enchantments_custom_data_equippable() {
    let item = CustomItem::new("minecraft:golden_apple")
        .custom_data("elevator")
        .typed_enchantment(EnchantmentId::minecraft("fire_aspect").unwrap(), 2)
        .food(FoodProperties::new(4, 0.3).can_always_eat(true))
        .attribute_modifier(
            AttributeModifier::new(AttributeType::MaxHealth)
                .amount(4.0)
                .operation(AttributeOperation::AddValue)
                .slot(EquipmentSlotGroup::Any),
        )
        .equippable(EquippableProperties::new(EquipmentSlot::Head).dispensable(false));

    let result =
        RecipeResult::from_custom_item(&item, 1).expect("all components are representable");

    let expected = serde_json::json!({
        "id": "minecraft:golden_apple",
        "count": 1,
        "components": {
            "minecraft:attribute_modifiers": [
                {
                    "id": "minecraft:max_health",
                    "type": "minecraft:max_health",
                    "amount": 4.0,
                    "operation": "add_value",
                    "slot": "any"
                }
            ],
            "minecraft:custom_data": {"elevator": true},
            "minecraft:enchantments": {"minecraft:fire_aspect": 2},
            "minecraft:equippable": {
                "slot": "head",
                "dispensable": false,
                "swappable": true,
                "damage_on_hurt": true
            },
            "minecraft:food": {
                "nutrition": 4,
                // f32 round-trip, see the smelting-recipe test's experience field.
                "saturation": 0.30000001192092896,
                "can_always_eat": true
            }
        }
    });

    assert_eq!(serde_json::to_value(&result).unwrap(), expected);

    // The same identity survives an ItemStack round trip (the concrete-stack
    // sibling of the recipe-result view above).
    let stack = ItemStack::new(ItemId::minecraft("golden_apple").unwrap())
        .component(ItemComponent::custom_data_marker("elevator"))
        .component(ItemComponent::enchantment(
            EnchantmentId::minecraft("fire_aspect").unwrap(),
            2,
        ));
    let components = stack.stack_components().unwrap();
    assert_eq!(components.base_item(), "minecraft:golden_apple");
    assert!(
        components
            .components()
            .iter()
            .any(|(k, _)| k == "minecraft:custom_data")
    );

    // Sanity: the standalone typed builder used above is the same one
    // sand-components documents for enchantment entries.
    let _ = EnchantmentEntry::new(EnchantmentId::minecraft("fire_aspect").unwrap(), 2);
}

// ── Dialogs ─────────────────────────────────────────────────────────────────
// Wiki: "Dialog" (https://minecraft.wiki/w/Dialog), checked 2026-07-19.

/// Notice dialog with title, a text body, and a button with an action,
/// tooltip, and explicit width.
#[test]
fn canonical_26_2_dialog_notice_with_button_action() {
    let dialog = Dialog::notice("canon:welcome")
        .title("Welcome!")
        .body(DialogBody::text("Choose an option."))
        .button(
            DialogButton::new("Start")
                .action(DialogAction::run_function(id("start")))
                .tooltip("Begin your journey")
                .width(150),
        )
        .pause(true)
        .external_title(true);

    let expected = serde_json::json!({
        "type": "minecraft:notice",
        "title": {"text": "Welcome!"},
        "body": [
            {"type": "minecraft:plain_message", "contents": {"text": "Choose an option."}}
        ],
        "buttons": [
            {
                "label": {"text": "Start"},
                "action": {"type": "minecraft:run_command", "command": "/function canon:start"},
                "tooltip": {"text": "Begin your journey"},
                "width": 150
            }
        ],
        "pause": true,
        "external_title": true
    });

    assert_eq!(dialog.to_json(), expected);
    assert_eq!(content_json(&dialog), expected);
    assert_eq!(dialog.resource_path(), "canon/dialog/welcome.json");
    assert_eq!(dialog.component_dir(), "dialog");
    assert_eq!(
        dialog.required_features(),
        &[sand_version::ComponentFeature::Dialogs]
    );
}

// ── Enchantments ────────────────────────────────────────────────────────────
// Wiki: "Enchantment definition" (https://minecraft.wiki/w/Enchantment_definition), checked 2026-07-19.

/// Enchantment definition with description, supported/primary items,
/// exclusivity, weight/level/cost tuning, slots, and raw effects.
#[test]
fn canonical_26_2_enchantment_definition() {
    let enchantment = Enchantment::new(id("fire_walker_plus"))
        .description_translate("enchantment.canon.fire_walker_plus")
        .supported_items("#minecraft:enchantable/foot_armor")
        .primary_items("minecraft:diamond_boots")
        .exclusive_set("#minecraft:exclusive_set/boots")
        .weight(5)
        .max_level(3)
        .min_cost(EnchantmentCost::new(10, 8))
        .max_cost(EnchantmentCost::new(40, 8))
        .anvil_cost(4)
        .slots(["feet"])
        .effects_raw(serde_json::json!({}));

    let expected = serde_json::json!({
        "description": {"translate": "enchantment.canon.fire_walker_plus"},
        "supported_items": "#minecraft:enchantable/foot_armor",
        "primary_items": "minecraft:diamond_boots",
        "exclusive_set": "#minecraft:exclusive_set/boots",
        "weight": 5,
        "max_level": 3,
        "min_cost": {"base": 10, "per_level_above_first": 8},
        "max_cost": {"base": 40, "per_level_above_first": 8},
        "anvil_cost": 4,
        "slots": ["feet"],
        "effects": {}
    });

    assert_eq!(enchantment.to_json(), expected);
    assert_eq!(enchantment.component_dir(), "enchantment");
    assert_eq!(
        enchantment.required_features(),
        &[sand_version::ComponentFeature::Enchantments]
    );
}

// ── Worldgen ────────────────────────────────────────────────────────────────
// Wiki: "Biome definition" (https://minecraft.wiki/w/Biome_definition), checked 2026-07-19.

/// Biome with required colors, grass/foliage overrides, an ambient sound, and
/// non-default precipitation/temperature/downfall.
#[test]
fn canonical_26_2_worldgen_biome() {
    let biome = Biome::new(
        id("worldgen/biome/scorched_wastes"),
        BiomeEffects::new(0xC0D8FF, 0x3F76E4, 0x050533, 0x78A7FF)
            .grass_color(0x8A_B689)
            .foliage_color(0x71_A74D)
            .ambient_sound("minecraft:ambient.nether_wastes.loop"),
    )
    .has_precipitation(false)
    .temperature(2.0)
    .temperature_modifier("none")
    .downfall(0.0);

    let expected = serde_json::json!({
        "has_precipitation": false,
        "temperature": 2.0,
        "temperature_modifier": "none",
        "downfall": 0.0,
        "effects": {
            "fog_color": 0xC0D8FF,
            "water_color": 0x3F76E4,
            "water_fog_color": 0x050533,
            "sky_color": 0x78A7FF,
            "grass_color": 0x8A_B689,
            "foliage_color": 0x71_A74D,
            "ambient_sound": "minecraft:ambient.nether_wastes.loop"
        }
    });

    assert_eq!(biome.to_json(), expected);
    assert_eq!(biome.component_dir(), "worldgen/biome");
}

/// Wiki: "Placed feature" (https://minecraft.wiki/w/Placed_feature), checked 2026-07-19.
#[test]
fn canonical_26_2_worldgen_placed_feature() {
    let feature = PlacedFeature::new(
        id("worldgen/placed_feature/scorched_shrub"),
        "canon:scorched_shrub",
    )
    .placement_modifier(serde_json::json!({"type": "minecraft:count", "count": 4}))
    .placement_modifier(serde_json::json!({"type": "minecraft:square"}));

    let expected = serde_json::json!({
        "feature": "canon:scorched_shrub",
        "placement": [
            {"type": "minecraft:count", "count": 4},
            {"type": "minecraft:square"}
        ]
    });

    assert_eq!(feature.to_json(), expected);
    assert_eq!(feature.component_dir(), "worldgen/placed_feature");
}
