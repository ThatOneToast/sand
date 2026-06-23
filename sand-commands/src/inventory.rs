//! Typed inventory slot API for `item replace`, `item modify`, `clear`, and `give`.
//!
//! # Slot taxonomy (unified)
//!
//! [`ItemSlot`](crate::execute_args::ItemSlot) is the canonical slot type used by both
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
//!
//! # Deprecated aliases
//!
//! [`InventorySlot`] and [`SlotPattern`] are deprecated. They convert to [`ItemSlot`]
//! via `From` implementations so existing code keeps compiling until you migrate.

use std::fmt;

use crate::execute_args::ItemSlot;
use crate::selector::Selector;

// ── InventorySlot (deprecated) ────────────────────────────────────────────────

/// A specific inventory slot in a player or entity.
///
/// Deprecated: use [`ItemSlot`](crate::execute_args::ItemSlot) instead.
/// All [`Inventory`] methods now accept `impl Into<ItemSlot>`.
#[deprecated(note = "use `ItemSlot` from `sand_commands::execute_args` instead")]
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

#[allow(deprecated)]
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

#[allow(deprecated)]
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

/// Convert a deprecated [`InventorySlot`] to the canonical [`ItemSlot`].
#[allow(deprecated)]
impl From<InventorySlot> for ItemSlot {
    fn from(slot: InventorySlot) -> Self {
        match slot {
            InventorySlot::Mainhand => ItemSlot::MainHand,
            InventorySlot::Offhand => ItemSlot::OffHand,
            InventorySlot::ArmorHead => ItemSlot::Head,
            InventorySlot::ArmorChest => ItemSlot::Chest,
            InventorySlot::ArmorLegs => ItemSlot::Legs,
            InventorySlot::ArmorFeet => ItemSlot::Feet,
            InventorySlot::Hotbar(n) => ItemSlot::Hotbar(n),
            InventorySlot::Container(n) => ItemSlot::Container(n),
        }
    }
}

// ── SlotPattern (deprecated) ──────────────────────────────────────────────────

/// A slot pattern for wildcard `execute if items` checks (1.20.5+).
///
/// Deprecated: use [`ItemSlot`](crate::execute_args::ItemSlot) wildcard variants
/// (`ItemSlot::AnyHotbar`, `ItemSlot::AnyArmor`, etc.) instead.
#[deprecated(note = "use `ItemSlot` wildcard variants instead (e.g. `ItemSlot::AnyHotbar`)")]
#[allow(deprecated)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotPattern {
    /// A specific slot — same as using [`InventorySlot`] directly.
    #[allow(deprecated)]
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

#[allow(deprecated)]
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

#[allow(deprecated)]
impl From<InventorySlot> for SlotPattern {
    fn from(slot: InventorySlot) -> Self {
        SlotPattern::Slot(slot)
    }
}

/// Convert a deprecated [`SlotPattern`] to the canonical [`ItemSlot`].
#[allow(deprecated)]
impl From<SlotPattern> for ItemSlot {
    fn from(pattern: SlotPattern) -> Self {
        match pattern {
            SlotPattern::Slot(s) => ItemSlot::from(s),
            SlotPattern::AnyHotbar => ItemSlot::AnyHotbar,
            SlotPattern::AnyContainer => ItemSlot::AnyContainer,
            SlotPattern::AnyArmor => ItemSlot::AnyArmor,
            SlotPattern::AnyWeapon => ItemSlot::AnyWeapon,
            SlotPattern::Any => ItemSlot::Raw("*".into()),
        }
    }
}

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
    /// Accepts any type that converts to [`ItemSlot`], including the deprecated
    /// [`InventorySlot`] via its `From` implementation.
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
    #[allow(deprecated)]
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

    // ── Deprecated InventorySlot compat tests ─────────────────────────────────

    #[test]
    #[allow(deprecated)]
    fn deprecated_inventory_slot_converts_to_item_slot() {
        assert_eq!(ItemSlot::from(InventorySlot::Mainhand), ItemSlot::MainHand);
        assert_eq!(ItemSlot::from(InventorySlot::Offhand), ItemSlot::OffHand);
        assert_eq!(ItemSlot::from(InventorySlot::ArmorHead), ItemSlot::Head);
        assert_eq!(ItemSlot::from(InventorySlot::ArmorFeet), ItemSlot::Feet);
        assert_eq!(
            ItemSlot::from(InventorySlot::Hotbar(3)),
            ItemSlot::Hotbar(3)
        );
        assert_eq!(
            ItemSlot::from(InventorySlot::Container(5)),
            ItemSlot::Container(5)
        );
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_slot_pattern_converts_to_item_slot() {
        assert_eq!(ItemSlot::from(SlotPattern::AnyHotbar), ItemSlot::AnyHotbar);
        assert_eq!(
            ItemSlot::from(SlotPattern::AnyContainer),
            ItemSlot::AnyContainer
        );
        assert_eq!(ItemSlot::from(SlotPattern::AnyArmor), ItemSlot::AnyArmor);
        assert_eq!(ItemSlot::from(SlotPattern::AnyWeapon), ItemSlot::AnyWeapon);
        assert_eq!(ItemSlot::from(SlotPattern::Any), ItemSlot::Raw("*".into()));
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_inventory_slot_still_works_with_inventory() {
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
    #[allow(deprecated)]
    fn deprecated_slot_display() {
        assert_eq!(InventorySlot::Mainhand.to_string(), "weapon.mainhand");
        assert_eq!(InventorySlot::Offhand.to_string(), "weapon.offhand");
        assert_eq!(InventorySlot::ArmorHead.to_string(), "armor.head");
        assert_eq!(InventorySlot::ArmorFeet.to_string(), "armor.feet");
        assert_eq!(InventorySlot::Hotbar(0).to_string(), "hotbar.0");
        assert_eq!(InventorySlot::Hotbar(8).to_string(), "hotbar.8");
        assert_eq!(InventorySlot::Container(3).to_string(), "container.3");
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_slot_pattern_display() {
        assert_eq!(SlotPattern::AnyHotbar.to_string(), "hotbar.*");
        assert_eq!(SlotPattern::AnyContainer.to_string(), "container.*");
        assert_eq!(SlotPattern::Any.to_string(), "*");
        assert_eq!(
            SlotPattern::Slot(InventorySlot::Mainhand).to_string(),
            "weapon.mainhand"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn all_hotbar_has_9_slots() {
        let slots = InventorySlot::all_hotbar();
        assert_eq!(slots.len(), 9);
        assert_eq!(slots[0], InventorySlot::Hotbar(0));
        assert_eq!(slots[8], InventorySlot::Hotbar(8));
    }

    #[test]
    #[allow(deprecated)]
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

    #[test]
    #[allow(deprecated)]
    fn execute_if_items_deprecated_compat() {
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
    #[allow(deprecated)]
    fn execute_if_items_pattern_deprecated_compat() {
        let cmd = Execute::new()
            .if_items_pattern(Selector::self_(), SlotPattern::AnyHotbar, "minecraft:torch")
            .run_raw("say has torch");
        assert_eq!(
            cmd,
            "execute if items entity @s hotbar.* minecraft:torch run say has torch"
        );
    }

    #[test]
    #[allow(deprecated)]
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
