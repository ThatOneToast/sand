//! # Advancements
//!
//! Demonstrates creating custom advancements with triggers, criteria,
//! display settings, and rewards.

use sand_core::{
    Advancement, AdvancementDisplay, AdvancementFrame, AdvancementIcon,
    AdvancementRewards, AdvancementTrigger, Criterion,
};
use sand_macros::component;

// ── Basic advancement with a tick trigger ────────────────────────────────────
// Fires every tick — commonly used with revocation for one-shot detection.

#[component]
pub fn first_join() -> Advancement {
    Advancement::new("my_pack:first_join".parse().unwrap())
        .criterion("joined", Criterion::new(AdvancementTrigger::Tick))
        .rewards(AdvancementRewards::new().function("my_pack:on_first_join"))
}

// ── Advancement with display ─────────────────────────────────────────────────
// Shows a toast notification and appears in the advancement screen.

#[component]
pub fn get_diamonds() -> Advancement {
    Advancement::new("my_pack:get_diamonds".parse().unwrap())
        .display(
            AdvancementDisplay::new(
                AdvancementIcon::new("minecraft:diamond"),
                "Diamond Collector",
                "Mine your first diamond",
            )
            .frame(AdvancementFrame::Task)
            .show_toast(true)
            .announce_to_chat(true),
        )
        .criterion(
            "has_diamond",
            Criterion::new(AdvancementTrigger::inventory_changed(vec![
                sand_core::generated::Item::Diamond,
            ])),
        )
}

// ── Challenge advancement ────────────────────────────────────────────────────
// Uses the Challenge frame for difficult achievements.

#[component]
pub fn dragon_slayer() -> Advancement {
    Advancement::new("my_pack:dragon_slayer".parse().unwrap())
        .parent("my_pack:get_diamonds".parse().unwrap())
        .display(
            AdvancementDisplay::new(
                AdvancementIcon::new("minecraft:dragon_head"),
                "Dragon Slayer",
                "Defeat the Ender Dragon",
            )
            .frame(AdvancementFrame::Challenge)
            .announce_to_chat(true),
        )
        .criterion(
            "killed_dragon",
            Criterion::new(AdvancementTrigger::Custom {
                trigger: "minecraft:player_killed_entity".to_string(),
                conditions: serde_json::json!({
                    "entity": {
                        "type": "minecraft:ender_dragon"
                    }
                }),
            }),
        )
        .rewards(
            AdvancementRewards::new()
                .experience(1000)
                .function("my_pack:dragon_reward"),
        )
}

// ── Advancement with multiple criteria ───────────────────────────────────────
// Requires all criteria to be met (or specify custom requirement groups).

#[component]
pub fn master_miner() -> Advancement {
    Advancement::new("my_pack:master_miner".parse().unwrap())
        .display(
            AdvancementDisplay::new(
                AdvancementIcon::new("minecraft:netherite_pickaxe"),
                "Master Miner",
                "Mine all rare ores",
            )
            .frame(AdvancementFrame::Goal),
        )
        .criterion(
            "mined_diamond",
            Criterion::new(AdvancementTrigger::Custom {
                trigger: "minecraft:inventory_changed".to_string(),
                conditions: serde_json::json!({
                    "items": [{"items": "minecraft:diamond"}]
                }),
            }),
        )
        .criterion(
            "mined_emerald",
            Criterion::new(AdvancementTrigger::Custom {
                trigger: "minecraft:inventory_changed".to_string(),
                conditions: serde_json::json!({
                    "items": [{"items": "minecraft:emerald"}]
                }),
            }),
        )
        // Both criteria must be met (default behavior)
        .requirements(vec![
            vec!["mined_diamond".to_string()],
            vec!["mined_emerald".to_string()],
        ])
}
