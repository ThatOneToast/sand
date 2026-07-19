//! Typed inventory slot API for `item replace`, `item modify`, `clear`, and `give`.
//!
//! # Slot taxonomy (unified)
//!
//! [`ItemSlot`] is the canonical slot type used by both
//! inventory operations and `execute if items` checks. All [`Inventory`] methods accept
//! `impl Into<ItemSlot>` so you can pass an `ItemSlot` directly:
//!
//! ```rust,ignore
//! use sand_commands::{Inventory, ItemSlot, Selector};
//!
//! let inv = Inventory::of(Selector::self_());
//! inv.give("minecraft:diamond");
//! inv.set(ItemSlot::MainHand, "minecraft:diamond_sword");
//! inv.set(ItemSlot::Hotbar(3), "minecraft:torch");
//! inv.clear_slot(ItemSlot::MainHand);
//! inv.clear_item("minecraft:dirt");
//! ```

use std::fmt;

use crate::execute_args::ItemSlot;
use crate::selector::Selector;

// ── Inventory ─────────────────────────────────────────────────────────────────

/// Fluent inventory operations for an entity selector.
#[derive(Debug, Clone)]
pub struct Inventory {
    selector: Selector,
}

impl Inventory {
    /// Validate that slot indices are within their valid ranges.
    fn check_slot_bounds(slot: &ItemSlot) {
        match slot {
            ItemSlot::Container(n) if *n >= 54 => {
                panic!("ItemSlot::Container must be within range 0-53");
            }
            ItemSlot::Hotbar(n) if *n >= 9 => {
                panic!("ItemSlot::Hotbar must be within range 0-8");
            }
            ItemSlot::Inventory(n) if *n >= 27 => {
                panic!("ItemSlot::Inventory must be within range 0-26");
            }
            _ => {}
        }
    }

    /// Create an inventory handle for the given entity selector.
    pub fn of(selector: Selector) -> Self {
        Self { selector }
    }

    // ── Give / set ────────────────────────────────────────────────────────

    /// `give <selector> <item>` — add an item to the entity's inventory.
    pub fn give(&self, item: impl fmt::Display) -> String {
        format!("give {} {item}", self.selector)
    }

    /// `give <selector> <item> <count>` — add `count` copies of an item.
    pub fn give_count(&self, item: impl fmt::Display, count: u32) -> String {
        format!("give {} {item} {count}", self.selector)
    }

    /// `item replace entity <selector> <slot> with <item>` — overwrite a slot.
    ///
    /// Accepts any type that converts to [`ItemSlot`].
    pub fn set(&self, slot: impl Into<ItemSlot>, item: impl fmt::Display) -> String {
        let slot = slot.into();
        Self::check_slot_bounds(&slot);
        format!("item replace entity {} {slot} with {item}", self.selector)
    }

    /// `item replace entity <selector> <slot> with <item> <count>` — overwrite with a stack size.
    pub fn set_count(
        &self,
        slot: impl Into<ItemSlot>,
        item: impl fmt::Display,
        count: u32,
    ) -> String {
        let slot = slot.into();
        Self::check_slot_bounds(&slot);
        format!(
            "item replace entity {} {slot} with {item} {count}",
            self.selector
        )
    }

    // ── Clear ─────────────────────────────────────────────────────────────

    /// `item replace entity <selector> <slot> with air` — empty a specific slot.
    pub fn clear_slot(&self, slot: impl Into<ItemSlot>) -> String {
        let slot = slot.into();
        Self::check_slot_bounds(&slot);
        format!("item replace entity {} {slot} with air", self.selector)
    }

    /// `clear <selector> <item>` — remove all stacks of a specific item.
    pub fn clear_item(&self, item: impl Into<String>) -> String {
        format!("clear {} {}", self.selector, item.into())
    }

    /// `clear <selector> <item> <count>` — remove up to `count` of an item.
    pub fn clear_item_count(&self, item: impl Into<String>, count: u32) -> String {
        format!("clear {} {} {count}", self.selector, item.into())
    }

    /// `clear <selector>` — remove everything from the inventory.
    pub fn clear_all(&self) -> String {
        format!("clear {}", self.selector)
    }

    // ── Copy ──────────────────────────────────────────────────────────────

    /// Copy the item in `source_slot` of another entity into `slot` of this entity.
    pub fn copy_from(
        &self,
        slot: impl Into<ItemSlot>,
        source: Selector,
        source_slot: impl Into<ItemSlot>,
    ) -> String {
        let slot = slot.into();
        let source_slot = source_slot.into();
        Self::check_slot_bounds(&slot);
        format!(
            "item replace entity {} {slot} from entity {source} {source_slot}",
            self.selector
        )
    }

    // ── Modify ────────────────────────────────────────────────────────────

    /// `item modify entity <selector> <slot> <modifier>` — apply an item modifier.
    pub fn modify(&self, slot: impl Into<ItemSlot>, modifier: impl Into<String>) -> String {
        let slot = slot.into();
        Self::check_slot_bounds(&slot);
        format!(
            "item modify entity {} {slot} {}",
            self.selector,
            modifier.into()
        )
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Execute, ItemSlot, Selector};

    fn inv() -> Inventory {
        Inventory::of(Selector::self_())
    }

    // ── ItemSlot (canonical type) tests ───────────────────────────────────────

    #[test]
    fn item_slot_set_mainhand() {
        assert_eq!(
            inv().set(ItemSlot::MainHand, "minecraft:diamond_sword"),
            "item replace entity @s weapon.mainhand with minecraft:diamond_sword"
        );
    }

    #[test]
    fn item_slot_set_hotbar() {
        assert_eq!(
            inv().set(ItemSlot::Hotbar(3), "minecraft:torch"),
            "item replace entity @s hotbar.3 with minecraft:torch"
        );
    }

    #[test]
    fn item_slot_set_armor_head() {
        assert_eq!(
            inv().set(ItemSlot::Head, "minecraft:diamond_helmet"),
            "item replace entity @s armor.head with minecraft:diamond_helmet"
        );
    }

    #[test]
    fn item_slot_set_offhand() {
        assert_eq!(
            inv().set(ItemSlot::OffHand, "minecraft:shield"),
            "item replace entity @s weapon.offhand with minecraft:shield"
        );
    }

    #[test]
    fn item_slot_clear_slot() {
        assert_eq!(
            inv().clear_slot(ItemSlot::MainHand),
            "item replace entity @s weapon.mainhand with air"
        );
    }

    #[test]
    fn item_slot_all_families() {
        assert_eq!(ItemSlot::Head.to_string(), "armor.head");
        assert_eq!(ItemSlot::Chest.to_string(), "armor.chest");
        assert_eq!(ItemSlot::Legs.to_string(), "armor.legs");
        assert_eq!(ItemSlot::Feet.to_string(), "armor.feet");
        assert_eq!(ItemSlot::AnyArmor.to_string(), "armor.*");
        assert_eq!(ItemSlot::MainHand.to_string(), "weapon.mainhand");
        assert_eq!(ItemSlot::OffHand.to_string(), "weapon.offhand");
        assert_eq!(ItemSlot::AnyWeapon.to_string(), "weapon.*");
        assert_eq!(ItemSlot::Hotbar(0).to_string(), "hotbar.0");
        assert_eq!(ItemSlot::AnyHotbar.to_string(), "hotbar.*");
        assert_eq!(ItemSlot::Inventory(0).to_string(), "inventory.0");
        assert_eq!(ItemSlot::AnyInventory.to_string(), "inventory.*");
        assert_eq!(ItemSlot::Container(5).to_string(), "container.5");
        assert_eq!(ItemSlot::AnyContainer.to_string(), "container.*");
        assert_eq!(ItemSlot::HorseSaddle.to_string(), "horse.saddle");
        assert_eq!(ItemSlot::HorseChest.to_string(), "horse.chest");
        assert_eq!(ItemSlot::HorseArmor.to_string(), "horse.armor");
        assert_eq!(ItemSlot::AnyHorse.to_string(), "horse.*");
        assert_eq!(ItemSlot::AnyVillager.to_string(), "villager.*");
        assert_eq!(ItemSlot::Raw("custom.*".into()).to_string(), "custom.*");
    }

    #[test]
    fn execute_if_items_entity_item_slot() {
        let cmd = Execute::new()
            .if_items_entity(
                Selector::self_(),
                ItemSlot::MainHand,
                "minecraft:diamond_sword",
            )
            .run_raw("say holding sword");
        assert_eq!(
            cmd,
            "execute if items entity @s weapon.mainhand minecraft:diamond_sword run say holding sword"
        );
    }

    #[test]
    fn give() {
        assert_eq!(inv().give("minecraft:diamond"), "give @s minecraft:diamond");
        assert_eq!(
            inv().give_count("minecraft:torch", 16),
            "give @s minecraft:torch 16"
        );
    }

    #[test]
    fn clear_item() {
        assert_eq!(
            inv().clear_item("minecraft:dirt"),
            "clear @s minecraft:dirt"
        );
    }

    #[test]
    fn copy_from_item_slot() {
        assert_eq!(
            inv().copy_from(
                ItemSlot::Container(0),
                Selector::nearest_player(),
                ItemSlot::MainHand
            ),
            "item replace entity @s container.0 from entity @p weapon.mainhand"
        );
    }
}
