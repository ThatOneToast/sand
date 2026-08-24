//! Argument types used exclusively by the [`Execute`] command chain.
//!
//! | Type | Used by | Purpose |
//! |---|---|---|
//! | [`Anchor`] | `anchored`, `facing entity` | Eye-level vs. foot-level reference point |
//! | [`Swizzle`] | `align` | Which axes to snap to the block grid |
//! | [`NbtStoreKind`] | `store result/success … nbt` | NBT data type for stored values |
//! | [`ItemSlot`] | `if items entity/block` | Inventory slot specifier with wildcard support |
//!
//! [`Execute`]: crate::execute::Execute

use std::fmt;

use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};

// ── Anchor ────────────────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::command::Anchor` for the canonical contract."]
/// Entity anchor point for `execute anchored` and `execute facing entity`.
///
/// Controls whether position calculations are relative to the entity's
/// **eye level** or **foot level**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    #[doc = "**API Contract:** Run `sand api show sand::command::Anchor::Eyes` for the canonical contract."]
    /// `eyes` — the entity's eye/head level.
    Eyes,
    #[doc = "**API Contract:** Run `sand api show sand::command::Anchor::Feet` for the canonical contract."]
    /// `feet` — the entity's foot level (bottom of their bounding box).
    Feet,
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Anchor::Eyes => write!(f, "eyes"),
            Anchor::Feet => write!(f, "feet"),
        }
    }
}

// ── Swizzle ───────────────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::command::Swizzle` for the canonical contract."]
/// Axis combination for `execute align` — specifies which coordinate axes to
/// floor to block boundaries.
#[derive(Debug, Clone)]
pub struct Swizzle(String);

impl Swizzle {
    /// `x` — floor the X coordinate only.
    #[doc = "**API Contract:** Run `sand api show sand::command::Swizzle::x` for the canonical contract."]
    pub fn x() -> Self {
        Swizzle("x".into())
    }
    /// `y` — floor the Y coordinate only.
    #[doc = "**API Contract:** Run `sand api show sand::command::Swizzle::y` for the canonical contract."]
    pub fn y() -> Self {
        Swizzle("y".into())
    }
    /// `z` — floor the Z coordinate only.
    #[doc = "**API Contract:** Run `sand api show sand::command::Swizzle::z` for the canonical contract."]
    pub fn z() -> Self {
        Swizzle("z".into())
    }
    /// `xy` — floor both X and Y coordinates.
    #[doc = "**API Contract:** Run `sand api show sand::command::Swizzle::xy` for the canonical contract."]
    pub fn xy() -> Self {
        Swizzle("xy".into())
    }
    /// `xz` — floor both X and Z coordinates.
    #[doc = "**API Contract:** Run `sand api show sand::command::Swizzle::xz` for the canonical contract."]
    pub fn xz() -> Self {
        Swizzle("xz".into())
    }
    /// `yz` — floor both Y and Z coordinates.
    #[doc = "**API Contract:** Run `sand api show sand::command::Swizzle::yz` for the canonical contract."]
    pub fn yz() -> Self {
        Swizzle("yz".into())
    }
    /// `xyz` — floor all three coordinates.
    #[doc = "**API Contract:** Run `sand api show sand::command::Swizzle::xyz` for the canonical contract."]
    pub fn xyz() -> Self {
        Swizzle("xyz".into())
    }
}

impl fmt::Display for Swizzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── NbtStoreKind ──────────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::command::NbtStoreKind` for the canonical contract."]
/// The NBT data type used when writing a value via `execute store result/success … nbt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NbtStoreKind {
    #[doc = "**API Contract:** Run `sand api show sand::command::NbtStoreKind::Byte` for the canonical contract."]
    /// `byte` — 8-bit signed integer.
    Byte,
    #[doc = "**API Contract:** Run `sand api show sand::command::NbtStoreKind::Short` for the canonical contract."]
    /// `short` — 16-bit signed integer.
    Short,
    #[doc = "**API Contract:** Run `sand api show sand::command::NbtStoreKind::Int` for the canonical contract."]
    /// `int` — 32-bit signed integer.
    Int,
    #[doc = "**API Contract:** Run `sand api show sand::command::NbtStoreKind::Long` for the canonical contract."]
    /// `long` — 64-bit signed integer.
    Long,
    #[doc = "**API Contract:** Run `sand api show sand::command::NbtStoreKind::Float` for the canonical contract."]
    /// `float` — 32-bit floating-point.
    Float,
    #[doc = "**API Contract:** Run `sand api show sand::command::NbtStoreKind::Double` for the canonical contract."]
    /// `double` — 64-bit floating-point.
    Double,
}

impl fmt::Display for NbtStoreKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NbtStoreKind::Byte => "byte",
            NbtStoreKind::Short => "short",
            NbtStoreKind::Int => "int",
            NbtStoreKind::Long => "long",
            NbtStoreKind::Float => "float",
            NbtStoreKind::Double => "double",
        };
        write!(f, "{s}")
    }
}

// ── ItemSlot ──────────────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot` for the canonical contract."]
/// An inventory slot specifier for `execute if items entity/block`.
///
/// `ItemSlot` supports
/// wildcard variants that match any slot in a category.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "slots do nothing until passed to a command"]
pub enum ItemSlot {
    // ── Armor ─────────────────────────────────────────────────────────────────
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Head` for the canonical contract."]
    /// `armor.head` — the helmet slot.
    Head,
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Chest` for the canonical contract."]
    /// `armor.chest` — the chestplate slot.
    Chest,
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Legs` for the canonical contract."]
    /// `armor.legs` — the leggings slot.
    Legs,
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Feet` for the canonical contract."]
    /// `armor.feet` — the boots slot.
    Feet,
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::AnyArmor` for the canonical contract."]
    /// `armor.*` — any one of the four armor slots.
    AnyArmor,

    // ── Weapon ────────────────────────────────────────────────────────────────
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::MainHand` for the canonical contract."]
    /// `weapon.mainhand` — the main hand slot.
    MainHand,
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::OffHand` for the canonical contract."]
    /// `weapon.offhand` — the off-hand slot.
    OffHand,
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::AnyWeapon` for the canonical contract."]
    /// `weapon.*` — either the main hand or off-hand slot.
    AnyWeapon,

    // ── Hotbar ────────────────────────────────────────────────────────────────
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Hotbar` for the canonical contract."]
    /// `hotbar.<n>` — a specific hotbar slot (0 … 8).
    Hotbar(
        #[doc = "The `Hotbar` variant carries the value described by its variant semantics: `hotbar.<n>` — a specific hotbar slot (0 … 8)."]
        #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Hotbar::0` for the canonical contract."]
        u8,
    ),
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::AnyHotbar` for the canonical contract."]
    /// `hotbar.*` — any of the 9 hotbar slots.
    AnyHotbar,

    // ── Main inventory ────────────────────────────────────────────────────────
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Inventory` for the canonical contract."]
    /// `inventory.<n>` — a specific main inventory slot (0 … 26).
    Inventory(
        #[doc = "The `Inventory` variant carries the value described by its variant semantics: `inventory.<n>` — a specific main inventory slot (0 … 26)."]
        #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Inventory::0` for the canonical contract."]
        u8,
    ),
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::AnyInventory` for the canonical contract."]
    /// `inventory.*` — any main inventory slot.
    AnyInventory,

    // ── Container ─────────────────────────────────────────────────────────────
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Container` for the canonical contract."]
    /// `container.<n>` — a container slot by index (0 … 53).
    Container(
        #[doc = "The `Container` variant carries the value described by its variant semantics: `container.<n>` — a container slot by index (0 … 53)."]
        #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Container::0` for the canonical contract."]
        u8,
    ),
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::AnyContainer` for the canonical contract."]
    /// `container.*` — any slot in a container.
    AnyContainer,

    // ── Mount equipment ────────────────────────────────────────────────────────
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::HorseSaddle` for the canonical contract."]
    /// `horse.saddle` — saddle slot on rideable mobs.
    HorseSaddle,
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::HorseChest` for the canonical contract."]
    /// `horse.chest` — chest slot on donkeys and llamas.
    HorseChest,
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::HorseArmor` for the canonical contract."]
    /// `horse.armor` — armor slot on horses.
    HorseArmor,
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::AnyHorse` for the canonical contract."]
    /// `horse.*` — any horse equipment slot.
    AnyHorse,

    // ── Villager ──────────────────────────────────────────────────────────────
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::AnyVillager` for the canonical contract."]
    /// `villager.*` — any villager trade slot.
    AnyVillager,

    // ── Raw ───────────────────────────────────────────────────────────────────
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Raw` for the canonical contract."]
    /// An unchecked raw slot string for slots not covered by the above variants.
    Raw(
        #[doc = "The `Raw` variant carries the value described by its variant semantics: An unchecked raw slot string for slots not covered by the above variants."]
        #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::Raw::0` for the canonical contract."]
        String,
    ),
}

impl fmt::Display for ItemSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: std::borrow::Cow<str> = match self {
            ItemSlot::Head => "armor.head".into(),
            ItemSlot::Chest => "armor.chest".into(),
            ItemSlot::Legs => "armor.legs".into(),
            ItemSlot::Feet => "armor.feet".into(),
            ItemSlot::AnyArmor => "armor.*".into(),
            ItemSlot::MainHand => "weapon.mainhand".into(),
            ItemSlot::OffHand => "weapon.offhand".into(),
            ItemSlot::AnyWeapon => "weapon.*".into(),
            ItemSlot::Hotbar(n) => format!("hotbar.{n}").into(),
            ItemSlot::AnyHotbar => "hotbar.*".into(),
            ItemSlot::Inventory(n) => format!("inventory.{n}").into(),
            ItemSlot::AnyInventory => "inventory.*".into(),
            ItemSlot::Container(n) => format!("container.{n}").into(),
            ItemSlot::AnyContainer => "container.*".into(),
            ItemSlot::HorseSaddle => "horse.saddle".into(),
            ItemSlot::HorseChest => "horse.chest".into(),
            ItemSlot::HorseArmor => "horse.armor".into(),
            ItemSlot::AnyHorse => "horse.*".into(),
            ItemSlot::AnyVillager => "villager.*".into(),
            ItemSlot::Raw(s) => s.as_str().into(),
        };
        write!(f, "{s}")
    }
}

impl ItemSlot {
    /// Explicit raw slot syntax for modded or future slot families.
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::raw` for the canonical contract."]
    pub fn raw(value: impl Into<String>) -> Self {
        Self::Raw(value.into())
    }

    /// Whether this slot matches more than one physical inventory slot.
    ///
    /// Wildcard slots (`armor.*`, `weapon.*`, `hotbar.*`, `inventory.*`,
    /// `container.*`, `horse.*`, `villager.*`, and any [`ItemSlot::Raw`]
    /// ending in `*`) are valid for read/check contexts such as
    /// `execute if items`, but Minecraft's single-slot write grammar
    /// (`item replace`/`item modify`) requires exactly one resolved slot.
    /// See [`Inventory`](crate::inventory::Inventory)'s write-slot validation.
    #[doc = "**API Contract:** Run `sand api show sand::command::ItemSlot::is_wildcard` for the canonical contract."]
    pub fn is_wildcard(&self) -> bool {
        match self {
            Self::AnyArmor
            | Self::AnyWeapon
            | Self::AnyHotbar
            | Self::AnyInventory
            | Self::AnyContainer
            | Self::AnyHorse
            | Self::AnyVillager => true,
            Self::Raw(s) => s.ends_with('*'),
            _ => false,
        }
    }
}

impl Validate for ItemSlot {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        let invalid = match self {
            Self::Hotbar(n) if *n > 8 => Some(("hotbar", *n, 8)),
            Self::Inventory(n) if *n > 26 => Some(("inventory", *n, 26)),
            Self::Container(n) if *n > 53 => Some(("container", *n, 53)),
            _ => None,
        };
        if let Some((family, value, max)) = invalid {
            return Err(CommandError::new(
                "ItemSlot",
                "index",
                format!("{family} slot index must be 0..={max}, got `{value}`"),
            ));
        }
        Ok(())
    }
}

impl RenderCommand for ItemSlot {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_display() {
        assert_eq!(Anchor::Eyes.to_string(), "eyes");
        assert_eq!(Anchor::Feet.to_string(), "feet");
    }

    #[test]
    fn swizzle_display() {
        assert_eq!(Swizzle::x().to_string(), "x");
        assert_eq!(Swizzle::xy().to_string(), "xy");
        assert_eq!(Swizzle::xyz().to_string(), "xyz");
        assert_eq!(Swizzle::xz().to_string(), "xz");
        assert_eq!(Swizzle::yz().to_string(), "yz");
    }

    #[test]
    fn nbt_store_kind_display() {
        assert_eq!(NbtStoreKind::Byte.to_string(), "byte");
        assert_eq!(NbtStoreKind::Int.to_string(), "int");
        assert_eq!(NbtStoreKind::Double.to_string(), "double");
        assert_eq!(NbtStoreKind::Long.to_string(), "long");
    }

    #[test]
    fn item_slot_display() {
        assert_eq!(ItemSlot::Head.to_string(), "armor.head");
        assert_eq!(ItemSlot::Feet.to_string(), "armor.feet");
        assert_eq!(ItemSlot::AnyArmor.to_string(), "armor.*");
        assert_eq!(ItemSlot::MainHand.to_string(), "weapon.mainhand");
        assert_eq!(ItemSlot::AnyWeapon.to_string(), "weapon.*");
        assert_eq!(ItemSlot::Hotbar(3).to_string(), "hotbar.3");
        assert_eq!(ItemSlot::AnyHotbar.to_string(), "hotbar.*");
        assert_eq!(ItemSlot::Container(0).to_string(), "container.0");
        assert_eq!(ItemSlot::AnyContainer.to_string(), "container.*");
        assert_eq!(ItemSlot::HorseSaddle.to_string(), "horse.saddle");
        assert_eq!(ItemSlot::AnyHorse.to_string(), "horse.*");
        assert_eq!(ItemSlot::AnyVillager.to_string(), "villager.*");
        assert_eq!(ItemSlot::Raw("custom.*".into()).to_string(), "custom.*");
    }

    #[test]
    fn item_slot_is_wildcard() {
        assert!(ItemSlot::AnyArmor.is_wildcard());
        assert!(ItemSlot::AnyWeapon.is_wildcard());
        assert!(ItemSlot::AnyHotbar.is_wildcard());
        assert!(ItemSlot::AnyInventory.is_wildcard());
        assert!(ItemSlot::AnyContainer.is_wildcard());
        assert!(ItemSlot::AnyHorse.is_wildcard());
        assert!(ItemSlot::AnyVillager.is_wildcard());
        assert!(ItemSlot::raw("custom.*").is_wildcard());
        assert!(!ItemSlot::MainHand.is_wildcard());
        assert!(!ItemSlot::Hotbar(3).is_wildcard());
        assert!(!ItemSlot::raw("custom.slot").is_wildcard());
    }

    #[test]
    fn item_slot_validation_is_shared() {
        assert!(ItemSlot::Hotbar(9).try_build().is_err());
        assert!(ItemSlot::Inventory(27).try_build().is_err());
        assert!(ItemSlot::Container(54).try_build().is_err());
        assert_eq!(
            ItemSlot::raw("modded.slot").try_build().unwrap(),
            "modded.slot"
        );
    }
}
