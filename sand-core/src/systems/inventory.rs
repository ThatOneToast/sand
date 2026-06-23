//! High-level typed inventory helpers.
//!
//! # Example
//! ```rust,ignore
//! use sand_core::systems::inventory::InventorySystem;
//! use sand_core::cmd::{Selector, ItemSlot};
//!
//! // Check mainhand then run a command:
//! let cmd = InventorySystem::for_entity(Selector::self_())
//!     .has("minecraft:diamond_sword")
//!     .in_mainhand()
//!     .run("say has sword");
//!
//! // Replace a slot:
//! let cmd = InventorySystem::for_entity(Selector::self_())
//!     .replace(ItemSlot::MainHand, "minecraft:iron_sword");
//!
//! // Clear items:
//! let cmd = InventorySystem::for_entity(Selector::self_())
//!     .clear_item("minecraft:arrow")
//!     .amount(64);
//! ```

use std::fmt;

use sand_commands::selector::Selector;
use sand_commands::{Execute, ItemSlot};

/// High-level builder for inventory operations on a single entity.
#[derive(Debug, Clone)]
pub struct InventorySystem {
    selector: Selector,
}

/// Intermediate builder — holds the item string before the slot is specified.
#[derive(Debug, Clone)]
pub struct HasItemCheck {
    selector: Selector,
    item: String,
}

/// Builder for `clear <selector> <item> [<count>]` commands.
#[derive(Debug, Clone)]
pub struct ClearBuilder {
    selector: Selector,
    item: String,
}

impl InventorySystem {
    /// Start an inventory operation for the given entity selector.
    pub fn for_entity(selector: Selector) -> Self {
        Self { selector }
    }

    /// Begin an item-presence check.
    ///
    /// Chain with `.in_slot(slot)`, `.in_mainhand()`, `.in_any_slot()`, etc. to get
    /// an `Execute` builder that can be finished with `.run(cmd)`.
    pub fn has(self, item: impl fmt::Display) -> HasItemCheck {
        HasItemCheck {
            selector: self.selector,
            item: item.to_string(),
        }
    }

    /// Shorthand for `.has(item).in_slot(slot)`.
    pub fn has_in(self, slot: impl Into<ItemSlot>, item: impl fmt::Display) -> Execute {
        Execute::new().if_items_entity(self.selector, slot.into(), item.to_string())
    }

    /// `item replace entity <selector> <slot> with <item>` — replace a slot's contents.
    pub fn replace(self, slot: impl Into<ItemSlot>, item: impl fmt::Display) -> String {
        format!(
            "item replace entity {} {} with {}",
            self.selector,
            slot.into(),
            item
        )
    }

    /// `item replace entity <selector> <slot> with <item> <count>`.
    pub fn replace_count(
        self,
        slot: impl Into<ItemSlot>,
        item: impl fmt::Display,
        count: u32,
    ) -> String {
        format!(
            "item replace entity {} {} with {} {}",
            self.selector,
            slot.into(),
            item,
            count
        )
    }

    /// `item replace entity <selector> <slot> with air` — clear a single slot.
    pub fn clear_slot(self, slot: impl Into<ItemSlot>) -> String {
        format!(
            "item replace entity {} {} with air",
            self.selector,
            slot.into()
        )
    }

    /// Begin a `clear` command. Call `.amount(n)` or use the returned builder
    /// as a `String` (via `Display`) to clear all matching stacks.
    pub fn clear_item(self, item: impl Into<String>) -> ClearBuilder {
        ClearBuilder {
            selector: self.selector,
            item: item.into(),
        }
    }

    /// `give <selector> <item>` — give an item directly.
    pub fn give(self, item: impl fmt::Display) -> String {
        format!("give {} {}", self.selector, item)
    }
}

impl HasItemCheck {
    /// `execute if items entity <selector> <slot> <item>` — check in a specific slot.
    pub fn in_slot(self, slot: impl Into<ItemSlot>) -> Execute {
        Execute::new().if_items_entity(self.selector, slot.into(), self.item)
    }

    /// Check in the `weapon.*` (mainhand or offhand) slots.
    pub fn in_any_weapon(self) -> Execute {
        self.in_slot(ItemSlot::AnyWeapon)
    }

    /// Check in the `weapon.mainhand` slot.
    pub fn in_mainhand(self) -> Execute {
        self.in_slot(ItemSlot::MainHand)
    }

    /// Check in the `weapon.offhand` slot.
    pub fn in_offhand(self) -> Execute {
        self.in_slot(ItemSlot::OffHand)
    }

    /// Check in any of the four `armor.*` slots.
    pub fn in_armor(self) -> Execute {
        self.in_slot(ItemSlot::AnyArmor)
    }

    /// Check in any of the 9 `hotbar.*` slots.
    pub fn in_hotbar(self) -> Execute {
        self.in_slot(ItemSlot::AnyHotbar)
    }

    /// Check in any `inventory.*` slot (the main 27-slot grid).
    pub fn in_inventory(self) -> Execute {
        self.in_slot(ItemSlot::AnyInventory)
    }

    /// Check across all slots using `*` — any slot in the entity's full inventory.
    pub fn in_any_slot(self) -> Execute {
        self.in_slot(ItemSlot::Raw("*".into()))
    }

    /// `execute unless items entity <selector> <slot> <item>` — the negated form.
    pub fn not_in_slot(self, slot: impl Into<ItemSlot>) -> Execute {
        Execute::new().unless_items_entity(self.selector, slot.into(), self.item)
    }

    /// Skip if the item is anywhere in the full inventory (`*`).
    pub fn not_anywhere(self) -> Execute {
        Execute::new().unless_items_entity(self.selector, ItemSlot::Raw("*".into()), self.item)
    }
}

impl ClearBuilder {
    /// `clear <selector> <item> <count>` — remove up to `count` items.
    pub fn amount(self, count: u32) -> String {
        format!("clear {} {} {}", self.selector, self.item, count)
    }
}

impl fmt::Display for ClearBuilder {
    /// `clear <selector> <item>` — remove all matching stacks.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "clear {} {}", self.selector, self.item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sand_commands::selector::Selector;

    #[test]
    fn has_in_mainhand() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:diamond_sword")
            .in_mainhand();
        assert_eq!(
            exec.run("say armed"),
            "execute if items entity @s weapon.mainhand minecraft:diamond_sword run say armed"
        );
    }

    #[test]
    fn has_in_slot_explicit() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:shield")
            .in_slot(ItemSlot::OffHand);
        assert_eq!(
            exec.run("say blocking"),
            "execute if items entity @s weapon.offhand minecraft:shield run say blocking"
        );
    }

    #[test]
    fn has_in_any_slot() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:arrow")
            .in_any_slot();
        assert_eq!(
            exec.run("say has arrows"),
            "execute if items entity @s * minecraft:arrow run say has arrows"
        );
    }

    #[test]
    fn has_in_hotbar() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:totem_of_undying")
            .in_hotbar();
        assert_eq!(
            exec.run("say totem"),
            "execute if items entity @s hotbar.* minecraft:totem_of_undying run say totem"
        );
    }

    #[test]
    fn has_in_armor() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:elytra")
            .in_armor();
        assert_eq!(
            exec.run("say flying"),
            "execute if items entity @s armor.* minecraft:elytra run say flying"
        );
    }

    #[test]
    fn not_anywhere() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:arrow")
            .not_anywhere();
        assert_eq!(
            exec.run("say no arrows"),
            "execute unless items entity @s * minecraft:arrow run say no arrows"
        );
    }

    #[test]
    fn has_in_shorthand() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has_in(ItemSlot::MainHand, "minecraft:bow");
        assert_eq!(
            exec.run("say bow"),
            "execute if items entity @s weapon.mainhand minecraft:bow run say bow"
        );
    }

    #[test]
    fn replace_slot() {
        let cmd = InventorySystem::for_entity(Selector::self_())
            .replace(ItemSlot::MainHand, "minecraft:diamond_sword");
        assert_eq!(
            cmd,
            "item replace entity @s weapon.mainhand with minecraft:diamond_sword"
        );
    }

    #[test]
    fn replace_slot_with_count() {
        let cmd = InventorySystem::for_entity(Selector::self_()).replace_count(
            ItemSlot::Hotbar(0),
            "minecraft:arrow",
            64,
        );
        assert_eq!(
            cmd,
            "item replace entity @s hotbar.0 with minecraft:arrow 64"
        );
    }

    #[test]
    fn clear_slot() {
        let cmd = InventorySystem::for_entity(Selector::self_()).clear_slot(ItemSlot::OffHand);
        assert_eq!(cmd, "item replace entity @s weapon.offhand with air");
    }

    #[test]
    fn clear_item_display() {
        let builder = InventorySystem::for_entity(Selector::self_()).clear_item("minecraft:arrow");
        assert_eq!(builder.to_string(), "clear @s minecraft:arrow");
    }

    #[test]
    fn clear_item_amount() {
        let cmd = InventorySystem::for_entity(Selector::self_())
            .clear_item("minecraft:arrow")
            .amount(16);
        assert_eq!(cmd, "clear @s minecraft:arrow 16");
    }

    #[test]
    fn give_item() {
        let cmd = InventorySystem::for_entity(Selector::all_players()).give("minecraft:diamond");
        assert_eq!(cmd, "give @a minecraft:diamond");
    }

    #[test]
    fn custom_item_string_passthrough() {
        // CustomItem::to_string() produces "minecraft:shield[custom_data={...}]"
        // InventorySystem accepts impl fmt::Display so CustomItem works directly.
        let item_str = "minecraft:shield[custom_data={powers_shockwave:1b}]";
        let exec = InventorySystem::for_entity(Selector::self_())
            .has(item_str)
            .in_offhand();
        assert_eq!(
            exec.run("say shockwave"),
            "execute if items entity @s weapon.offhand minecraft:shield[custom_data={powers_shockwave:1b}] run say shockwave"
        );
    }
}
