//! Typed inventory slot API for `item replace`, `item modify`, `clear`, and `give`.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use sand_commands::{Inventory, InventorySlot, SlotPattern, Selector};
//!
//! let inv = Inventory::of(Selector::self_());
//!
//! inv.give("minecraft:diamond");
//! inv.set(InventorySlot::Mainhand, "minecraft:diamond_sword");
//! inv.set(InventorySlot::Hotbar(3), "minecraft:torch");
//! inv.clear_slot(InventorySlot::Mainhand);
//! inv.clear_item("minecraft:dirt");
//! ```

use std::fmt;

use crate::selector::Selector;

// ── InventorySlot ─────────────────────────────────────────────────────────────

/// A specific inventory slot in a player or entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventorySlot {
    /// The item held in the main hand (`weapon.mainhand`).
    Mainhand,
    /// The item held in the off hand (`weapon.offhand`).
    Offhand,
    /// Helmet slot (`armor.head`).
    ArmorHead,
    /// Chestplate slot (`armor.chest`).
    ArmorChest,
    /// Leggings slot (`armor.legs`).
    ArmorLegs,
    /// Boots slot (`armor.feet`).
    ArmorFeet,
    /// Hotbar slot 0–8 (`hotbar.N`).
    Hotbar(u8),
    /// Main inventory slot 0–26 (`container.N`).
    Container(u8),
}

impl InventorySlot {
    /// All hotbar slots in order (0–8).
    pub fn all_hotbar() -> [InventorySlot; 9] {
        std::array::from_fn(|i| InventorySlot::Hotbar(i as u8))
    }

    /// All main-inventory container slots in order (0–26).
    pub fn all_container() -> [InventorySlot; 27] {
        std::array::from_fn(|i| InventorySlot::Container(i as u8))
    }

    /// All armor slots in order (head → feet).
    pub fn all_armor() -> [InventorySlot; 4] {
        [
            InventorySlot::ArmorHead,
            InventorySlot::ArmorChest,
            InventorySlot::ArmorLegs,
            InventorySlot::ArmorFeet,
        ]
    }
}

impl fmt::Display for InventorySlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InventorySlot::Mainhand => write!(f, "weapon.mainhand"),
            InventorySlot::Offhand => write!(f, "weapon.offhand"),
            InventorySlot::ArmorHead => write!(f, "armor.head"),
            InventorySlot::ArmorChest => write!(f, "armor.chest"),
            InventorySlot::ArmorLegs => write!(f, "armor.legs"),
            InventorySlot::ArmorFeet => write!(f, "armor.feet"),
            InventorySlot::Hotbar(n) => write!(f, "hotbar.{n}"),
            InventorySlot::Container(n) => write!(f, "container.{n}"),
        }
    }
}

// ── SlotPattern ───────────────────────────────────────────────────────────────

/// A slot pattern for wildcard `execute if items` checks (1.20.5+).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotPattern {
    /// A specific slot — same as using [`InventorySlot`] directly.
    Slot(InventorySlot),
    /// Any hotbar slot (`hotbar.*`).
    AnyHotbar,
    /// Any main-inventory container slot (`container.*`).
    AnyContainer,
    /// Any armor slot (`armor.*`).
    AnyArmor,
    /// Any weapon slot (`weapon.*`).
    AnyWeapon,
    /// Every slot (`*`).
    Any,
}

impl fmt::Display for SlotPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlotPattern::Slot(s) => write!(f, "{s}"),
            SlotPattern::AnyHotbar => write!(f, "hotbar.*"),
            SlotPattern::AnyContainer => write!(f, "container.*"),
            SlotPattern::AnyArmor => write!(f, "armor.*"),
            SlotPattern::AnyWeapon => write!(f, "weapon.*"),
            SlotPattern::Any => write!(f, "*"),
        }
    }
}

impl From<InventorySlot> for SlotPattern {
    fn from(slot: InventorySlot) -> Self {
        SlotPattern::Slot(slot)
    }
}

// ── Inventory ─────────────────────────────────────────────────────────────────

/// Fluent inventory operations for an entity selector.
#[derive(Debug, Clone)]
pub struct Inventory {
    selector: Selector,
}

impl Inventory {
    /// Validate that inventory slots are within their valid ranges.
    pub(crate) fn ensure_slot(slot: &InventorySlot) {
        match slot {
            InventorySlot::Container(n) => {
                if *n >= 27 {
                    panic!("InventorySlot::Container must be within range 0-26");
                }
            }
            InventorySlot::Hotbar(n) => {
                if *n >= 9 {
                    panic!("InventorySlot::Hotbar must be within range 0-8");
                }
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
    pub fn set(&self, slot: InventorySlot, item: impl fmt::Display) -> String {
        Self::ensure_slot(&slot);
        format!("item replace entity {} {slot} with {item}", self.selector)
    }

    /// `item replace entity <selector> <slot> with <item> <count>` — overwrite with a stack size.
    pub fn set_count(&self, slot: InventorySlot, item: impl fmt::Display, count: u32) -> String {
        Self::ensure_slot(&slot);
        format!(
            "item replace entity {} {slot} with {item} {count}",
            self.selector
        )
    }

    // ── Clear ─────────────────────────────────────────────────────────────

    /// `item replace entity <selector> <slot> with air` — empty a specific slot.
    pub fn clear_slot(&self, slot: InventorySlot) -> String {
        Self::ensure_slot(&slot);
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
        slot: InventorySlot,
        source: Selector,
        source_slot: InventorySlot,
    ) -> String {
        Self::ensure_slot(&slot);
        format!(
            "item replace entity {} {slot} from entity {source} {source_slot}",
            self.selector
        )
    }

    // ── Modify ────────────────────────────────────────────────────────────

    /// `item modify entity <selector> <slot> <modifier>` — apply an item modifier.
    pub fn modify(&self, slot: InventorySlot, modifier: impl Into<String>) -> String {
        Self::ensure_slot(&slot);
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
    use crate::{Execute, Selector};

    fn inv() -> Inventory {
        Inventory::of(Selector::self_())
    }

    #[test]
    fn slot_display() {
        assert_eq!(InventorySlot::Mainhand.to_string(), "weapon.mainhand");
        assert_eq!(InventorySlot::Offhand.to_string(), "weapon.offhand");
        assert_eq!(InventorySlot::ArmorHead.to_string(), "armor.head");
        assert_eq!(InventorySlot::ArmorFeet.to_string(), "armor.feet");
        assert_eq!(InventorySlot::Hotbar(0).to_string(), "hotbar.0");
        assert_eq!(InventorySlot::Hotbar(8).to_string(), "hotbar.8");
        assert_eq!(InventorySlot::Container(3).to_string(), "container.3");
    }

    #[test]
    fn slot_pattern_display() {
        assert_eq!(SlotPattern::AnyHotbar.to_string(), "hotbar.*");
        assert_eq!(SlotPattern::AnyContainer.to_string(), "container.*");
        assert_eq!(SlotPattern::Any.to_string(), "*");
        assert_eq!(
            SlotPattern::Slot(InventorySlot::Mainhand).to_string(),
            "weapon.mainhand"
        );
    }

    #[test]
    fn all_hotbar_has_9_slots() {
        let slots = InventorySlot::all_hotbar();
        assert_eq!(slots.len(), 9);
        assert_eq!(slots[0], InventorySlot::Hotbar(0));
        assert_eq!(slots[8], InventorySlot::Hotbar(8));
    }

    #[test]
    fn all_container_has_27_slots() {
        assert_eq!(InventorySlot::all_container().len(), 27);
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
    fn set_slot() {
        assert_eq!(
            inv().set(InventorySlot::Mainhand, "minecraft:diamond_sword"),
            "item replace entity @s weapon.mainhand with minecraft:diamond_sword"
        );
        assert_eq!(
            inv().set(InventorySlot::Hotbar(3), "minecraft:torch"),
            "item replace entity @s hotbar.3 with minecraft:torch"
        );
    }

    #[test]
    fn clear_slot() {
        assert_eq!(
            inv().clear_slot(InventorySlot::Mainhand),
            "item replace entity @s weapon.mainhand with air"
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
    fn copy_from() {
        assert_eq!(
            inv().copy_from(
                InventorySlot::Container(0),
                Selector::nearest_player(),
                InventorySlot::Mainhand
            ),
            "item replace entity @s container.0 from entity @p weapon.mainhand"
        );
    }

    #[test]
    fn execute_if_items() {
        let cmd = Execute::new()
            .if_items(
                Selector::self_(),
                InventorySlot::Mainhand,
                "minecraft:diamond_sword",
            )
            .run_raw("say holding sword");
        assert_eq!(
            cmd,
            "execute if items entity @s weapon.mainhand minecraft:diamond_sword run say holding sword"
        );
    }

    #[test]
    fn execute_if_items_pattern() {
        let cmd = Execute::new()
            .if_items_pattern(Selector::self_(), SlotPattern::AnyHotbar, "minecraft:torch")
            .run_raw("say has torch");
        assert_eq!(
            cmd,
            "execute if items entity @s hotbar.* minecraft:torch run say has torch"
        );
    }

    #[test]
    fn rust_iteration_over_hotbar() {
        let cmds: Vec<String> = InventorySlot::all_hotbar()
            .iter()
            .map(|slot| inv().set(*slot, "minecraft:air"))
            .collect();
        assert_eq!(cmds.len(), 9);
        assert!(cmds[0].contains("hotbar.0"));
        assert!(cmds[8].contains("hotbar.8"));
    }
}
