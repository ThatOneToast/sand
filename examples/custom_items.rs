//! # Custom items (1.21+)
//!
//! Demonstrates Minecraft 1.21's item component system for defining custom
//! items with food, tools, equipment, and other properties.

use sand_core::{
    AttributeModifier, AttributeOperation, AttributeType, ConsumableAnimation,
    ConsumableProperties, CustomItem, DyedColor, EquipmentSlot,
    EquipmentSlotGroup, EquippableProperties, FoodProperties, ItemRarity,
    ToolProperties, ToolRule,
};
use sand_macros::component;

// ── Custom food item ─────────────────────────────────────────────────────────
// Defines a food item with nutrition, saturation, and custom eating animation.

#[component]
pub fn magic_apple() -> CustomItem {
    CustomItem::new("my_pack:magic_apple".parse().unwrap(), "minecraft:apple")
        .custom_name(r#"{"text":"Magic Apple","color":"light_purple","italic":false}"#)
        .rarity(ItemRarity::Epic)
        .food(FoodProperties::new(8, 12.8).can_always_eat(true))
        .consumable(
            ConsumableProperties::new(1.0) // 1 second to eat
                .animation(ConsumableAnimation::Eat)
                .sound("minecraft:entity.player.burp"),
        )
        .enchantment_glint_override(true)
        .max_stack_size(16)
        .lore_line(r#"{"text":"A mystical fruit","color":"gray","italic":true}"#)
}

// ── Custom tool ──────────────────────────────────────────────────────────────
// A pickaxe with custom mining rules and durability.

#[component]
pub fn auto_smelting_pick() -> CustomItem {
    CustomItem::new(
        "my_pack:auto_smelting_pick".parse().unwrap(),
        "minecraft:netherite_pickaxe",
    )
    .custom_name(r#"{"text":"Smelter's Pick","color":"gold","italic":false}"#)
    .tool(
        ToolProperties::new()
            .rule(
                ToolRule::new(vec!["#minecraft:mineable/pickaxe".to_string()])
                    .speed(12.0)
                    .correct_for_drops(true),
            )
            .default_mining_speed(1.0)
            .damage_per_block(1),
    )
    .max_damage(3000)
    .repair_cost(5)
    .enchantment("minecraft:efficiency", 5)
    .enchantment("minecraft:unbreaking", 3)
    .attribute_modifier(AttributeModifier::new(
        AttributeType::AttackDamage,
        AttributeOperation::AddValue,
        10.0,
        EquipmentSlotGroup::Mainhand,
    ))
}

// ── Custom armor piece ───────────────────────────────────────────────────────
// Equipment with custom armor value, color, and attributes.

#[component]
pub fn royal_chestplate() -> CustomItem {
    CustomItem::new(
        "my_pack:royal_chestplate".parse().unwrap(),
        "minecraft:leather_chestplate",
    )
    .custom_name(r#"{"text":"Royal Chestplate","color":"gold","italic":false}"#)
    .rarity(ItemRarity::Rare)
    .equippable(
        EquippableProperties::new(EquipmentSlot::Chest)
            .equip_sound("minecraft:item.armor.equip_gold")
            .dispensable(true)
            .swappable(true)
            .damage_on_hurt(true),
    )
    .dyed_color(DyedColor::new(160, 0, 200)) // Royal purple
    .attribute_modifier(AttributeModifier::new(
        AttributeType::Armor,
        AttributeOperation::AddValue,
        12.0,
        EquipmentSlotGroup::Chest,
    ))
    .attribute_modifier(AttributeModifier::new(
        AttributeType::ArmorToughness,
        AttributeOperation::AddValue,
        4.0,
        EquipmentSlotGroup::Chest,
    ))
    .max_damage(800)
    .fire_resistant(true)
}

// ── Unbreakable utility item ─────────────────────────────────────────────────
// A non-stackable utility item with cooldown.

#[component]
pub fn teleport_wand() -> CustomItem {
    CustomItem::new(
        "my_pack:teleport_wand".parse().unwrap(),
        "minecraft:blaze_rod",
    )
    .custom_name(r#"{"text":"Teleport Wand","color":"aqua","italic":false}"#)
    .rarity(ItemRarity::Uncommon)
    .unbreakable(true)
    .max_stack_size(1)
    .enchantment_glint_override(true)
    .use_cooldown("my_pack:teleport", 5.0) // 5 second cooldown
    .lore_line(r#"{"text":"Right-click to teleport forward","color":"gray"}"#)
    .lore_line(r#"{"text":"5 second cooldown","color":"dark_gray"}"#)
}

// ── Item with stored enchantments ────────────────────────────────────────────
// An enchanted book with specific enchantments.

#[component]
pub fn super_book() -> CustomItem {
    CustomItem::new(
        "my_pack:super_book".parse().unwrap(),
        "minecraft:enchanted_book",
    )
    .custom_name(r#"{"text":"Book of Power","color":"yellow","italic":false}"#)
    .stored_enchantment("minecraft:sharpness", 10)
    .stored_enchantment("minecraft:fire_aspect", 5)
    .rarity(ItemRarity::Epic)
}
