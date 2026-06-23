//! Typed helpers for [`CustomItem`] that accept [`IntoFunctionRef`] instead of raw strings.
//!
//! # Why an extension trait?
//!
//! [`CustomItem`] lives in `sand-components`, which does not depend on `sand-core`.
//! Rather than creating a circular dependency, this module defines a trait that
//! lives in `sand-core` and uses `IntoFunctionRef` from the same crate.
//!
//! # Example
//! ```rust,ignore
//! use sand_core::{CustomItem, ResourceLocation};
//! use sand_core::custom_item_ext::{CustomItemExt, CustomItemId};
//!
//! // Statically-typed item identity:
//! pub static SHOCKWAVE_SHIELD: CustomItemId = CustomItemId::new("minecraft:shield", "powers_shockwave");
//!
//! // Typed on_use with a #[function]-registered handler:
//! fn build_advancements(item: &CustomItem) -> Vec<Advancement> {
//!     vec![
//!         item.on_use_fn(
//!             ResourceLocation::new("my_pack", "items/shockwave/on_use"),
//!             my_pack::on_shockwave_use, // fn() -> Vec<String>
//!         )
//!     ]
//! }
//!
//! // Quick execute check via CustomItemId:
//! let check = SHOCKWAVE_SHIELD.has_in_offhand().run("say shockwave ready");
//! ```

use std::fmt;

use sand_commands::selector::Selector;
use sand_commands::{Execute, ItemSlot};

use crate::function::IntoFunctionRef;
use crate::{Advancement, CustomItem, ResourceLocation};

/// Extension trait for [`CustomItem`] that accepts typed function references.
///
/// Import this trait to unlock `.on_use_fn()`, `.on_kill_fn()`, and `.item_check_in()`.
pub trait CustomItemExt {
    /// Build a use-item advancement whose reward is a typed function ref.
    ///
    /// Identical to [`CustomItem::on_use_advancement`] but accepts any
    /// [`IntoFunctionRef`] — including `#[function]`-registered closures.
    fn on_use_fn(&self, location: ResourceLocation, handler: impl IntoFunctionRef) -> Advancement;

    /// Build a kill-entity advancement whose reward is a typed function ref.
    fn on_kill_fn(&self, location: ResourceLocation, handler: impl IntoFunctionRef) -> Advancement;

    /// Build a custom-trigger advancement whose reward is a typed function ref.
    fn on_trigger_fn(
        &self,
        location: ResourceLocation,
        trigger: crate::AdvancementTrigger,
        handler: impl IntoFunctionRef,
    ) -> Advancement;

    /// `execute if items entity @s <slot> <item_string>`.
    ///
    /// `item_string` is the `Display` form of this `CustomItem` (includes
    /// the `custom_data` component if set).
    fn item_check_in(&self, slot: impl Into<ItemSlot>) -> Execute;

    /// Same as `item_check_in(ItemSlot::MainHand)`.
    fn item_check_mainhand(&self) -> Execute {
        self.item_check_in(ItemSlot::MainHand)
    }

    /// Same as `item_check_in(ItemSlot::OffHand)`.
    fn item_check_offhand(&self) -> Execute {
        self.item_check_in(ItemSlot::OffHand)
    }

    /// Same as `item_check_in(ItemSlot::Raw("*"))` — any slot.
    fn item_check_anywhere(&self) -> Execute {
        self.item_check_in(ItemSlot::Raw("*".into()))
    }
}

impl CustomItemExt for CustomItem {
    fn on_use_fn(&self, location: ResourceLocation, handler: impl IntoFunctionRef) -> Advancement {
        self.on_use_advancement(location, handler.into_function_id())
    }

    fn on_kill_fn(&self, location: ResourceLocation, handler: impl IntoFunctionRef) -> Advancement {
        self.on_kill_advancement(location, handler.into_function_id())
    }

    fn on_trigger_fn(
        &self,
        location: ResourceLocation,
        trigger: crate::AdvancementTrigger,
        handler: impl IntoFunctionRef,
    ) -> Advancement {
        self.custom_trigger_advancement(location, trigger, handler.into_function_id())
    }

    fn item_check_in(&self, slot: impl Into<ItemSlot>) -> Execute {
        Execute::new().if_items_entity(Selector::self_(), slot.into(), self.to_string())
    }
}

// ── CustomItemId ──────────────────────────────────────────────────────────────

/// A statically-typed identifier for a custom item.
///
/// Stores the base item ID and the `custom_data` marker key. Use this type
/// for const-evaluable item identity without carrying the full [`CustomItem`]
/// builder at runtime.
///
/// # Example
/// ```rust,ignore
/// use sand_core::custom_item_ext::CustomItemId;
///
/// pub static SHOCKWAVE_SHIELD: CustomItemId =
///     CustomItemId::new("minecraft:shield", "powers_shockwave");
///
/// // Generate execute if items check:
/// let check = SHOCKWAVE_SHIELD.has_in(sand_core::cmd::ItemSlot::OffHand).run("say shockwave");
/// // → execute if items entity @s weapon.offhand minecraft:shield[custom_data={powers_shockwave:1b}] run say shockwave
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomItemId {
    base_item: &'static str,
    custom_data_key: &'static str,
}

impl CustomItemId {
    /// Create a new `CustomItemId`.
    ///
    /// - `base_item` — the Minecraft item ID, e.g. `"minecraft:shield"`.
    /// - `custom_data_key` — the marker key set via `.custom_data("key")`, e.g. `"powers_shockwave"`.
    pub const fn new(base_item: &'static str, custom_data_key: &'static str) -> Self {
        Self {
            base_item,
            custom_data_key,
        }
    }

    /// The item component string used in `execute if items entity` commands.
    ///
    /// Format: `{base_item}[custom_data={{custom_data_key:1b}}]`
    pub fn item_string(&self) -> String {
        format!(
            "{}[custom_data={{{}:1b}}]",
            self.base_item, self.custom_data_key
        )
    }

    /// `execute if items entity @s <slot> <item_string>`.
    pub fn has_in(&self, slot: impl Into<ItemSlot>) -> Execute {
        Execute::new().if_items_entity(Selector::self_(), slot.into(), self.item_string())
    }

    /// `execute if items entity @s weapon.mainhand <item_string>`.
    pub fn has_in_mainhand(&self) -> Execute {
        self.has_in(ItemSlot::MainHand)
    }

    /// `execute if items entity @s weapon.offhand <item_string>`.
    pub fn has_in_offhand(&self) -> Execute {
        self.has_in(ItemSlot::OffHand)
    }

    /// `execute if items entity @s * <item_string>` — any slot.
    pub fn has_anywhere(&self) -> Execute {
        self.has_in(ItemSlot::Raw("*".into()))
    }
}

impl fmt::Display for CustomItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}[custom_data={{{}:1b}}]",
            self.base_item, self.custom_data_key
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SHOCKWAVE: CustomItemId = CustomItemId::new("minecraft:shield", "powers_shockwave");

    #[test]
    fn item_string() {
        assert_eq!(
            SHOCKWAVE.item_string(),
            "minecraft:shield[custom_data={powers_shockwave:1b}]"
        );
    }

    #[test]
    fn display() {
        assert_eq!(
            SHOCKWAVE.to_string(),
            "minecraft:shield[custom_data={powers_shockwave:1b}]"
        );
    }

    #[test]
    fn has_in_mainhand() {
        let exec = SHOCKWAVE.has_in_mainhand();
        assert_eq!(
            exec.run("say shockwave"),
            "execute if items entity @s weapon.mainhand minecraft:shield[custom_data={powers_shockwave:1b}] run say shockwave"
        );
    }

    #[test]
    fn has_in_offhand() {
        let exec = SHOCKWAVE.has_in_offhand();
        assert_eq!(
            exec.run("say offhand"),
            "execute if items entity @s weapon.offhand minecraft:shield[custom_data={powers_shockwave:1b}] run say offhand"
        );
    }

    #[test]
    fn has_anywhere() {
        let exec = SHOCKWAVE.has_anywhere();
        assert_eq!(
            exec.run("say any"),
            "execute if items entity @s * minecraft:shield[custom_data={powers_shockwave:1b}] run say any"
        );
    }

    #[test]
    fn custom_item_ext_on_use_fn_accepts_raw_string() {
        let item = CustomItem::new("minecraft:shield").custom_data("powers_shockwave");
        let adv = item.on_use_fn(
            ResourceLocation::new("my_pack", "items/shockwave/on_use").unwrap(),
            "my_pack:functions/on_shockwave_use",
        );
        // Just verify it builds without panicking — the full JSON is tested in sand-components.
        let _ = adv;
    }

    #[test]
    fn item_check_in_matches_execute_pattern() {
        let item = CustomItem::new("minecraft:shield").custom_data("powers_shockwave");
        let exec = item.item_check_in(ItemSlot::OffHand);
        assert_eq!(
            exec.run("say shockwave"),
            "execute if items entity @s weapon.offhand minecraft:shield[custom_data={powers_shockwave:1b}] run say shockwave"
        );
    }

    #[test]
    fn item_check_anywhere() {
        let item = CustomItem::new("minecraft:diamond_sword").custom_data("my_sword");
        let exec = item.item_check_anywhere();
        assert_eq!(
            exec.run("say has sword"),
            "execute if items entity @s * minecraft:diamond_sword[custom_data={my_sword:1b}] run say has sword"
        );
    }
}
