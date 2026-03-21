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

// ── Anchor ────────────────────────────────────────────────────────────────────

/// Entity anchor point for `execute anchored` and `execute facing entity`.
///
/// Controls whether position calculations are relative to the entity's
/// **eye level** or **foot level**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// `eyes` — the entity's eye/head level.
    Eyes,
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

/// Axis combination for `execute align` — specifies which coordinate axes to
/// floor to block boundaries.
#[derive(Debug, Clone)]
pub struct Swizzle(String);

impl Swizzle {
    /// `x` — floor the X coordinate only.
    pub fn x() -> Self {
        Swizzle("x".into())
    }
    /// `y` — floor the Y coordinate only.
    pub fn y() -> Self {
        Swizzle("y".into())
    }
    /// `z` — floor the Z coordinate only.
    pub fn z() -> Self {
        Swizzle("z".into())
    }
    /// `xy` — floor both X and Y coordinates.
    pub fn xy() -> Self {
        Swizzle("xy".into())
    }
    /// `xz` — floor both X and Z coordinates.
    pub fn xz() -> Self {
        Swizzle("xz".into())
    }
    /// `yz` — floor both Y and Z coordinates.
    pub fn yz() -> Self {
        Swizzle("yz".into())
    }
    /// `xyz` — floor all three coordinates.
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

/// The NBT data type used when writing a value via `execute store result/success … nbt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NbtStoreKind {
    /// `byte` — 8-bit signed integer.
    Byte,
    /// `short` — 16-bit signed integer.
    Short,
    /// `int` — 32-bit signed integer.
    Int,
    /// `long` — 64-bit signed integer.
    Long,
    /// `float` — 32-bit floating-point.
    Float,
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

/// An inventory slot specifier for `execute if items entity/block`.
///
/// Unlike [`InventorySlot`](crate::inventory::InventorySlot), `ItemSlot` supports
/// wildcard variants that match any slot in a category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemSlot {
    // ── Armor ─────────────────────────────────────────────────────────────────
    /// `armor.head` — the helmet slot.
    Head,
    /// `armor.chest` — the chestplate slot.
    Chest,
    /// `armor.legs` — the leggings slot.
    Legs,
    /// `armor.feet` — the boots slot.
    Feet,
    /// `armor.*` — any one of the four armor slots.
    AnyArmor,

    // ── Weapon ────────────────────────────────────────────────────────────────
    /// `weapon.mainhand` — the main hand slot.
    MainHand,
    /// `weapon.offhand` — the off-hand slot.
    OffHand,
    /// `weapon.*` — either the main hand or off-hand slot.
    AnyWeapon,

    // ── Hotbar ────────────────────────────────────────────────────────────────
    /// `hotbar.<n>` — a specific hotbar slot (0 … 8).
    Hotbar(u8),
    /// `hotbar.*` — any of the 9 hotbar slots.
    AnyHotbar,

    // ── Main inventory ────────────────────────────────────────────────────────
    /// `inventory.<n>` — a specific main inventory slot (0 … 26).
    Inventory(u8),
    /// `inventory.*` — any main inventory slot.
    AnyInventory,

    // ── Container ─────────────────────────────────────────────────────────────
    /// `container.<n>` — a container slot by index (0 … 53).
    Container(u8),
    /// `container.*` — any slot in a container.
    AnyContainer,

    // ── Mount equipment ────────────────────────────────────────────────────────
    /// `horse.saddle` — saddle slot on rideable mobs.
    HorseSaddle,
    /// `horse.chest` — chest slot on donkeys and llamas.
    HorseChest,
    /// `horse.armor` — armor slot on horses.
    HorseArmor,
    /// `horse.*` — any horse equipment slot.
    AnyHorse,

    // ── Villager ──────────────────────────────────────────────────────────────
    /// `villager.*` — any villager trade slot.
    AnyVillager,

    // ── Raw ───────────────────────────────────────────────────────────────────
    /// A raw slot string for slots not covered by the above variants.
    Raw(String),
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
}
