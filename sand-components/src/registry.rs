//! Typed registry identifier wrappers.
//!
//! Every entry here wraps a validated [`ResourceLocation`] and provides:
//!
//! - `::minecraft(path)` — construct a `minecraft:` prefixed ID without unwrapping
//! - `::custom(rl)` — wrap any `ResourceLocation` (modded / pack-specific IDs)
//! - `From<ResourceLocation>` — convert a parsed ID directly
//! - `Display` / `Serialize` — emit `namespace:path` strings
//!
//! # Relation to generated enums
//!
//! `sand-core::generated::{Item, Block, EntityType, …}` already provide
//! strongly-typed vanilla constants.  These wrapper types complement them:
//! use the generated enum for vanilla values, and the `*Id` wrappers for
//! modded or pack-specific IDs that are not in the generated list.
//!
//! ```rust
//! use sand_components::registry::ItemId;
//!
//! // Vanilla item (plain resource location)
//! let diamond = ItemId::minecraft("diamond").unwrap();
//! assert_eq!(diamond.to_string(), "minecraft:diamond");
//!
//! // Modded item (custom resource location)
//! use sand_components::ResourceLocation;
//! let custom: ItemId = ResourceLocation::new("mymod", "arcane_sword").unwrap().into();
//! assert_eq!(custom.to_string(), "mymod:arcane_sword");
//! ```

use std::fmt;
use std::marker::PhantomData;

use serde::{Serialize, Serializer};

use crate::error::Result;
use crate::resource_location::ResourceLocation;

// ── Macro to avoid repetition ────────────────────────────────────────────────

macro_rules! registry_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(ResourceLocation);

        impl $name {
            /// Construct a `minecraft:<path>` ID.  Returns an error if `path` is invalid.
            pub fn minecraft(path: impl AsRef<str>) -> Result<Self> {
                Ok(Self(ResourceLocation::minecraft(path)?))
            }

            /// Wrap any [`ResourceLocation`] as this registry ID.
            pub fn custom(rl: ResourceLocation) -> Self {
                Self(rl)
            }

            /// Access the inner [`ResourceLocation`].
            pub fn as_resource_location(&self) -> &ResourceLocation {
                &self.0
            }
        }

        impl From<ResourceLocation> for $name {
            fn from(rl: ResourceLocation) -> Self {
                Self(rl)
            }
        }

        impl From<$name> for ResourceLocation {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
                self.0.serialize(s)
            }
        }

        impl std::str::FromStr for $name {
            type Err = crate::error::SandError;
            fn from_str(s: &str) -> Result<Self> {
                Ok(Self(s.parse()?))
            }
        }
    };
}

registry_id! {
    /// Typed Minecraft item identifier (e.g. `minecraft:diamond_sword` or `mymod:arcane_blade`).
    ///
    /// For vanilla items, prefer the generated `sand_core::generated::Item` enum.
    /// Use `ItemId` for modded items or when you have a `ResourceLocation` at hand.
    ItemId
}

registry_id! {
    /// Typed Minecraft block identifier (e.g. `minecraft:stone` or `mymod:custom_ore`).
    ///
    /// For vanilla blocks, prefer the generated `sand_core::generated::Block` enum.
    BlockId
}

registry_id! {
    /// Typed Minecraft entity type identifier (e.g. `minecraft:zombie` or `mymod:boss`).
    ///
    /// For vanilla entity types, prefer the generated `sand_core::generated::EntityType` enum.
    EntityTypeId
}

impl sand_commands::IntoTextEntityType for EntityTypeId {
    fn into_text_entity_type(self) -> String {
        self.to_string()
    }
}

impl sand_commands::IntoTextEntityType for &EntityTypeId {
    fn into_text_entity_type(self) -> String {
        self.to_string()
    }
}

impl sand_commands::selector::IntoEntityType for EntityTypeId {
    fn into_entity_type(self) -> String {
        self.to_string()
    }
}

impl sand_commands::selector::IntoEntityType for &EntityTypeId {
    fn into_entity_type(self) -> String {
        self.to_string()
    }
}

registry_id! {
    /// Typed Minecraft function identifier (e.g. `minecraft:load` or `mypack:tick`).
    FunctionId
}

registry_id! {
    /// Typed Minecraft enchantment identifier (e.g. `minecraft:sharpness` or `mymod:arcane`).
    EnchantmentId
}

registry_id! {
    /// Typed Minecraft biome identifier (e.g. `minecraft:plains` or `mymod:mystic_forest`).
    BiomeId
}

registry_id! {
    /// Typed Minecraft dimension identifier (e.g. `minecraft:overworld` or `mymod:pocket`).
    DimensionId
}

registry_id! {
    /// Typed Minecraft damage type identifier (e.g. `minecraft:generic` or `mymod:arcane`).
    DamageTypeId
}

registry_id! {
    /// Typed Minecraft structure identifier (e.g. `minecraft:village` or `mymod:dungeon`).
    StructureId
}

registry_id! {
    /// Resource-location-backed Minecraft status-effect identifier.
    ///
    /// This is the shared registry form used for dynamic, generated, and modded
    /// IDs. [`crate::EffectId`] remains available as the enum-style vanilla
    /// convenience and converts to and from this type.
    StatusEffectId
}

registry_id! {
    /// Resource-location-backed Minecraft potion identifier.
    ///
    /// The `PotionRegistryId` name deliberately avoids colliding with the
    /// existing enum-style [`crate::PotionId`] compatibility API.
    PotionRegistryId
}

// ── TagId<T> ─────────────────────────────────────────────────────────────────

/// A typed tag identifier scoped to a specific registry kind `T`.
///
/// The phantom `T` marker allows you to distinguish `TagId<ItemId>` from
/// `TagId<BlockId>` in API signatures, preventing accidental cross-registry
/// mixing.
///
/// Minecraft serializes tags as `#namespace:path` in some contexts (item
/// predicates) and `namespace:path` in others (data files).  Use
/// [`TagId::to_tag_string`] for the `#`-prefixed form and [`fmt::Display`] for
/// the plain form.
///
/// # Example
/// ```rust
/// use sand_components::registry::{TagId, ItemId};
///
/// let tag: TagId<ItemId> = TagId::minecraft("logs").unwrap();
/// assert_eq!(tag.to_string(), "minecraft:logs");
/// assert_eq!(tag.to_tag_string(), "#minecraft:logs");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagId<T> {
    rl: ResourceLocation,
    _marker: PhantomData<T>,
}

impl<T> TagId<T> {
    /// Construct a `minecraft:<path>` tag.  Returns an error if `path` is invalid.
    pub fn minecraft(path: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            rl: ResourceLocation::minecraft(path)?,
            _marker: PhantomData,
        })
    }

    /// Wrap any [`ResourceLocation`] as a tag ID.
    pub fn custom(rl: ResourceLocation) -> Self {
        Self {
            rl,
            _marker: PhantomData,
        }
    }

    /// Returns the `#namespace:path` form used in item predicates and ingredients.
    pub fn to_tag_string(&self) -> String {
        format!("#{}", self.rl)
    }

    /// Access the inner [`ResourceLocation`].
    pub fn as_resource_location(&self) -> &ResourceLocation {
        &self.rl
    }
}

impl<T> fmt::Display for TagId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.rl.fmt(f)
    }
}

impl<T> Serialize for TagId<T> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        self.rl.serialize(s)
    }
}

impl<T> std::str::FromStr for TagId<T> {
    type Err = crate::error::SandError;
    fn from_str(s: &str) -> Result<Self> {
        let stripped = s.strip_prefix('#').unwrap_or(s);
        Ok(Self {
            rl: stripped.parse()?,
            _marker: PhantomData,
        })
    }
}

impl<T> From<ResourceLocation> for TagId<T> {
    fn from(rl: ResourceLocation) -> Self {
        Self {
            rl,
            _marker: PhantomData,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_id_minecraft() {
        let id = ItemId::minecraft("diamond_sword").unwrap();
        assert_eq!(id.to_string(), "minecraft:diamond_sword");
    }

    #[test]
    fn item_id_custom() {
        let rl = ResourceLocation::new("mymod", "arcane_blade").unwrap();
        let id = ItemId::custom(rl);
        assert_eq!(id.to_string(), "mymod:arcane_blade");
    }

    #[test]
    fn item_id_from_resource_location() {
        let rl = ResourceLocation::new("mymod", "sword").unwrap();
        let id: ItemId = rl.into();
        assert_eq!(id.to_string(), "mymod:sword");
    }

    #[test]
    fn status_effect_and_potion_registry_ids_validate_and_serialize() {
        let effect = StatusEffectId::minecraft("speed").unwrap();
        let potion: PotionRegistryId = "mymod:arcane_brew".parse().unwrap();
        assert_eq!(effect.to_string(), "minecraft:speed");
        assert_eq!(potion.to_string(), "mymod:arcane_brew");
        assert_eq!(
            serde_json::to_value(effect).unwrap(),
            serde_json::json!("minecraft:speed")
        );
        assert!("not namespaced".parse::<StatusEffectId>().is_err());
        assert!("minecraft:bad path".parse::<PotionRegistryId>().is_err());
    }

    #[test]
    fn item_id_parse() {
        let id: ItemId = "minecraft:golden_apple".parse().unwrap();
        assert_eq!(id.to_string(), "minecraft:golden_apple");
    }

    #[test]
    fn item_id_invalid_namespace_rejected() {
        assert!(ItemId::minecraft("Invalid Path").is_err());
    }

    #[test]
    fn tag_id_minecraft() {
        let tag: TagId<ItemId> = TagId::minecraft("logs").unwrap();
        assert_eq!(tag.to_string(), "minecraft:logs");
        assert_eq!(tag.to_tag_string(), "#minecraft:logs");
    }

    #[test]
    fn tag_id_parse_with_hash() {
        let tag: TagId<ItemId> = "#minecraft:logs".parse().unwrap();
        assert_eq!(tag.to_string(), "minecraft:logs");
    }

    #[test]
    fn tag_id_parse_without_hash() {
        let tag: TagId<BlockId> = "minecraft:planks".parse().unwrap();
        assert_eq!(tag.to_tag_string(), "#minecraft:planks");
    }

    #[test]
    fn tag_id_custom() {
        let rl = ResourceLocation::new("mymod", "special_blocks").unwrap();
        let tag: TagId<BlockId> = TagId::custom(rl);
        assert_eq!(tag.to_string(), "mymod:special_blocks");
    }

    #[test]
    fn dimension_id_minecraft() {
        let id = DimensionId::minecraft("overworld").unwrap();
        assert_eq!(id.to_string(), "minecraft:overworld");
    }

    #[test]
    fn entity_type_id_builds_typed_text_hover() {
        let text = sand_commands::Text::new("Inspect").hover_entity(
            EntityTypeId::minecraft("zombie").unwrap(),
            sand_commands::Text::new("Undead"),
        );
        let value: serde_json::Value = serde_json::from_str(&text.to_string()).unwrap();
        assert_eq!(value["hoverEvent"]["type"], "minecraft:zombie");
        assert_eq!(value["hoverEvent"]["name"]["text"], "Undead");
    }
}
