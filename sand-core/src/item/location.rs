//! Typed, validated item-bearing locations (#229 Phase 7).
//!
//! [`ItemLocation`] names *where* an item stack lives — a player's hand, an
//! equipment slot, an inventory index, a block container slot, or an item
//! entity's own stack — without reading or mutating it. It is the addressing
//! half of the item-snapshot model; [`super::snapshot::ItemSnapshot`] is the
//! captured-data half.
//!
//! # NBT path stability
//!
//! Every location resolves to a vanilla entity/block-entity NBT path
//! (`SelectedItem`, `Inventory[{Slot:N}]`, `ArmorItems[N]`, `HandItems[N]`,
//! `Items[{Slot:N}]`, `Item`). These are long-stable *structural* NBT tags —
//! unrelated to the 1.20.5+ item-component encoding change, which affects
//! only the *contents* of an item compound, not which entity/block-entity
//! tag holds it. Rendering is therefore version-independent by design; no
//! `VersionProfile` parameter is threaded through [`ItemLocation`] itself.
//! This is a deliberate, documented simplification — see
//! `ItemLocation::EntityEquipment`'s `Body` slot, which is explicitly
//! unsupported below because its backing tag is genuinely uncertain across
//! the supported version range, rather than guessed.
//!
//! None of these paths have been independently runtime-verified against a
//! live 1.21.4/26.2 server as part of this change — see `LIM-VAL-008` in
//! `ai/known-limitations.md`. They are Sand's best-confidence encoding of
//! long-documented vanilla structure, not a certified claim.

use std::fmt;

use sand_commands::coord::BlockPos;
use sand_commands::nbt::DataTarget;
use sand_commands::selector::Selector;

use sand_components::EquipmentSlot;

/// Deterministic short label naming the *kind* of location, used in
/// diagnostics and canonical rendering — never a raw path fragment.
type LocationKind = &'static str;

/// A validated player inventory slot index (`0..=35`, matching vanilla's
/// `Inventory` list: `0..=8` hotbar, `9..=35` main inventory).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InventoryIndex(u8);

impl InventoryIndex {
    pub const MAX: u8 = 35;

    pub fn new(index: u8) -> Result<Self, ItemLocationError> {
        if index > Self::MAX {
            return Err(ItemLocationError::IndexOutOfRange {
                location_kind: "player inventory slot",
                index: u32::from(index),
                max: u32::from(Self::MAX),
            });
        }
        Ok(Self(index))
    }

    pub fn get(self) -> u8 {
        self.0
    }
}

/// A validated hotbar slot index (`0..=8`). Distinct from
/// [`InventoryIndex`] for a self-documenting call site even though both
/// resolve to the same `Inventory[{Slot:N}]` addressing — slot `0..=8`
/// *is* the hotbar in vanilla's `Inventory` list, there is no separate
/// hotbar-only NBT structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HotbarIndex(u8);

impl HotbarIndex {
    pub const MAX: u8 = 8;

    pub fn new(index: u8) -> Result<Self, ItemLocationError> {
        if index > Self::MAX {
            return Err(ItemLocationError::IndexOutOfRange {
                location_kind: "player hotbar slot",
                index: u32::from(index),
                max: u32::from(Self::MAX),
            });
        }
        Ok(Self(index))
    }

    pub fn get(self) -> u8 {
        self.0
    }
}

/// A validated block-container slot index (`0..=53`, matching the widest
/// vanilla single-block container — a double chest — and
/// [`sand_commands::ItemSlot::Container`]'s existing validated bound, reused
/// here for consistency rather than picking an independent limit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContainerIndex(u8);

impl ContainerIndex {
    pub const MAX: u8 = 53;

    pub fn new(index: u8) -> Result<Self, ItemLocationError> {
        if index > Self::MAX {
            return Err(ItemLocationError::IndexOutOfRange {
                location_kind: "block container slot",
                index: u32::from(index),
                max: u32::from(Self::MAX),
            });
        }
        Ok(Self(index))
    }

    pub fn get(self) -> u8 {
        self.0
    }
}

/// A typed, validated item-bearing location.
///
/// Construct via the associated functions (`player_equipment`,
/// `entity_equipment`) where validation is required; the remaining variants
/// are directly constructible since every field is already a validated
/// type. Never render `.nbt_source()`'s output into a hand-written command —
/// use [`super::snapshot::ItemSnapshot::capture`], which composes it with
/// `DataModify`/`Execute` typed builders.
#[derive(Debug, Clone)]
pub enum ItemLocation {
    /// The player's currently-selected hotbar item (`SelectedItem`).
    PlayerMainHand,
    /// The player's offhand slot (`Inventory[{Slot:-106b}]`).
    PlayerOffHand,
    /// A player armor slot. Only `Head`/`Chest`/`Legs`/`Feet` are
    /// constructible — `Mainhand`/`Offhand` have their own dedicated
    /// variants above, and `Body` does not apply to players. Construct via
    /// [`ItemLocation::player_equipment`], which enforces this.
    PlayerEquipment(EquipmentSlot),
    /// A player hotbar slot by validated index.
    PlayerHotbar(HotbarIndex),
    /// A player main-inventory-or-hotbar slot by validated index (the full
    /// `0..=35` vanilla `Inventory` range).
    PlayerInventory(InventoryIndex),
    /// An equipment slot on an arbitrary living entity (`ArmorItems`/
    /// `HandItems`), addressed by [`Selector`] rather than assumed to be the
    /// executing player. Construct via [`ItemLocation::entity_equipment`].
    EntityEquipment {
        entity: Selector,
        slot: EquipmentSlot,
    },
    /// A slot inside a block container's inventory (e.g. a chest), by
    /// validated index into that block entity's `Items` list.
    BlockContainer {
        position: BlockPos,
        slot: ContainerIndex,
    },
    /// An item entity's own stack (the `Item` compound on a dropped-item
    /// entity), addressed by [`Selector`].
    ItemEntity(Selector),
}

impl ItemLocation {
    /// A player armor location — [`EquipmentSlot::Head`],
    /// [`EquipmentSlot::Chest`], [`EquipmentSlot::Legs`], or
    /// [`EquipmentSlot::Feet`] only. `Mainhand`/`Offhand` are rejected (use
    /// [`ItemLocation::PlayerMainHand`]/[`ItemLocation::PlayerOffHand`]);
    /// `Body` is rejected (it does not apply to a player).
    pub fn player_equipment(slot: EquipmentSlot) -> Result<Self, ItemLocationError> {
        match slot {
            EquipmentSlot::Head
            | EquipmentSlot::Chest
            | EquipmentSlot::Legs
            | EquipmentSlot::Feet => Ok(Self::PlayerEquipment(slot)),
            EquipmentSlot::Mainhand | EquipmentSlot::Offhand => {
                Err(ItemLocationError::UnsupportedLocation {
                    location: format!("PlayerEquipment({slot:?})"),
                    reason: "use ItemLocation::PlayerMainHand/PlayerOffHand instead of a player-scoped EquipmentSlot::Mainhand/Offhand",
                })
            }
            EquipmentSlot::Body => Err(ItemLocationError::UnsupportedLocation {
                location: "PlayerEquipment(Body)".to_string(),
                reason: "the Body equipment slot does not apply to players",
            }),
        }
    }

    /// An equipment location on an arbitrary living entity. All
    /// [`EquipmentSlot`] variants are accepted except [`EquipmentSlot::Body`]
    /// — its backing NBT tag differs from the stable `ArmorItems`/
    /// `HandItems` structure used here in a way this phase has not verified
    /// across the supported version range (see the module doc), so it is
    /// rejected rather than guessed.
    pub fn entity_equipment(
        entity: Selector,
        slot: EquipmentSlot,
    ) -> Result<Self, ItemLocationError> {
        if matches!(slot, EquipmentSlot::Body) {
            return Err(ItemLocationError::UnsupportedLocation {
                location: "EntityEquipment(Body)".to_string(),
                reason: "the Body equipment slot's backing NBT tag is not verified for this phase — see LIM-ITEM-002",
            });
        }
        Ok(Self::EntityEquipment { entity, slot })
    }

    /// A short, stable label for this location's kind — used in
    /// diagnostics and as part of deterministic generated resource keys.
    /// Never includes a selector, position, or index (those vary at
    /// runtime/per-call; the kind alone must be deterministic across equal
    /// variants).
    pub fn kind(&self) -> LocationKind {
        match self {
            Self::PlayerMainHand => "player_main_hand",
            Self::PlayerOffHand => "player_off_hand",
            Self::PlayerEquipment(_) => "player_equipment",
            Self::PlayerHotbar(_) => "player_hotbar",
            Self::PlayerInventory(_) => "player_inventory",
            Self::EntityEquipment { .. } => "entity_equipment",
            Self::BlockContainer { .. } => "block_container",
            Self::ItemEntity(_) => "item_entity",
        }
    }

    /// Whether this location is scoped to the executing subject (`@s`)
    /// rather than an explicit external [`Selector`]/[`BlockPos`]. Used to
    /// diagnose "non-player source under player-only context" — a location
    /// with `is_self_scoped() == false` names its own target explicitly and
    /// is never implicitly bound to whatever `@s` happens to be.
    pub fn is_self_scoped(&self) -> bool {
        matches!(
            self,
            Self::PlayerMainHand
                | Self::PlayerOffHand
                | Self::PlayerEquipment(_)
                | Self::PlayerHotbar(_)
                | Self::PlayerInventory(_)
        )
    }

    /// Resolve this location to a `(DataTarget, NBT get-path)` pair suitable
    /// for the source side of `data modify <dest> <path> set from <target>
    /// <source_path>` (or `if data <target> <path>` for a presence check).
    ///
    /// Returns [`ItemLocationError::UnsupportedLocation`] for any location
    /// this phase cannot resolve exactly (currently none of the
    /// constructible variants — the unsupported cases are rejected earlier,
    /// at construction time, via [`ItemLocation::player_equipment`]/
    /// [`ItemLocation::entity_equipment`] — this method's `Result` exists so
    /// future variants can add fallible resolution without a breaking
    /// signature change).
    pub fn nbt_source(&self) -> Result<(DataTarget, String), ItemLocationError> {
        Ok(match self {
            Self::PlayerMainHand => (
                DataTarget::entity(Selector::self_()),
                "SelectedItem".to_string(),
            ),
            Self::PlayerOffHand => (
                DataTarget::entity(Selector::self_()),
                "Inventory[{Slot:-106b}]".to_string(),
            ),
            Self::PlayerEquipment(slot) => (
                DataTarget::entity(Selector::self_()),
                format!(
                    "Inventory[{{Slot:{}b}}]",
                    player_armor_inventory_slot(*slot)
                ),
            ),
            Self::PlayerHotbar(index) => (
                DataTarget::entity(Selector::self_()),
                format!("Inventory[{{Slot:{}b}}]", index.get()),
            ),
            Self::PlayerInventory(index) => (
                DataTarget::entity(Selector::self_()),
                format!("Inventory[{{Slot:{}b}}]", index.get()),
            ),
            Self::EntityEquipment { entity, slot } => (
                DataTarget::entity(entity.clone()),
                entity_equipment_path(*slot)?,
            ),
            Self::BlockContainer { position, slot } => (
                DataTarget::block(position.clone()),
                format!("Items[{{Slot:{}b}}]", slot.get()),
            ),
            Self::ItemEntity(selector) => {
                (DataTarget::entity(selector.clone()), "Item".to_string())
            }
        })
    }
}

/// Vanilla `Inventory` list `Slot` values for player armor
/// (`100`=feet, `101`=legs, `102`=chest, `103`=head).
fn player_armor_inventory_slot(slot: EquipmentSlot) -> i32 {
    match slot {
        EquipmentSlot::Feet => 100,
        EquipmentSlot::Legs => 101,
        EquipmentSlot::Chest => 102,
        EquipmentSlot::Head => 103,
        // Unreachable via ItemLocation::player_equipment's validation, but
        // exhaustively handled rather than panicking if ever reached
        // through another path.
        EquipmentSlot::Mainhand | EquipmentSlot::Offhand | EquipmentSlot::Body => 103,
    }
}

/// Vanilla `ArmorItems`/`HandItems` list index paths for non-player living
/// entities. `Body` is rejected upstream by
/// [`ItemLocation::entity_equipment`] and never reaches this function.
fn entity_equipment_path(slot: EquipmentSlot) -> Result<String, ItemLocationError> {
    Ok(match slot {
        EquipmentSlot::Feet => "ArmorItems[0]".to_string(),
        EquipmentSlot::Legs => "ArmorItems[1]".to_string(),
        EquipmentSlot::Chest => "ArmorItems[2]".to_string(),
        EquipmentSlot::Head => "ArmorItems[3]".to_string(),
        EquipmentSlot::Mainhand => "HandItems[0]".to_string(),
        EquipmentSlot::Offhand => "HandItems[1]".to_string(),
        EquipmentSlot::Body => {
            return Err(ItemLocationError::UnsupportedLocation {
                location: "EntityEquipment(Body)".to_string(),
                reason: "the Body equipment slot's backing NBT tag is not verified for this phase — see LIM-ITEM-002",
            });
        }
    })
}

/// A validated, actionable diagnostic for [`ItemLocation`] construction or
/// resolution failure. Always names the requested location and the specific
/// unsupported behavior — never a generic "unsupported" message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemLocationError {
    /// A slot/inventory index was outside its validated range.
    IndexOutOfRange {
        location_kind: &'static str,
        index: u32,
        max: u32,
    },
    /// The requested location is not representable in this phase.
    UnsupportedLocation {
        location: String,
        reason: &'static str,
    },
}

impl fmt::Display for ItemLocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexOutOfRange {
                location_kind,
                index,
                max,
            } => write!(
                f,
                "invalid {location_kind} index {index}: must be in range 0..={max}"
            ),
            Self::UnsupportedLocation { location, reason } => {
                write!(f, "unsupported item location `{location}`: {reason}")
            }
        }
    }
}

impl std::error::Error for ItemLocationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_hand_renders_selected_item() {
        let (target, path) = ItemLocation::PlayerMainHand.nbt_source().unwrap();
        assert_eq!(target.to_string(), "entity @s");
        assert_eq!(path, "SelectedItem");
    }

    #[test]
    fn off_hand_renders_inventory_slot_negative_106() {
        let (target, path) = ItemLocation::PlayerOffHand.nbt_source().unwrap();
        assert_eq!(target.to_string(), "entity @s");
        assert_eq!(path, "Inventory[{Slot:-106b}]");
    }

    #[test]
    fn player_armor_slots_render_canonical_inventory_indices() {
        let cases = [
            (EquipmentSlot::Feet, 100),
            (EquipmentSlot::Legs, 101),
            (EquipmentSlot::Chest, 102),
            (EquipmentSlot::Head, 103),
        ];
        for (slot, expected) in cases {
            let location = ItemLocation::player_equipment(slot).unwrap();
            let (_, path) = location.nbt_source().unwrap();
            assert_eq!(path, format!("Inventory[{{Slot:{expected}b}}]"), "{slot:?}");
        }
    }

    #[test]
    fn player_equipment_rejects_mainhand_offhand_and_body() {
        for slot in [
            EquipmentSlot::Mainhand,
            EquipmentSlot::Offhand,
            EquipmentSlot::Body,
        ] {
            let err = ItemLocation::player_equipment(slot).unwrap_err();
            assert!(matches!(err, ItemLocationError::UnsupportedLocation { .. }));
        }
    }

    #[test]
    fn hotbar_index_rejects_out_of_range() {
        assert!(HotbarIndex::new(8).is_ok());
        let err = HotbarIndex::new(9).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid player hotbar slot index 9: must be in range 0..=8"
        );
    }

    #[test]
    fn inventory_index_rejects_out_of_range() {
        assert!(InventoryIndex::new(35).is_ok());
        let err = InventoryIndex::new(36).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid player inventory slot index 36: must be in range 0..=35"
        );
    }

    #[test]
    fn container_index_rejects_out_of_range() {
        assert!(ContainerIndex::new(53).is_ok());
        let err = ContainerIndex::new(54).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid block container slot index 54: must be in range 0..=53"
        );
    }

    #[test]
    fn hotbar_and_inventory_share_canonical_inventory_slot_addressing() {
        let hotbar = ItemLocation::PlayerHotbar(HotbarIndex::new(3).unwrap());
        let inventory = ItemLocation::PlayerInventory(InventoryIndex::new(3).unwrap());
        assert_eq!(
            hotbar.nbt_source().unwrap().1,
            inventory.nbt_source().unwrap().1
        );
    }

    #[test]
    fn entity_equipment_renders_armor_and_hand_items() {
        let cases = [
            (EquipmentSlot::Feet, "ArmorItems[0]"),
            (EquipmentSlot::Legs, "ArmorItems[1]"),
            (EquipmentSlot::Chest, "ArmorItems[2]"),
            (EquipmentSlot::Head, "ArmorItems[3]"),
            (EquipmentSlot::Mainhand, "HandItems[0]"),
            (EquipmentSlot::Offhand, "HandItems[1]"),
        ];
        for (slot, expected) in cases {
            let location = ItemLocation::entity_equipment(Selector::self_(), slot).unwrap();
            let (_, path) = location.nbt_source().unwrap();
            assert_eq!(path, expected, "{slot:?}");
        }
    }

    #[test]
    fn entity_equipment_rejects_body_slot() {
        let err =
            ItemLocation::entity_equipment(Selector::self_(), EquipmentSlot::Body).unwrap_err();
        assert!(matches!(err, ItemLocationError::UnsupportedLocation { .. }));
        assert!(err.to_string().contains("Body"));
    }

    #[test]
    fn block_container_renders_items_slot() {
        let location = ItemLocation::BlockContainer {
            position: BlockPos::absolute(10, 64, -5),
            slot: ContainerIndex::new(12).unwrap(),
        };
        let (target, path) = location.nbt_source().unwrap();
        assert_eq!(target.to_string(), "block 10 64 -5");
        assert_eq!(path, "Items[{Slot:12b}]");
    }

    #[test]
    fn item_entity_renders_item_compound() {
        let location = ItemLocation::ItemEntity(Selector::self_());
        let (target, path) = location.nbt_source().unwrap();
        assert_eq!(target.to_string(), "entity @s");
        assert_eq!(path, "Item");
    }

    #[test]
    fn location_kind_is_deterministic_and_never_embeds_call_specific_data() {
        assert_eq!(ItemLocation::PlayerMainHand.kind(), "player_main_hand");
        assert_eq!(
            ItemLocation::ItemEntity(Selector::self_()).kind(),
            ItemLocation::ItemEntity(Selector::self_()).kind()
        );
    }

    #[test]
    fn self_scoped_locations_are_identified_correctly() {
        assert!(ItemLocation::PlayerMainHand.is_self_scoped());
        assert!(ItemLocation::PlayerOffHand.is_self_scoped());
        assert!(
            !ItemLocation::EntityEquipment {
                entity: Selector::self_(),
                slot: EquipmentSlot::Head,
            }
            .is_self_scoped()
        );
        assert!(
            !ItemLocation::BlockContainer {
                position: BlockPos::here(),
                slot: ContainerIndex::new(0).unwrap(),
            }
            .is_self_scoped()
        );
        assert!(!ItemLocation::ItemEntity(Selector::self_()).is_self_scoped());
    }
}
