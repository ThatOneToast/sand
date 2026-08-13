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
//! live 1.21.4/26.2 server as part of this change. They are Sand's
//! best-confidence encoding of long-documented vanilla structure, not a
//! certified claim — see `docs/testing/participant-role-evidence.md` for
//! what has and has not been verified on a real server.

use std::fmt;

use sand_commands::coord::BlockPos;
use sand_commands::execute_args::ItemSlot;
use sand_commands::nbt::{DataCommand, DataTarget, NbtPath, NbtRef, UntypedNbt};
use sand_commands::selector::Selector;

use crate::condition::Condition;
use sand_components::EquipmentSlot;

/// Deterministic short label naming the *kind* of location, used in
/// diagnostics and canonical rendering — never a raw path fragment.
type LocationKind = &'static str;

/// A validated player inventory slot index (`0..=35`, matching vanilla's
/// `Inventory` list: `0..=8` hotbar, `9..=35` main inventory).
#[sand_macros::api(registry = sand_api_contract, path = "sand::inventory::InventoryIndex", aliases = ["sand::item::InventoryIndex", "sand::item::location::InventoryIndex", "sand::prelude::InventoryIndex"], summary = "Validates a slot in a player's complete 36-slot inventory.", context = "This distinct type prevents a general player-inventory index from being confused with a hotbar-only or block-container index.", minecraft = "Maps to the Inventory[{Slot:N}] entity NBT entry used by vanilla inventory commands.", use_when = ["Addressing any numbered player inventory slot from 0 through 35"], avoid_when = ["Addressing only the hotbar or a block container"], example = "let slot = InventoryIndex::new(9)?;")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InventoryIndex(u8);

impl InventoryIndex {
    #[sand_macros::api(kind = "associated_const", registry = sand_api_contract, path = "sand::inventory::InventoryIndex::MAX", summary = "The highest valid player inventory index.", context = "It exposes the vanilla 36-slot range without repeating a magic number.", minecraft = "Matches the final Inventory list slot, index 35.", use_when = ["Validating generated UI or iteration bounds"], avoid_when = ["Representing a hotbar-only bound"], example = "assert_eq!(InventoryIndex::MAX, 35);")]
    pub const MAX: u8 = 35;

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::InventoryIndex::new", summary = "Validates a player inventory index.", context = "The constructor keeps out-of-range slot numbers from reaching generated item or NBT commands.", minecraft = "Accepts vanilla Inventory slots 0 through 35.", use_when = ["Converting a dynamic slot number into a typed location component"], avoid_when = ["A HotbarIndex or ContainerIndex states the intended range more clearly"], params(index = "The zero-based player inventory slot."), returns = "The validated inventory index or a range error.", example = "let slot = InventoryIndex::new(9)?;")]
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

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::InventoryIndex::get", summary = "Returns the validated zero-based inventory slot.", context = "Use this only at APIs that explicitly require the vanilla numeric representation.", minecraft = "Returns the value rendered as an Inventory slot number.", use_when = ["Interoperating with a typed API that takes a raw validated index"], avoid_when = ["Reconstructing inventory command syntax by hand"], returns = "The slot number in 0 through 35.", example = "assert_eq!(InventoryIndex::new(9)?.get(), 9);")]
    pub fn get(self) -> u8 {
        self.0
    }
}

/// A validated hotbar slot index (`0..=8`). Distinct from
/// [`InventoryIndex`] for a self-documenting call site even though both
/// resolve to the same `Inventory[{Slot:N}]` addressing — slot `0..=8`
/// *is* the hotbar in vanilla's `Inventory` list, there is no separate
/// hotbar-only NBT structure.
#[sand_macros::api(registry = sand_api_contract, path = "sand::inventory::HotbarIndex", aliases = ["sand::item::HotbarIndex", "sand::item::location::HotbarIndex", "sand::prelude::HotbarIndex"], summary = "Validates one of the nine player hotbar slots.", context = "A dedicated hotbar type documents that a location must be currently quick-accessible rather than any inventory slot.", minecraft = "Maps to Inventory slots 0 through 8, Minecraft's hotbar range.", use_when = ["Addressing a selected quick-access inventory position"], avoid_when = ["Addressing the full main inventory"], example = "let slot = HotbarIndex::new(0)?;")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HotbarIndex(u8);

impl HotbarIndex {
    #[sand_macros::api(kind = "associated_const", registry = sand_api_contract, path = "sand::inventory::HotbarIndex::MAX", summary = "The highest valid hotbar index.", context = "It records the fixed nine-slot quick-access range.", minecraft = "Matches hotbar slot 8.", use_when = ["Checking a hotbar index bound"], avoid_when = ["Checking all inventory slots"], example = "assert_eq!(HotbarIndex::MAX, 8);")]
    pub const MAX: u8 = 8;

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::HotbarIndex::new", summary = "Validates a hotbar slot index.", context = "It prevents a main-inventory or container index from being silently accepted as a quick-access slot.", minecraft = "Accepts vanilla hotbar slots 0 through 8.", use_when = ["Constructing a PlayerHotbar location"], avoid_when = ["Addressing a non-hotbar inventory slot"], params(index = "The zero-based hotbar slot."), returns = "The validated hotbar index or a range error.", example = "let slot = HotbarIndex::new(4)?;")]
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

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::HotbarIndex::get", summary = "Returns the validated hotbar slot number.", context = "This provides the numeric value while retaining validation at construction.", minecraft = "Returns the Inventory-list index for the hotbar slot.", use_when = ["Passing a validated hotbar index to a typed adapter"], avoid_when = ["Formatting raw item command syntax"], returns = "The slot number in 0 through 8.", example = "assert_eq!(HotbarIndex::new(4)?.get(), 4);")]
    pub fn get(self) -> u8 {
        self.0
    }
}

/// A validated block-container slot index (`0..=53`, matching the widest
/// vanilla single-block container — a double chest — and
/// [`sand_commands::ItemSlot::Container`]'s existing validated bound, reused
/// here for consistency rather than picking an independent limit).
#[sand_macros::api(registry = sand_api_contract, path = "sand::inventory::ContainerIndex", aliases = ["sand::item::ContainerIndex", "sand::item::location::ContainerIndex", "sand::prelude::ContainerIndex"], summary = "Validates a slot in a vanilla block container.", context = "Block containers use a separate range from player inventory, so the type makes container addressing explicit.", minecraft = "Maps to an Items list entry for a container block entity, bounded by a double chest's 54 slots.", use_when = ["Addressing a chest or another supported block container slot"], avoid_when = ["Addressing a player inventory position"], example = "let slot = ContainerIndex::new(0)?;")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContainerIndex(u8);

impl ContainerIndex {
    #[sand_macros::api(kind = "associated_const", registry = sand_api_contract, path = "sand::inventory::ContainerIndex::MAX", summary = "The largest supported block-container slot index.", context = "Sand uses the widest ordinary vanilla single-container bound to reject impossible locations early.", minecraft = "Matches index 53 of a 54-slot double chest.", use_when = ["Checking a container slot bound"], avoid_when = ["Assuming every container has this many slots"], example = "assert_eq!(ContainerIndex::MAX, 53);")]
    pub const MAX: u8 = 53;

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::ContainerIndex::new", summary = "Validates a block-container slot index.", context = "The constructor prevents impossible container locations from reaching generated data or item commands.", minecraft = "Accepts the zero-based range 0 through 53 used by Sand's supported container model.", use_when = ["Constructing a BlockContainer location"], avoid_when = ["Addressing a player inventory slot"], params(index = "The zero-based block-container slot."), returns = "The validated container index or a range error.", example = "let slot = ContainerIndex::new(27)?;")]
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

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::ContainerIndex::get", summary = "Returns the validated container slot number.", context = "This exposes the numeric form only after range validation.", minecraft = "Returns the index rendered inside a block entity's Items list path.", use_when = ["Passing the index to a typed container adapter"], avoid_when = ["Writing a raw block-entity NBT path"], returns = "The slot number in 0 through 53.", example = "assert_eq!(ContainerIndex::new(27)?.get(), 27);")]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::inventory::ItemLocation",
    aliases = ["sand::item::ItemLocation", "sand::item::location::ItemLocation", "sand::prelude::ItemLocation"],
    summary = "Names a validated live Minecraft item location.",
    context = "The enum separates mutable player, entity, block-container, and dropped-item locations from immutable ItemSnapshot evidence.",
    minecraft = "Lowers locations to the verified entity or block NBT path and, where vanilla supports it, an /item or execute-if-items target.",
    use_when = ["Reading, matching, replacing, or capturing a live item stack", "Making player versus external entity and block ownership explicit"],
    avoid_when = ["Keeping event-time item evidence after the source can change", "Using an unchecked raw NBT slot path"],
    variants(PlayerMainHand = "The executing player's selected hotbar stack.", PlayerOffHand = "The executing player's offhand stack.", PlayerEquipment = "One validated armor slot on the executing player.", PlayerHotbar = "One validated hotbar slot on the executing player.", PlayerInventory = "One validated general inventory slot on the executing player.", EntityEquipment = "A validated equipment slot on an explicitly selected living entity.", BlockContainer = "A validated slot in an explicitly positioned block container.", ItemEntity = "The item compound carried by an explicitly selected dropped-item entity.", EntityInventory = "A discoverable inventory slot on an explicitly selected entity.", EntityEnderChest = "An ender-chest entry with NBT access but no vanilla /item target."),
    variant_fields(PlayerEquipment = ["The validated player armor equipment slot."], PlayerHotbar = ["The validated player hotbar index."], PlayerInventory = ["The validated player inventory index."], EntityEquipment(entity = "The explicit entity owning the equipment.", slot = "The validated equipment slot."), BlockContainer(position = "The block position owning the container.", slot = "The validated container slot."), ItemEntity = ["The explicit selector for the dropped-item entity."], EntityInventory(entity = "The explicit entity owning the inventory.", slot = "The typed discoverable inventory slot."), EntityEnderChest(entity = "The explicit entity owning the ender chest.", slot = "The validated ender-chest index.")),
    example = "let location = ItemLocation::PlayerMainHand;"
)]
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
    /// A discoverable slot produced by [`ItemLocation::entity`].
    EntityInventory {
        entity: Selector,
        slot: EntityInventorySlot,
    },
    /// An ender-chest entry. Vanilla exposes this through entity NBT but not
    /// as an `/item` command slot, so read/snapshot operations are supported
    /// while live `/item replace` and `execute if items` are rejected.
    EntityEnderChest {
        entity: Selector,
        slot: EnderChestIndex,
    },
}

/// One entity inventory slot with both live `/item` and NBT addressing.
#[sand_macros::api(registry = sand_api_contract, path = "sand::inventory::EntityInventorySlot", aliases = ["sand::item::EntityInventorySlot", "sand::item::location::EntityInventorySlot", "sand::prelude::EntityInventorySlot"], summary = "Selects a discoverable live slot on an explicitly targeted entity.", context = "This enum is the internal shape exposed by ItemLocation::entity so callers can retain typed equipment and inventory distinctions.", minecraft = "Maps each variant to the corresponding entity NBT path and, when supported, vanilla /item slot.", use_when = ["Inspecting an explicitly selected entity's named or indexed slot"], avoid_when = ["Addressing the executing player's simple ItemLocation variants"], variants(SelectedItem = "The entity's selected item when that NBT field exists.", MainHand = "The entity main hand.", OffHand = "The entity offhand.", Head = "The head equipment slot.", Chest = "The chest equipment slot.", Legs = "The leg equipment slot.", Feet = "The feet equipment slot.", Hotbar = "A validated hotbar slot.", MainInventory = "A validated non-hotbar main inventory slot.", Inventory = "A validated complete player inventory slot."), variant_fields(Hotbar = ["The validated hotbar index."], MainInventory = ["The validated main-inventory index."], Inventory = ["The validated general inventory index."]), example = "let slot = EntityInventorySlot::MainHand;")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityInventorySlot {
    SelectedItem,
    MainHand,
    OffHand,
    Head,
    Chest,
    Legs,
    Feet,
    Hotbar(HotbarIndex),
    MainInventory(MainInventoryIndex),
    Inventory(InventoryIndex),
}

/// Validated main-inventory index (`0..=26`, excluding the hotbar).
#[sand_macros::api(registry = sand_api_contract, path = "sand::inventory::MainInventoryIndex", aliases = ["sand::item::MainInventoryIndex", "sand::item::location::MainInventoryIndex", "sand::prelude::MainInventoryIndex"], summary = "Validates a non-hotbar player main-inventory offset.", context = "This type names the 27 slots after the hotbar, avoiding ambiguity with the combined InventoryIndex range.", minecraft = "Maps offsets 0 through 26 to player Inventory slots 9 through 35.", use_when = ["Selecting an entity main-inventory slot without the hotbar"], avoid_when = ["Selecting the combined 0 through 35 inventory range"], example = "let slot = MainInventoryIndex::new(0)?;")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MainInventoryIndex(u8);

impl MainInventoryIndex {
    #[sand_macros::api(kind = "associated_const", registry = sand_api_contract, path = "sand::inventory::MainInventoryIndex::MAX", summary = "The highest valid main-inventory offset.", context = "The fixed 27-slot range excludes the nine hotbar positions.", minecraft = "Maps to player Inventory slot 35 after the hotbar offset.", use_when = ["Checking main-inventory bounds"], avoid_when = ["Checking full inventory indices"], example = "assert_eq!(MainInventoryIndex::MAX, 26);")]
    pub const MAX: u8 = 26;

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::MainInventoryIndex::new", summary = "Validates a main-inventory offset.", context = "The constructor retains the distinction between the 27-slot main inventory and the hotbar.", minecraft = "Accepts offsets 0 through 26, rendered after the nine-slot hotbar.", use_when = ["Constructing EntityInventory::main_inventory"], avoid_when = ["Addressing a hotbar or full inventory slot"], params(index = "The zero-based main-inventory offset."), returns = "The validated offset or a range error.", example = "let slot = MainInventoryIndex::new(12)?;")]
    pub fn new(index: u8) -> Result<Self, ItemLocationError> {
        if index > Self::MAX {
            return Err(ItemLocationError::IndexOutOfRange {
                location_kind: "player main inventory slot",
                index: u32::from(index),
                max: u32::from(Self::MAX),
            });
        }
        Ok(Self(index))
    }

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::MainInventoryIndex::get", summary = "Returns the validated main-inventory offset.", context = "It exposes the offset before Sand translates it to the underlying Inventory list slot.", minecraft = "Returns a value in 0 through 26.", use_when = ["Adapting a typed main-inventory offset"], avoid_when = ["Formatting raw NBT paths"], returns = "The zero-based main-inventory offset.", example = "assert_eq!(MainInventoryIndex::new(12)?.get(), 12);")]
    pub fn get(self) -> u8 {
        self.0
    }
}

/// Validated ender-chest index (`0..=26`).
#[sand_macros::api(registry = sand_api_contract, path = "sand::inventory::EnderChestIndex", aliases = ["sand::item::EnderChestIndex", "sand::item::location::EnderChestIndex", "sand::prelude::EnderChestIndex"], summary = "Validates one of an entity's 27 ender-chest entries.", context = "Ender chests have NBT addressing but are deliberately not modeled as a vanilla /item command target.", minecraft = "Maps to the entity EnderItems list at an index from 0 through 26.", use_when = ["Reading or capturing an entity ender-chest item through NBT"], avoid_when = ["Replacing a live item with /item; vanilla has no ender-chest slot target"], example = "let slot = EnderChestIndex::new(0)?;")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnderChestIndex(u8);

impl EnderChestIndex {
    #[sand_macros::api(kind = "associated_const", registry = sand_api_contract, path = "sand::inventory::EnderChestIndex::MAX", summary = "The highest valid ender-chest index.", context = "It captures the fixed 27-entry ender chest layout.", minecraft = "Matches EnderItems index 26.", use_when = ["Checking ender-chest bounds"], avoid_when = ["Checking normal container bounds"], example = "assert_eq!(EnderChestIndex::MAX, 26);")]
    pub const MAX: u8 = 26;

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::EnderChestIndex::new", summary = "Validates an ender-chest index.", context = "The constructor prevents an invalid EnderItems NBT entry from reaching generated data commands.", minecraft = "Accepts entries 0 through 26 of an entity EnderItems list.", use_when = ["Constructing EntityInventory::ender_chest"], avoid_when = ["Addressing a block container or ordinary inventory"], params(index = "The zero-based ender-chest entry."), returns = "The validated index or a range error.", example = "let slot = EnderChestIndex::new(5)?;")]
    pub fn new(index: u8) -> Result<Self, ItemLocationError> {
        if index > Self::MAX {
            return Err(ItemLocationError::IndexOutOfRange {
                location_kind: "ender chest slot",
                index: u32::from(index),
                max: u32::from(Self::MAX),
            });
        }
        Ok(Self(index))
    }

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::EnderChestIndex::get", summary = "Returns the validated ender-chest entry.", context = "It exposes the bounded numeric position while retaining construction-time validation.", minecraft = "Returns an EnderItems index in 0 through 26.", use_when = ["Adapting a typed ender-chest location"], avoid_when = ["Treating it as a vanilla /item slot"], returns = "The zero-based ender-chest entry.", example = "assert_eq!(EnderChestIndex::new(5)?.get(), 5);")]
    pub fn get(self) -> u8 {
        self.0
    }
}

/// Factory handle for entity inventory locations. The produced
/// [`ItemLocation`] remains the sole live-location representation.
#[derive(Debug, Clone)]
pub struct EntityInventory {
    entity: Selector,
}

/// Factory handle for block inventory locations.
#[derive(Debug, Clone)]
pub struct BlockInventory {
    position: BlockPos,
}

impl ItemLocation {
    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::ItemLocation::entity", summary = "Starts typed live-inventory addressing for one selected entity.", context = "The factory keeps the entity selector attached while the caller chooses an inventory or equipment slot.", minecraft = "Uses the selector as the target for generated item or NBT commands.", use_when = ["Addressing an explicitly selected entity inventory"], avoid_when = ["Addressing the executing player's standard slot; use a Player variant"], params(entity = "The entity selector owning the inventory."), returns = "An entity-inventory factory handle.", example = "let inventory = ItemLocation::entity(Selector::nearest_player());")]
    pub fn entity(entity: Selector) -> EntityInventory {
        EntityInventory { entity }
    }

    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::ItemLocation::block", summary = "Starts typed live-inventory addressing for one block container.", context = "The factory binds a block position before the caller chooses a validated container slot.", minecraft = "Uses the position as the target for generated block item and data commands.", use_when = ["Addressing a chest or another supported block container"], avoid_when = ["Addressing an entity inventory"], params(position = "The container block position."), returns = "A block-inventory factory handle.", example = "let chest = ItemLocation::block(BlockPos::new(0, 64, 0));")]
    pub fn block(position: BlockPos) -> BlockInventory {
        BlockInventory { position }
    }

    /// A player armor location — [`EquipmentSlot::Head`],
    /// [`EquipmentSlot::Chest`], [`EquipmentSlot::Legs`], or
    /// [`EquipmentSlot::Feet`] only. `Mainhand`/`Offhand` are rejected (use
    /// [`ItemLocation::PlayerMainHand`]/[`ItemLocation::PlayerOffHand`]);
    /// `Body` is rejected (it does not apply to a player).
    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::ItemLocation::player_equipment", summary = "Validates an armor location on the executing player.", context = "Main hand, offhand, and body are intentionally rejected so callers cannot generate ambiguous player equipment paths.", minecraft = "Maps head, chest, legs, or feet to the corresponding player Inventory entry.", use_when = ["Capturing or matching a player's armor slot"], avoid_when = ["Addressing a hand or non-player entity equipment"], params(slot = "The requested player armor equipment slot."), returns = "The validated player equipment location or a precise unsupported-location error.", example = "let helmet = ItemLocation::player_equipment(EquipmentSlot::Head)?;")]
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
    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::ItemLocation::entity_equipment", summary = "Validates an equipment location on an explicitly selected entity.", context = "The constructor preserves entity ownership and rejects Body because Sand has not verified a stable backing NBT path for it.", minecraft = "Maps supported armor and hand slots to ArmorItems or HandItems on the selected entity.", use_when = ["Reading or matching equipment on a non-player entity"], avoid_when = ["Addressing the executing player's conventional armor slot"], params(entity = "The entity selector owning the equipment.", slot = "The requested equipment slot."), returns = "The validated entity-equipment location or an unsupported-location error.", example = "let weapon = ItemLocation::entity_equipment(Selector::nearest_entity(), EquipmentSlot::Mainhand)?;")]
    pub fn entity_equipment(
        entity: Selector,
        slot: EquipmentSlot,
    ) -> Result<Self, ItemLocationError> {
        if matches!(slot, EquipmentSlot::Body) {
            return Err(ItemLocationError::UnsupportedLocation {
                location: "EntityEquipment(Body)".to_string(),
                reason: "the Body equipment slot's backing NBT tag is not verified for this phase",
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
            Self::EntityInventory { .. } => "entity_inventory",
            Self::EntityEnderChest { .. } => "entity_ender_chest",
        }
    }

    /// Whether this location is scoped to the executing subject (`@s`)
    /// rather than an explicit external [`Selector`]/[`BlockPos`]. A location
    /// with `is_self_scoped() == false` names its own target explicitly.
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

    /// Canonical typed NBT view of this live item location.
    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::ItemLocation::nbt", summary = "Returns the typed NBT view of this live item stack.", context = "The view is for typed data operations; it avoids exposing a hand-written NBT source string at normal call sites.", minecraft = "Addresses the matching entity or block item compound.", use_when = ["Copying a live item compound with typed NBT builders"], avoid_when = ["Capturing event-time evidence; use ItemSnapshot::capture"], returns = "The untyped NBT reference for the live item compound.", example = "let item_nbt = ItemLocation::PlayerMainHand.nbt();")]
    pub fn nbt(&self) -> NbtRef<UntypedNbt> {
        let (target, path) = self
            .nbt_source()
            .expect("constructible ItemLocation variants always have NBT addressing");
        NbtRef::new(target, NbtPath::new(path))
    }

    /// Snapshot/copy the current stack compound into a typed NBT destination.
    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::ItemLocation::copy_to", summary = "Builds a typed command copying this live item compound to NBT.", context = "Use it for deliberate data transfer while the source is still live; snapshots are safer for event-time evidence.", minecraft = "Renders a data modify set-from command from the location's item compound.", use_when = ["Copying a live item into a typed NBT destination"], avoid_when = ["Persisting an event observation after later commands can mutate the source"], params(destination = "The typed NBT destination."), returns = "The data command that performs the copy.", example = "let command = ItemLocation::PlayerMainHand.copy_to(&destination);")]
    pub fn copy_to<T>(&self, destination: &NbtRef<T>) -> DataCommand {
        destination.copy_from(&self.nbt())
    }

    /// Copy NBT into a block container slot.
    ///
    /// Entity/player inventory mutation is deliberately rejected here because
    /// vanilla does not safely permit arbitrary player NBT writes. Use
    /// [`replace_from`](Self::replace_from) for live entity inventory copies.
    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::inventory::ItemLocation::copy_from", summary = "Builds a typed NBT copy into a block-container item slot.", context = "Sand deliberately rejects entity and player inventory NBT writes because vanilla does not safely permit arbitrary live-inventory mutation through data commands.", minecraft = "Renders data modify only for a supported block container target.", use_when = ["Writing a complete item compound into a block container"], avoid_when = ["Mutating player or entity inventory; use replace_from for live stack copies"], params(source = "The typed NBT source item compound."), returns = "The data command or an unsupported-location error.", example = "let command = chest.slot(0)?.copy_from(&source)?;")]
    pub fn copy_from<T>(&self, source: &NbtRef<T>) -> Result<DataCommand, ItemLocationError> {
        match self {
            Self::BlockContainer { .. } => Ok(self.nbt().copy_from(source)),
            _ => Err(ItemLocationError::UnsupportedLocation {
                location: self.kind().to_string(),
                reason: "NBT writes to live entity/player inventory are unsafe; use ItemLocation::replace_from for item-to-item copies",
            }),
        }
    }

    /// Typed `execute if data` existence check for the stack compound.
    pub fn exists(&self) -> Condition {
        let reference = self.nbt();
        Condition::data_exists(&reference)
    }

    /// True when the live slot contains no item.
    pub fn is_empty(&self) -> Result<Condition, ItemLocationError> {
        Ok(!self.matches("*")?)
    }

    /// Match a live slot through typed `execute if items` condition IR.
    ///
    /// `item` is the vanilla item-stack predicate argument (an item ID,
    /// tag/wildcard, or component-bearing predicate syntax).
    pub fn matches(&self, item: impl Into<String>) -> Result<Condition, ItemLocationError> {
        let item = item.into();
        match self.item_target_slot()? {
            ItemCommandLocation::Entity { target, slot } => {
                Ok(Condition::items_entity(target, slot, item))
            }
            ItemCommandLocation::Block { position, slot } => {
                Ok(Condition::items_block(position, slot, item))
            }
        }
    }

    /// Copy a live stack using vanilla `/item replace ... from ...`.
    pub fn replace_from(&self, source: &ItemLocation) -> Result<String, ItemLocationError> {
        let destination = self.item_target_slot()?;
        let source = source.item_target_slot()?;
        Ok(format!(
            "item replace {} {} from {} {}",
            destination.target_text(),
            destination.slot(),
            source.target_text(),
            source.slot()
        ))
    }

    fn item_target_slot(&self) -> Result<ItemCommandLocation, ItemLocationError> {
        match self {
            Self::PlayerMainHand => Ok(ItemCommandLocation::Entity {
                target: Selector::self_(),
                slot: ItemSlot::MainHand,
            }),
            Self::PlayerOffHand => Ok(ItemCommandLocation::Entity {
                target: Selector::self_(),
                slot: ItemSlot::OffHand,
            }),
            Self::PlayerEquipment(slot) => Ok(ItemCommandLocation::Entity {
                target: Selector::self_(),
                slot: equipment_item_slot(*slot)?,
            }),
            Self::PlayerHotbar(index) => Ok(ItemCommandLocation::Entity {
                target: Selector::self_(),
                slot: ItemSlot::Hotbar(index.get()),
            }),
            Self::PlayerInventory(index) => Ok(ItemCommandLocation::Entity {
                target: Selector::self_(),
                slot: inventory_item_slot(*index),
            }),
            Self::EntityEquipment { entity, slot } => Ok(ItemCommandLocation::Entity {
                target: entity.clone(),
                slot: equipment_item_slot(*slot)?,
            }),
            Self::BlockContainer { position, slot } => Ok(ItemCommandLocation::Block {
                position: position.clone(),
                slot: ItemSlot::Container(slot.get()),
            }),
            Self::ItemEntity(entity) => Ok(ItemCommandLocation::Entity {
                target: entity.clone(),
                slot: ItemSlot::raw("contents"),
            }),
            Self::EntityInventory { entity, slot } => Ok(ItemCommandLocation::Entity {
                target: entity.clone(),
                slot: slot.item_slot(),
            }),
            Self::EntityEnderChest { .. } => Err(ItemLocationError::UnsupportedLocation {
                location: self.kind().to_string(),
                reason: "ender chest entries have NBT addressing but no vanilla `/item` slot",
            }),
        }
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
            Self::EntityInventory { entity, slot } => {
                (DataTarget::entity(entity.clone()), slot.nbt_path())
            }
            Self::EntityEnderChest { entity, slot } => (
                DataTarget::entity(entity.clone()),
                format!("EnderItems[{{Slot:{}b}}]", slot.get()),
            ),
        })
    }
}

impl EntityInventory {
    fn location(&self, slot: EntityInventorySlot) -> ItemLocation {
        ItemLocation::EntityInventory {
            entity: self.entity.clone(),
            slot,
        }
    }

    pub fn selected_item(&self) -> ItemLocation {
        self.location(EntityInventorySlot::SelectedItem)
    }

    pub fn mainhand(&self) -> ItemLocation {
        self.location(EntityInventorySlot::MainHand)
    }

    pub fn offhand(&self) -> ItemLocation {
        self.location(EntityInventorySlot::OffHand)
    }

    pub fn helmet(&self) -> ItemLocation {
        self.location(EntityInventorySlot::Head)
    }

    pub fn chestplate(&self) -> ItemLocation {
        self.location(EntityInventorySlot::Chest)
    }

    pub fn leggings(&self) -> ItemLocation {
        self.location(EntityInventorySlot::Legs)
    }

    pub fn boots(&self) -> ItemLocation {
        self.location(EntityInventorySlot::Feet)
    }

    pub fn hotbar(&self, index: u8) -> Result<ItemLocation, ItemLocationError> {
        Ok(self.location(EntityInventorySlot::Hotbar(HotbarIndex::new(index)?)))
    }

    pub fn main_inventory(&self, index: u8) -> Result<ItemLocation, ItemLocationError> {
        Ok(
            self.location(EntityInventorySlot::MainInventory(MainInventoryIndex::new(
                index,
            )?)),
        )
    }

    pub fn slot(&self, index: u8) -> Result<ItemLocation, ItemLocationError> {
        Ok(self.location(EntityInventorySlot::Inventory(InventoryIndex::new(index)?)))
    }

    pub fn ender_chest(&self, index: u8) -> Result<ItemLocation, ItemLocationError> {
        Ok(ItemLocation::EntityEnderChest {
            entity: self.entity.clone(),
            slot: EnderChestIndex::new(index)?,
        })
    }
}

impl BlockInventory {
    pub fn slot(&self, index: u8) -> Result<ItemLocation, ItemLocationError> {
        Ok(ItemLocation::BlockContainer {
            position: self.position.clone(),
            slot: ContainerIndex::new(index)?,
        })
    }
}

impl EntityInventorySlot {
    fn item_slot(self) -> ItemSlot {
        match self {
            Self::SelectedItem | Self::MainHand => ItemSlot::MainHand,
            Self::OffHand => ItemSlot::OffHand,
            Self::Head => ItemSlot::Head,
            Self::Chest => ItemSlot::Chest,
            Self::Legs => ItemSlot::Legs,
            Self::Feet => ItemSlot::Feet,
            Self::Hotbar(index) => ItemSlot::Hotbar(index.get()),
            Self::MainInventory(index) => ItemSlot::Inventory(index.get()),
            Self::Inventory(index) => inventory_item_slot(index),
        }
    }

    fn nbt_path(self) -> String {
        match self {
            Self::SelectedItem | Self::MainHand => "SelectedItem".to_string(),
            Self::OffHand => "Inventory[{Slot:-106b}]".to_string(),
            Self::Head => "Inventory[{Slot:103b}]".to_string(),
            Self::Chest => "Inventory[{Slot:102b}]".to_string(),
            Self::Legs => "Inventory[{Slot:101b}]".to_string(),
            Self::Feet => "Inventory[{Slot:100b}]".to_string(),
            Self::Hotbar(index) => format!("Inventory[{{Slot:{}b}}]", index.get()),
            Self::MainInventory(index) => {
                format!("Inventory[{{Slot:{}b}}]", index.get() + 9)
            }
            Self::Inventory(index) => {
                format!("Inventory[{{Slot:{}b}}]", index.get())
            }
        }
    }
}

enum ItemCommandLocation {
    Entity { target: Selector, slot: ItemSlot },
    Block { position: BlockPos, slot: ItemSlot },
}

impl ItemCommandLocation {
    fn target_text(&self) -> String {
        match self {
            Self::Entity { target, .. } => format!("entity {target}"),
            Self::Block { position, .. } => format!("block {position}"),
        }
    }

    fn slot(&self) -> &ItemSlot {
        match self {
            Self::Entity { slot, .. } | Self::Block { slot, .. } => slot,
        }
    }
}

fn equipment_item_slot(slot: EquipmentSlot) -> Result<ItemSlot, ItemLocationError> {
    match slot {
        EquipmentSlot::Head => Ok(ItemSlot::Head),
        EquipmentSlot::Chest => Ok(ItemSlot::Chest),
        EquipmentSlot::Legs => Ok(ItemSlot::Legs),
        EquipmentSlot::Feet => Ok(ItemSlot::Feet),
        EquipmentSlot::Mainhand => Ok(ItemSlot::MainHand),
        EquipmentSlot::Offhand => Ok(ItemSlot::OffHand),
        EquipmentSlot::Body => Err(ItemLocationError::UnsupportedLocation {
            location: "EquipmentSlot::Body".to_string(),
            reason: "body equipment is not verified across Sand's supported profiles",
        }),
    }
}

fn inventory_item_slot(index: InventoryIndex) -> ItemSlot {
    if index.get() <= 8 {
        ItemSlot::Hotbar(index.get())
    } else {
        ItemSlot::Inventory(index.get() - 9)
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
                reason: "the Body equipment slot's backing NBT tag is not verified for this phase",
            });
        }
    })
}

/// A validated, actionable diagnostic for [`ItemLocation`] construction or
/// resolution failure. Always names the requested location and the specific
/// unsupported behavior — never a generic "unsupported" message.
#[sand_macros::api(registry = sand_api_contract, path = "sand::inventory::ItemLocationError", aliases = ["sand::item::ItemLocationError", "sand::item::location::ItemLocationError"], summary = "Explains why a typed live item location is invalid or unsupported.", context = "The error preserves the requested slot kind and limitation so authors can choose a valid typed location instead of receiving an opaque command failure.", minecraft = "Prevents generation of out-of-range NBT paths or unsupported live /item targets.", use_when = ["Handling a fallible slot constructor or live location operation"], avoid_when = ["Representing an empty slot; that is a runtime item condition"], variants(IndexOutOfRange = "The requested numeric slot exceeded its validated range.", UnsupportedLocation = "Minecraft or Sand cannot safely represent the requested location operation."), variant_fields(IndexOutOfRange(location_kind = "The category of slot that rejected the index.", index = "The requested index.", max = "The highest supported index."), UnsupportedLocation(location = "The requested location description.", reason = "The specific unsupported Minecraft behavior.")), example = "let slot = HotbarIndex::new(9); ")]
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
    fn discoverable_entity_and_block_inventory_locations_share_nbt() {
        let inventory = ItemLocation::entity(Selector::self_());
        assert_eq!(
            inventory.mainhand().nbt().get().to_string(),
            "data get entity @s SelectedItem"
        );
        assert_eq!(
            inventory.hotbar(3).unwrap().nbt().as_str(),
            "Inventory[{Slot:3b}]"
        );
        assert_eq!(
            inventory.main_inventory(2).unwrap().nbt().as_str(),
            "Inventory[{Slot:11b}]"
        );
        assert_eq!(
            inventory.ender_chest(4).unwrap().nbt().as_str(),
            "EnderItems[{Slot:4b}]"
        );
        assert_eq!(
            ItemLocation::block(BlockPos::here())
                .slot(0)
                .unwrap()
                .nbt()
                .as_str(),
            "Items[{Slot:0b}]"
        );
    }

    #[test]
    fn inventory_copy_matching_and_empty_use_correct_command_families() {
        let entity = ItemLocation::entity(Selector::self_());
        let mainhand = entity.mainhand();
        let cache = sand_commands::Nbt::storage("pack:cache").path("last_item");
        assert_eq!(
            mainhand.copy_to(&cache).to_string(),
            "data modify storage pack:cache last_item set from entity @s SelectedItem"
        );

        let block = ItemLocation::block(BlockPos::here()).slot(0).unwrap();
        assert_eq!(
            block.replace_from(&mainhand).unwrap(),
            "item replace block ~ ~ ~ container.0 from entity @s weapon.mainhand"
        );
        assert_eq!(
            mainhand
                .matches("minecraft:diamond_sword")
                .unwrap()
                .execute_commands(false, "say yes"),
            vec!["execute if items entity @s weapon.mainhand minecraft:diamond_sword run say yes"]
        );
        assert_eq!(
            mainhand
                .is_empty()
                .unwrap()
                .execute_commands(false, "say empty"),
            vec!["execute unless items entity @s weapon.mainhand * run say empty"]
        );
    }

    #[test]
    fn inventory_indices_and_unsafe_player_nbt_writes_are_rejected() {
        assert!(ItemLocation::entity(Selector::self_()).hotbar(9).is_err());
        assert!(
            ItemLocation::entity(Selector::self_())
                .main_inventory(27)
                .is_err()
        );
        assert!(ItemLocation::block(BlockPos::here()).slot(54).is_err());
        let source = sand_commands::Nbt::storage("pack:data").path("item");
        assert!(
            ItemLocation::entity(Selector::self_())
                .mainhand()
                .copy_from(&source)
                .is_err()
        );
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
