//! # Loot tables
//!
//! Demonstrates loot tables with pools, conditions, functions, and typed
//! item/tag/loot-table-reference/enchantment IDs.

use sand_core::prelude::Text;
use sand_core::{
    EnchantmentId, EnchantmentSelector, ItemId, LootCondition, LootEntry, LootFunction, LootPool,
    LootTable, LootTableId, LootTableType, NumberProvider, TagId,
};
use sand_macros::datapack_component;

// ── Simple entity drop ───────────────────────────────────────────────────────
// Uses the convenience constructor for common entity loot patterns.

#[datapack_component]
pub fn zombie_drops() -> LootTable {
    LootTable::entity_drop(
        "my_pack:entities/zombie".parse().unwrap(),
        "minecraft:rotten_flesh",
        0,
        2,
        Some(1),
    )
}

// ── Chest loot ───────────────────────────────────────────────────────────────
// Typed item entries, an item-tag entry, and a nested loot table reference.

#[datapack_component]
pub fn dungeon_chest() -> LootTable {
    LootTable::new("my_pack:chests/dungeon".parse().unwrap())
        .loot_type(LootTableType::Chest)
        .pool(
            LootPool::new()
                .rolls(NumberProvider::Uniform { min: 2.0, max: 4.0 })
                .entry(LootEntry::item(ItemId::minecraft("iron_ingot").unwrap()))
                .entry(LootEntry::item(ItemId::minecraft("gold_ingot").unwrap()))
                .entry(LootEntry::tag(TagId::minecraft("logs").unwrap()))
                .entry(LootEntry::loot_table(LootTableId::custom(
                    "my_pack:chests/bonus".parse().unwrap(),
                ))),
        )
        .pool(
            // One guaranteed enchanted book, randomly enchanted from a tag.
            LootPool::new().rolls(1).entry(LootEntry::Item {
                name: ItemId::minecraft("book").unwrap().to_string(),
                weight: None,
                quality: None,
                functions: vec![LootFunction::EnchantRandomly {
                    options: Some(vec![EnchantmentSelector::Tag(
                        TagId::minecraft("in_enchanting_table").unwrap(),
                    )]),
                    only_compatible: true,
                }],
                conditions: Vec::new(),
            }),
        )
}

// ── Full loot table with conditions and typed set_name text ─────────────────
// Demonstrates manual construction with conditions and functions.

#[datapack_component]
pub fn boss_loot() -> LootTable {
    LootTable::new("my_pack:entities/boss".parse().unwrap())
        .loot_type(LootTableType::Entity)
        .pool(
            LootPool::new()
                .rolls(1)
                // Only drop if killed by a player.
                .condition(LootCondition::KilledByPlayer)
                .entry(LootEntry::Item {
                    name: ItemId::minecraft("nether_star").unwrap().to_string(),
                    weight: None,
                    quality: None,
                    functions: vec![
                        LootFunction::SetCount {
                            count: NumberProvider::Constant(1.0),
                            add: false,
                        },
                        LootFunction::set_name(Text::new("Heart of the Boss").gold()),
                    ],
                    conditions: Vec::new(),
                })
                .entry(LootEntry::Item {
                    name: ItemId::minecraft("diamond").unwrap().to_string(),
                    weight: None,
                    quality: None,
                    functions: vec![
                        LootFunction::SetCount {
                            count: NumberProvider::Uniform { min: 3.0, max: 7.0 },
                            add: false,
                        },
                        LootFunction::EnchantWithLevels {
                            levels: NumberProvider::Constant(30.0),
                            options: Some(EnchantmentSelector::Id(
                                EnchantmentId::minecraft("sharpness").unwrap(),
                            )),
                        },
                    ],
                    conditions: Vec::new(),
                }),
        )
        .pool(
            // Bonus pool: 50% chance of bonus loot.
            LootPool::new()
                .rolls(1)
                .condition(LootCondition::RandomChance { chance: 0.5 })
                .entry(LootEntry::item(
                    ItemId::minecraft("enchanted_golden_apple").unwrap(),
                )),
        )
}

// ── Loot table with alternatives ─────────────────────────────────────────────
// First matching entry wins — useful for tiered drops.

#[datapack_component]
pub fn tiered_drops() -> LootTable {
    LootTable::new("my_pack:gameplay/tiered".parse().unwrap()).pool(
        LootPool::new().rolls(1).entry(LootEntry::alternatives(vec![
            // 5% chance: diamond
            LootEntry::Item {
                name: ItemId::minecraft("diamond").unwrap().to_string(),
                weight: None,
                quality: None,
                functions: Vec::new(),
                conditions: vec![LootCondition::RandomChance { chance: 0.05 }],
            },
            // 20% chance: gold
            LootEntry::Item {
                name: ItemId::minecraft("gold_ingot").unwrap().to_string(),
                weight: None,
                quality: None,
                functions: Vec::new(),
                conditions: vec![LootCondition::RandomChance { chance: 0.20 }],
            },
            // fallback: iron (always matches)
            LootEntry::item(ItemId::minecraft("iron_ingot").unwrap()),
        ])),
    )
}
