//! # Loot tables
//!
//! Demonstrates loot tables with pools, conditions, functions,
//! and convenience constructors.

use sand_core::{
    LootCondition, LootEntry, LootFunction, LootPool, LootTable,
    LootTableType, NumberProvider,
};
use sand_macros::component;

// ── Simple entity drop ───────────────────────────────────────────────────────
// Uses the convenience constructor for common entity loot patterns.

#[component]
pub fn zombie_drops() -> LootTable {
    LootTable::entity_drop(
        "my_pack:entities/zombie".parse().unwrap(),
        vec![
            // Always drop 0-2 rotten flesh
            LootPool::new()
                .rolls(NumberProvider::Constant(1.0))
                .entry(
                    LootEntry::item("minecraft:rotten_flesh")
                        .function(LootFunction::SetCount {
                            count: NumberProvider::Uniform { min: 0.0, max: 2.0 },
                            add: false,
                        })
                        .function(LootFunction::LootingEnchant {
                            count: NumberProvider::Uniform { min: 0.0, max: 1.0 },
                            limit: Some(3),
                        }),
                ),
            // Rare iron ingot drop (2.5% chance + looting bonus)
            LootPool::new()
                .rolls(NumberProvider::Constant(1.0))
                .entry(
                    LootEntry::item("minecraft:iron_ingot")
                        .condition(LootCondition::RandomChanceWithLooting {
                            chance: 0.025,
                            looting_multiplier: 0.01,
                        }),
                ),
        ],
    )
}

// ── Chest loot ───────────────────────────────────────────────────────────────
// Uses the chest loot convenience constructor.

#[component]
pub fn dungeon_chest() -> LootTable {
    LootTable::chest_loot(
        "my_pack:chests/dungeon".parse().unwrap(),
        vec![
            // 2-4 rolls of common items
            LootPool::new()
                .rolls(NumberProvider::Uniform { min: 2.0, max: 4.0 })
                .entry(LootEntry::item("minecraft:iron_ingot").weight(10))
                .entry(LootEntry::item("minecraft:gold_ingot").weight(5))
                .entry(LootEntry::item("minecraft:diamond").weight(1)),
            // One guaranteed enchanted book
            LootPool::new()
                .rolls(NumberProvider::Constant(1.0))
                .entry(
                    LootEntry::item("minecraft:book")
                        .function(LootFunction::EnchantRandomly { options: None }),
                ),
        ],
    )
}

// ── Full loot table with conditions ──────────────────────────────────────────
// Demonstrates manual construction with conditions and functions.

#[component]
pub fn boss_loot() -> LootTable {
    LootTable::new("my_pack:entities/boss".parse().unwrap())
        .loot_type(LootTableType::Entity)
        .pool(
            LootPool::new()
                .rolls(NumberProvider::Constant(1.0))
                // Only drop if killed by a player
                .condition(LootCondition::KilledByPlayer)
                .entry(
                    LootEntry::item("minecraft:nether_star")
                        .function(LootFunction::SetCount {
                            count: NumberProvider::Constant(1.0),
                            add: false,
                        }),
                )
                .entry(
                    LootEntry::item("minecraft:diamond")
                        .function(LootFunction::SetCount {
                            count: NumberProvider::Uniform { min: 3.0, max: 7.0 },
                            add: false,
                        })
                        .function(LootFunction::LootingEnchant {
                            count: NumberProvider::Uniform { min: 0.0, max: 2.0 },
                            limit: Some(10),
                        }),
                ),
        )
        .pool(
            // Bonus pool: 50% chance of bonus loot
            LootPool::new()
                .rolls(NumberProvider::Constant(1.0))
                .condition(LootCondition::RandomChance { chance: 0.5 })
                .entry(
                    LootEntry::item("minecraft:enchanted_golden_apple")
                        .function(LootFunction::SetCount {
                            count: NumberProvider::Constant(1.0),
                            add: false,
                        }),
                ),
        )
}

// ── Loot table with alternatives ─────────────────────────────────────────────
// First matching entry wins — useful for tiered drops.

#[component]
pub fn tiered_drops() -> LootTable {
    LootTable::new("my_pack:gameplay/tiered".parse().unwrap())
        .pool(
            LootPool::new()
                .rolls(NumberProvider::Constant(1.0))
                .entry(LootEntry::alternatives(vec![
                    // 5% chance: diamond
                    LootEntry::item("minecraft:diamond")
                        .condition(LootCondition::RandomChance { chance: 0.05 }),
                    // 20% chance: gold
                    LootEntry::item("minecraft:gold_ingot")
                        .condition(LootCondition::RandomChance { chance: 0.20 }),
                    // fallback: iron (always matches)
                    LootEntry::item("minecraft:iron_ingot"),
                ])),
        )
}
