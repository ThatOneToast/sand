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
//!
//! # Validation boundary (see [#172](https://github.com/ThatOneToast/sand/issues/172))
//!
//! Every [`Inventory`] method builds its command line, but the historical
//! infallible methods (`give`, `set`, `clear_slot`, `copy_from`, `modify`,
//! …) never panic and never eagerly reject malformed input — construction
//! stays ergonomic for chained builder call sites, matching the
//! `BlockState`/`SetBlock` convention in [`crate::blocks`]. Instead:
//!
//! - Slot bounds no longer panic (`check_slot_bounds` was replaced —
//!   out-of-range indices are Sand diagnostics, not process aborts).
//! - Every rendered line's typed node is retained in a pre-write export
//!   registry (mirroring [`crate::blocks`]'s `BlockCommandNode` pattern), so
//!   the export pipeline's `validate_collected_line` re-validates the
//!   *typed* slot/item/modifier/count data even for lines produced by the
//!   infallible methods.
//! - `try_*` counterparts (`try_give`, `try_set`, `try_clear_slot`,
//!   `try_copy_from`, `try_modify`, …) validate immediately and return
//!   [`CommandResult<String>`] for call sites that want to fail fast instead
//!   of waiting for export.
//!
//! ## Wildcard slots
//!
//! [`ItemSlot::is_wildcard`] slots (`armor.*`, `hotbar.*`, …) are valid for
//! read/check contexts such as `execute if items`, but Minecraft's
//! single-slot write grammar (`item replace`/`item modify`) requires
//! exactly one resolved slot. `Inventory`'s slot-accepting write methods
//! reject wildcard slots; `execute if items` (in [`crate::execute`]) is
//! unaffected and continues to accept them.
//!
//! ## Item and modifier payload validation
//!
//! `give`/`set`/`clear_item`/etc. accept `impl Display`/`impl Into<String>`
//! item payloads that may carry item-component/NBT syntax
//! (`minecraft:diamond_sword[custom_name='"Foo"']`). The validated path
//! checks only the leading `namespace:path` item ID for resource-location
//! shape; any trailing `[...]`/`{...}` component/NBT payload is an
//! intentional raw escape hatch and is not re-parsed. `modify`'s modifier
//! argument is validated the same way, with an optional leading `#` for
//! item-modifier tags.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

use crate::error::{CommandError, CommandResult};
use crate::execute_args::ItemSlot;
use crate::render::{CommandProfile, Validate};
use crate::selector::Selector;
use crate::validate;

// ── Pre-write export re-validation registry ─────────────────────────────────
//
// Mirrors `crate::blocks`'s `BlockCommandNode` registry: a rendered line's
// typed node is retained here so the export pipeline's
// `validate_collected_line` can re-validate the *typed node* (not re-parse
// the rendered string) even though `Inventory`'s ordinary methods return
// plain rendered `String`s once collected into a function body.
#[derive(Debug, Clone)]
enum InventoryCommandNode {
    Give {
        item: String,
    },
    GiveCount {
        item: String,
        count: u32,
    },
    Set {
        slot: ItemSlot,
        item: String,
    },
    SetCount {
        slot: ItemSlot,
        item: String,
        count: u32,
    },
    ClearSlot {
        slot: ItemSlot,
    },
    ClearItem {
        item: String,
    },
    ClearItemCount {
        item: String,
        // Retained for symmetry/debuggability with the other `*Count`
        // variants even though `0` is deliberately never rejected — see
        // `InventoryCommandNode::validate`.
        #[allow(dead_code)]
        count: u32,
    },
    CopyFrom {
        slot: ItemSlot,
        source_slot: ItemSlot,
    },
    Modify {
        slot: ItemSlot,
        modifier: String,
    },
}

impl InventoryCommandNode {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        match self {
            Self::Give { item } => validate_item_str(item, "Inventory::give", "item"),
            Self::GiveCount { item, count } => {
                validate_item_str(item, "Inventory::give_count", "item")?;
                validate::positive_u32(*count, "Inventory::give_count", "count")?;
                Ok(())
            }
            Self::Set { slot, item } => {
                validate_write_slot(slot, profile, "Inventory::set", "slot")?;
                validate_item_str(item, "Inventory::set", "item")
            }
            Self::SetCount { slot, item, count } => {
                validate_write_slot(slot, profile, "Inventory::set_count", "slot")?;
                validate_item_str(item, "Inventory::set_count", "item")?;
                validate::positive_u32(*count, "Inventory::set_count", "count")?;
                Ok(())
            }
            Self::ClearSlot { slot } => {
                validate_write_slot(slot, profile, "Inventory::clear_slot", "slot")
            }
            Self::ClearItem { item } => validate_item_str(item, "Inventory::clear_item", "item"),
            Self::ClearItemCount { item, .. } => {
                // A count of `0` is meaningful here — vanilla `clear` with an
                // explicit `0` count reports the matching stack count without
                // removing anything, so it is not rejected (see module docs).
                validate_item_str(item, "Inventory::clear_item_count", "item")
            }
            Self::CopyFrom { slot, source_slot } => {
                validate_write_slot(slot, profile, "Inventory::copy_from", "slot")?;
                validate_write_slot(source_slot, profile, "Inventory::copy_from", "source_slot")
            }
            Self::Modify { slot, modifier } => {
                validate_write_slot(slot, profile, "Inventory::modify", "slot")?;
                validate_modifier_str(modifier, "Inventory::modify", "modifier")
            }
        }
    }
}

fn registered_inventory_lines() -> &'static Mutex<BTreeMap<String, InventoryCommandNode>> {
    static LINES: OnceLock<Mutex<BTreeMap<String, InventoryCommandNode>>> = OnceLock::new();
    LINES.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn register_inventory_line(line: &str, node: InventoryCommandNode) {
    registered_inventory_lines()
        .lock()
        .expect("inventory command registry poisoned")
        .insert(line.to_owned(), node);
}

/// Re-validate a previously rendered inventory command line's typed node
/// against `profile`, if this module rendered it. Lines this module did not
/// render (including hand-written raw inventory commands) are left alone —
/// the same "unknown lines pass through" contract every other registered
/// family (`nbt`, `execute_ir`, `blocks`, particles, sound, display, text,
/// effect) uses.
pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    let node = registered_inventory_lines()
        .lock()
        .expect("inventory command registry poisoned")
        .get(line)
        .cloned();
    if let Some(node) = node {
        node.validate(profile)?;
    }
    Ok(())
}

/// Clear the process-global pre-write re-validation registry.
///
/// This registry is process-global (an `OnceLock<Mutex<BTreeMap<..>>>`), so
/// without an explicit reset, stale entries from an earlier export in the
/// same process persist forever: if a later, unrelated export happens to
/// render a command line with byte-identical text to a previously
/// registered (and possibly now-invalid, e.g. wildcard-write) line, that
/// unrelated line would be re-validated against the *stale* typed node
/// instead of being treated as an ordinary/raw line — a false positive or
/// false negative unrelated to the current export.
///
/// The export pipeline calls this once at the start of every export (see
/// `sand-core`'s `try_export_components_impl`), mirroring
/// `sand_components::dialog::reset_dialog_callbacks_for_export`'s
/// per-export reset for the dialog-callback registry. Safe to call before
/// any `Inventory` methods run for the export: registration always happens
/// synchronously when a line is rendered, so resetting up front never
/// discards state a later step in the *same* export still needs.
pub fn reset_registry_for_export() {
    registered_inventory_lines()
        .lock()
        .expect("inventory command registry poisoned")
        .clear();
}

// ── Validation helpers ───────────────────────────────────────────────────────

/// Reject a wildcard slot (valid only in read/check contexts) and enforce
/// [`ItemSlot`]'s shared bounds validation for single-slot write commands.
fn validate_write_slot(
    slot: &ItemSlot,
    profile: &CommandProfile,
    helper: &'static str,
    field: &'static str,
) -> CommandResult<()> {
    Validate::validate(slot, profile).map_err(|e| e.with_context(helper))?;
    if slot.is_wildcard() {
        return Err(CommandError::new(
            helper,
            field,
            format!(
                "wildcard slot `{slot}` is only valid in read/check contexts (e.g. `execute if items`); a single-slot write command requires exactly one resolved slot"
            ),
        )
        .with_code("SAND-INVENTORY-WILDCARD-WRITE"));
    }
    Ok(())
}

/// Validate an item payload's leading `namespace:path` ID. Any trailing
/// `[...]`/`{...}` item-component/NBT payload is preserved verbatim as a raw
/// escape hatch (see module docs).
fn validate_item_str(item: &str, helper: &'static str, field: &'static str) -> CommandResult<()> {
    validate::non_empty(item, helper, field)?;
    let id_part = item.find(['[', '{']).map_or(item, |i| &item[..i]);
    validate::resource_location_shape(id_part, helper, field)?;
    Ok(())
}

/// Validate an item-modifier reference: a `namespace:path` resource
/// location, optionally prefixed with `#` for an item-modifier tag.
fn validate_modifier_str(
    modifier: &str,
    helper: &'static str,
    field: &'static str,
) -> CommandResult<()> {
    let id = modifier.strip_prefix('#').unwrap_or(modifier);
    validate::resource_location_shape(id, helper, field)?;
    Ok(())
}

// ── Inventory ─────────────────────────────────────────────────────────────────

/// Fluent inventory operations for an entity selector.
#[derive(Debug, Clone)]
pub struct Inventory {
    selector: Selector,
}

impl Inventory {
    /// Create an inventory handle for the given entity selector.
    pub fn of(selector: Selector) -> Self {
        Self { selector }
    }

    // ── Give / set ────────────────────────────────────────────────────────

    /// `give <selector> <item>` — add an item to the entity's inventory.
    ///
    /// Never panics. The rendered line is re-validated at export time; use
    /// [`Inventory::try_give`] to fail fast instead.
    pub fn give(&self, item: impl fmt::Display) -> String {
        let item = item.to_string();
        let line = format!("give {} {item}", self.selector);
        register_inventory_line(&line, InventoryCommandNode::Give { item });
        line
    }

    /// Fallible [`Inventory::give`] — validates the item ID shape first.
    pub fn try_give(&self, item: impl fmt::Display) -> CommandResult<String> {
        let item = item.to_string();
        validate_item_str(&item, "Inventory::try_give", "item")?;
        Ok(self.give(item))
    }

    /// `give <selector> <item> <count>` — add `count` copies of an item.
    ///
    /// Never panics. The rendered line is re-validated at export time; use
    /// [`Inventory::try_give_count`] to fail fast instead.
    pub fn give_count(&self, item: impl fmt::Display, count: u32) -> String {
        let item = item.to_string();
        let line = format!("give {} {item} {count}", self.selector);
        register_inventory_line(&line, InventoryCommandNode::GiveCount { item, count });
        line
    }

    /// Fallible [`Inventory::give_count`] — validates the item ID shape and
    /// rejects a count of `0` (a zero-count `give` is a no-op Minecraft
    /// rejects rather than accepting).
    pub fn try_give_count(&self, item: impl fmt::Display, count: u32) -> CommandResult<String> {
        let item = item.to_string();
        validate_item_str(&item, "Inventory::try_give_count", "item")?;
        validate::positive_u32(count, "Inventory::try_give_count", "count")?;
        Ok(self.give_count(item, count))
    }

    /// `item replace entity <selector> <slot> with <item>` — overwrite a slot.
    ///
    /// Accepts any type that converts to [`ItemSlot`]. Never panics on an
    /// out-of-range or wildcard slot — the rendered line is re-validated at
    /// export time; use [`Inventory::try_set`] to fail fast instead.
    pub fn set(&self, slot: impl Into<ItemSlot>, item: impl fmt::Display) -> String {
        let slot = slot.into();
        let item = item.to_string();
        let line = format!("item replace entity {} {slot} with {item}", self.selector);
        register_inventory_line(&line, InventoryCommandNode::Set { slot, item });
        line
    }

    /// Fallible [`Inventory::set`] — rejects out-of-range/wildcard slots and
    /// malformed item IDs.
    pub fn try_set(
        &self,
        slot: impl Into<ItemSlot>,
        item: impl fmt::Display,
    ) -> CommandResult<String> {
        let slot = slot.into();
        let item = item.to_string();
        validate_write_slot(
            &slot,
            &CommandProfile::unprofiled(),
            "Inventory::try_set",
            "slot",
        )?;
        validate_item_str(&item, "Inventory::try_set", "item")?;
        Ok(self.set(slot, item))
    }

    /// `item replace entity <selector> <slot> with <item> <count>` — overwrite with a stack size.
    ///
    /// Never panics; use [`Inventory::try_set_count`] to fail fast instead.
    pub fn set_count(
        &self,
        slot: impl Into<ItemSlot>,
        item: impl fmt::Display,
        count: u32,
    ) -> String {
        let slot = slot.into();
        let item = item.to_string();
        let line = format!(
            "item replace entity {} {slot} with {item} {count}",
            self.selector
        );
        register_inventory_line(&line, InventoryCommandNode::SetCount { slot, item, count });
        line
    }

    /// Fallible [`Inventory::set_count`] — rejects out-of-range/wildcard
    /// slots, malformed item IDs, and a count of `0`.
    pub fn try_set_count(
        &self,
        slot: impl Into<ItemSlot>,
        item: impl fmt::Display,
        count: u32,
    ) -> CommandResult<String> {
        let slot = slot.into();
        let item = item.to_string();
        validate_write_slot(
            &slot,
            &CommandProfile::unprofiled(),
            "Inventory::try_set_count",
            "slot",
        )?;
        validate_item_str(&item, "Inventory::try_set_count", "item")?;
        validate::positive_u32(count, "Inventory::try_set_count", "count")?;
        Ok(self.set_count(slot, item, count))
    }

    // ── Clear ─────────────────────────────────────────────────────────────

    /// `item replace entity <selector> <slot> with air` — empty a specific slot.
    ///
    /// Never panics; use [`Inventory::try_clear_slot`] to fail fast instead.
    pub fn clear_slot(&self, slot: impl Into<ItemSlot>) -> String {
        let slot = slot.into();
        let line = format!("item replace entity {} {slot} with air", self.selector);
        register_inventory_line(&line, InventoryCommandNode::ClearSlot { slot });
        line
    }

    /// Fallible [`Inventory::clear_slot`] — rejects out-of-range/wildcard slots.
    pub fn try_clear_slot(&self, slot: impl Into<ItemSlot>) -> CommandResult<String> {
        let slot = slot.into();
        validate_write_slot(
            &slot,
            &CommandProfile::unprofiled(),
            "Inventory::try_clear_slot",
            "slot",
        )?;
        Ok(self.clear_slot(slot))
    }

    /// `clear <selector> <item>` — remove all stacks of a specific item.
    ///
    /// Never panics; use [`Inventory::try_clear_item`] to fail fast instead.
    pub fn clear_item(&self, item: impl Into<String>) -> String {
        let item = item.into();
        let line = format!("clear {} {}", self.selector, item);
        register_inventory_line(&line, InventoryCommandNode::ClearItem { item });
        line
    }

    /// Fallible [`Inventory::clear_item`] — validates the item ID shape.
    pub fn try_clear_item(&self, item: impl Into<String>) -> CommandResult<String> {
        let item = item.into();
        validate_item_str(&item, "Inventory::try_clear_item", "item")?;
        Ok(self.clear_item(item))
    }

    /// `clear <selector> <item> <count>` — remove up to `count` of an item.
    ///
    /// A count of `0` is meaningful vanilla syntax: it reports the matching
    /// count without removing anything, so it is not rejected.
    /// Never panics; use [`Inventory::try_clear_item_count`] to fail fast instead.
    pub fn clear_item_count(&self, item: impl Into<String>, count: u32) -> String {
        let item = item.into();
        let line = format!("clear {} {} {count}", self.selector, item);
        register_inventory_line(&line, InventoryCommandNode::ClearItemCount { item, count });
        line
    }

    /// Fallible [`Inventory::clear_item_count`] — validates the item ID shape.
    pub fn try_clear_item_count(
        &self,
        item: impl Into<String>,
        count: u32,
    ) -> CommandResult<String> {
        let item = item.into();
        validate_item_str(&item, "Inventory::try_clear_item_count", "item")?;
        Ok(self.clear_item_count(item, count))
    }

    /// `clear <selector>` — remove everything from the inventory.
    pub fn clear_all(&self) -> String {
        format!("clear {}", self.selector)
    }

    // ── Copy ──────────────────────────────────────────────────────────────

    /// Copy the item in `source_slot` of another entity into `slot` of this entity.
    ///
    /// Never panics; use [`Inventory::try_copy_from`] to fail fast instead.
    pub fn copy_from(
        &self,
        slot: impl Into<ItemSlot>,
        source: Selector,
        source_slot: impl Into<ItemSlot>,
    ) -> String {
        let slot = slot.into();
        let source_slot = source_slot.into();
        let line = format!(
            "item replace entity {} {slot} from entity {source} {source_slot}",
            self.selector
        );
        register_inventory_line(&line, InventoryCommandNode::CopyFrom { slot, source_slot });
        line
    }

    /// Fallible [`Inventory::copy_from`] — validates both the destination
    /// and source slots (neither may be out-of-range or a wildcard: a
    /// single-item copy resolves to exactly one slot on each side).
    pub fn try_copy_from(
        &self,
        slot: impl Into<ItemSlot>,
        source: Selector,
        source_slot: impl Into<ItemSlot>,
    ) -> CommandResult<String> {
        let slot = slot.into();
        let source_slot = source_slot.into();
        let profile = CommandProfile::unprofiled();
        validate_write_slot(&slot, &profile, "Inventory::try_copy_from", "slot")?;
        validate_write_slot(
            &source_slot,
            &profile,
            "Inventory::try_copy_from",
            "source_slot",
        )?;
        Ok(self.copy_from(slot, source, source_slot))
    }

    // ── Modify ────────────────────────────────────────────────────────────

    /// `item modify entity <selector> <slot> <modifier>` — apply an item modifier.
    ///
    /// Never panics; use [`Inventory::try_modify`] to fail fast instead.
    pub fn modify(&self, slot: impl Into<ItemSlot>, modifier: impl Into<String>) -> String {
        let slot = slot.into();
        let modifier = modifier.into();
        let line = format!("item modify entity {} {slot} {}", self.selector, modifier);
        register_inventory_line(&line, InventoryCommandNode::Modify { slot, modifier });
        line
    }

    /// Fallible [`Inventory::modify`] — rejects out-of-range/wildcard slots
    /// and modifier references that are not `namespace:path`-shaped (an
    /// optional leading `#` for item-modifier tags is accepted).
    pub fn try_modify(
        &self,
        slot: impl Into<ItemSlot>,
        modifier: impl Into<String>,
    ) -> CommandResult<String> {
        let slot = slot.into();
        let modifier = modifier.into();
        validate_write_slot(
            &slot,
            &CommandProfile::unprofiled(),
            "Inventory::try_modify",
            "slot",
        )?;
        validate_modifier_str(&modifier, "Inventory::try_modify", "modifier")?;
        Ok(self.modify(slot, modifier))
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

    // ── No-panic on invalid slots (issue #172) ──────────────────────────────

    #[test]
    fn invalid_slot_bounds_do_not_panic() {
        // These previously panicked via `check_slot_bounds`; they must now
        // simply produce (invalid) command text, caught by validation instead.
        let _ = inv().set(ItemSlot::Hotbar(99), "minecraft:stone");
        let _ = inv().set_count(ItemSlot::Inventory(99), "minecraft:stone", 1);
        let _ = inv().clear_slot(ItemSlot::Container(200));
        let _ = inv().copy_from(
            ItemSlot::Hotbar(99),
            Selector::nearest_player(),
            ItemSlot::MainHand,
        );
        let _ = inv().modify(ItemSlot::Hotbar(99), "minecraft:example");
    }

    // ── Fallible validation ──────────────────────────────────────────────────

    #[test]
    fn try_set_rejects_out_of_range_slot() {
        assert!(
            inv()
                .try_set(ItemSlot::Hotbar(9), "minecraft:stone")
                .is_err()
        );
        assert!(
            inv()
                .try_set(ItemSlot::Inventory(27), "minecraft:stone")
                .is_err()
        );
        assert!(
            inv()
                .try_set(ItemSlot::Container(54), "minecraft:stone")
                .is_err()
        );
    }

    #[test]
    fn try_set_rejects_wildcard_slot() {
        assert!(
            inv()
                .try_set(ItemSlot::AnyHotbar, "minecraft:stone")
                .is_err()
        );
        assert!(
            inv()
                .try_set(ItemSlot::raw("custom.*"), "minecraft:stone")
                .is_err()
        );
    }

    #[test]
    fn try_set_accepts_valid_slot_and_matches_infallible_output() {
        let result = inv().try_set(ItemSlot::MainHand, "minecraft:diamond_sword");
        assert_eq!(
            result.unwrap(),
            inv().set(ItemSlot::MainHand, "minecraft:diamond_sword")
        );
    }

    #[test]
    fn try_set_count_rejects_zero_count() {
        assert!(
            inv()
                .try_set_count(ItemSlot::MainHand, "minecraft:stone", 0)
                .is_err()
        );
    }

    #[test]
    fn try_give_rejects_malformed_item_id() {
        assert!(inv().try_give("Diamond").is_err());
        assert!(inv().try_give("diamond").is_err());
        assert!(inv().try_give("").is_err());
    }

    #[test]
    fn try_give_accepts_component_syntax_as_escape_hatch() {
        assert_eq!(
            inv()
                .try_give("minecraft:diamond_sword[custom_name='\"Foo\"']")
                .unwrap(),
            "give @s minecraft:diamond_sword[custom_name='\"Foo\"']"
        );
    }

    #[test]
    fn try_give_count_rejects_zero() {
        assert!(inv().try_give_count("minecraft:diamond", 0).is_err());
        assert!(inv().try_give_count("minecraft:diamond", 1).is_ok());
    }

    #[test]
    fn try_clear_item_count_allows_zero() {
        // Vanilla `clear` with an explicit `0` count is a valid "report the
        // matching count without clearing" query, not an error.
        assert!(inv().try_clear_item_count("minecraft:diamond", 0).is_ok());
    }

    #[test]
    fn try_clear_item_rejects_malformed_item_id() {
        assert!(inv().try_clear_item("not valid").is_err());
    }

    #[test]
    fn try_copy_from_validates_source_and_destination_slots() {
        assert!(
            inv()
                .try_copy_from(
                    ItemSlot::Hotbar(99),
                    Selector::nearest_player(),
                    ItemSlot::MainHand
                )
                .is_err(),
            "invalid destination slot must be rejected"
        );
        assert!(
            inv()
                .try_copy_from(
                    ItemSlot::MainHand,
                    Selector::nearest_player(),
                    ItemSlot::AnyHotbar
                )
                .is_err(),
            "wildcard source slot must be rejected"
        );
        assert!(
            inv()
                .try_copy_from(
                    ItemSlot::Container(0),
                    Selector::nearest_player(),
                    ItemSlot::MainHand
                )
                .is_ok()
        );
    }

    #[test]
    fn try_modify_validates_slot_and_modifier() {
        assert!(
            inv()
                .try_modify(ItemSlot::AnyHotbar, "minecraft:example")
                .is_err()
        );
        assert!(
            inv()
                .try_modify(ItemSlot::MainHand, "not a modifier")
                .is_err()
        );
        assert_eq!(
            inv()
                .try_modify(ItemSlot::MainHand, "minecraft:example")
                .unwrap(),
            "item modify entity @s weapon.mainhand minecraft:example"
        );
    }

    #[test]
    fn try_modify_accepts_tag_prefixed_modifier() {
        assert!(
            inv()
                .try_modify(ItemSlot::MainHand, "#minecraft:example_tag")
                .is_ok()
        );
    }

    // ── Exporter pre-write validation (registered line re-validation) ───────

    #[test]
    fn exporter_revalidation_rejects_wildcard_write_from_infallible_path() {
        let profile = CommandProfile::unprofiled();
        // The infallible `set` call must not panic, but the line it renders
        // is registered so export-time validation still catches the wildcard
        // write.
        let line = inv().set(ItemSlot::AnyHotbar, "minecraft:stone");
        assert!(validate_registered_line(&line, &profile).is_err());
    }

    #[test]
    fn exporter_revalidation_rejects_out_of_range_slot_from_infallible_path() {
        let profile = CommandProfile::unprofiled();
        let line = inv().clear_slot(ItemSlot::Container(200));
        assert!(validate_registered_line(&line, &profile).is_err());
    }

    #[test]
    fn exporter_revalidation_accepts_valid_infallible_lines() {
        let profile = CommandProfile::unprofiled();
        let line = inv().set(ItemSlot::MainHand, "minecraft:diamond_sword");
        assert_eq!(validate_registered_line(&line, &profile).unwrap(), ());
    }

    #[test]
    fn exporter_revalidation_ignores_unregistered_lines() {
        let profile = CommandProfile::unprofiled();
        assert!(validate_registered_line("say hello", &profile).is_ok());
    }
}
