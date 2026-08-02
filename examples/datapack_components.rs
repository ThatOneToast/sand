//! Typed datapack components.

use sand_core::prelude::*;
use sand_core::sand_components::worldgen::providers::{BlockState, BlockStateProvider};
use sand_macros::component;

#[component]
pub fn starter_dialog() -> Dialog {
    Dialog::notice_local("starter")
        .title(Text::new("Starter Kit").gold())
        .body(DialogBody::text(Text::new(
            "Your datapack JSON came from typed Rust.",
        )))
}

#[component]
pub fn starter_item() -> CustomItem {
    CustomItem::new("minecraft:stick")
        .id("example:dash_wand")
        .component(ItemComponent::custom_name(
            Text::new("Dash Wand").aqua().bold(true),
        ))
        .component(ItemComponent::lore(vec![
            Text::new("Right click to dash").gray(),
            Text::new("Consumes mana").dark_gray(),
        ]))
        .component(ItemComponent::EnchantmentGlintOverride(true))
        .component(ItemComponent::max_stack_size(1))
}

/// A complete vanilla-like custom dimension type with typed registry references.
#[component]
pub fn bright_overworld() -> DimensionType {
    DimensionType::overworld_like(
        ResourceLocation::new("example", "bright_overworld").expect("static ID is valid"),
    )
    .ambient_light(0.25)
    .monster_spawn_light_level(MonsterSpawnLightLevel::Constant(7))
}

/// A minimal typed configured feature: a single simple-block feature that
/// places one block state.
#[component]
pub fn ashen_shrub_feature() -> ConfiguredFeature {
    ConfiguredFeature::simple_block(
        ResourceLocation::new("example", "ashen_shrub").expect("static ID is valid"),
        BlockStateProvider::simple(BlockState::new(
            BlockId::minecraft("fern").expect("built-in block ID is valid"),
        )),
    )
}

/// A placed feature referencing the typed configured feature above.
#[component]
pub fn ashen_shrub_placement() -> PlacedFeature {
    PlacedFeature::new(
        ResourceLocation::new("example", "ashen_shrub").expect("static ID is valid"),
        ashen_shrub_feature().id(),
    )
    .placement_modifier(serde_json::json!({ "type": "minecraft:count", "count": 3 }))
}

#[component]
pub fn quartz_trim_material() -> TrimMaterial {
    TrimMaterial::new(ResourceLocation::new("example", "quartz").unwrap())
        .asset_name(TrimAssetName::new("quartz").unwrap())
        .ingredient(ItemId::minecraft("quartz").unwrap())
        .item_model_index(0.1)
        .description(TextComponent::translate(
            "trim_material.example.quartz",
        ))
}

#[component]
pub fn bolt_trim_pattern() -> TrimPattern {
    TrimPattern::new(ResourceLocation::new("example", "bolt").unwrap())
        .asset_id(ResourceLocation::new("example", "bolt").unwrap())
        .template_item(ItemId::minecraft("bolt_armor_trim_smithing_template").unwrap())
        .description(TextComponent::translate("trim_pattern.example.bolt"))
}

/// A typed 1.21+ enchantment: typed description, item-tag references, a
/// typed active slot, and a typed `minecraft:knockback` value effect.
#[component]
pub fn swift_step_enchantment() -> Enchantment {
    Enchantment::new(ResourceLocation::new("example", "swift_step").unwrap())
        .description(TextComponent::translate("enchantment.example.swift_step"))
        .supported_items(TagId::<ItemId>::minecraft("enchantable/foot_armor").unwrap())
        .exclusive_set(TagId::<EnchantmentId>::minecraft("exclusive_set/boots").unwrap())
        .slot(EquipmentSlotGroup::Feet)
        .knockback_effect(
            EnchantmentValueOperation::Add,
            LevelBasedValue::Linear {
                base: 0.5,
                per_level_above_first: 0.25,
            },
        )
}

#[component]
pub fn mob_enchantments() -> EnchantmentProvider {
    EnchantmentProvider::by_cost_with_difficulty(
        ResourceLocation::new("example", "mob_enchantments").unwrap(),
        TagId::<EnchantmentId>::minecraft("on_mob_spawn_equipment").unwrap(),
        5,
        17,
    )
}
