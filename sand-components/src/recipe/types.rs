//! Shared types used across all recipe variants.

use std::fmt::Display;

use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};

// ── Ingredient ───────────────────────────────────────────────────────────────

/// Represents a recipe ingredient that can be specified by item ID or item tag.
pub struct Ingredient {
    pub item: Option<String>,
    pub tag: Option<String>,
}

impl Ingredient {
    /// Creates an ingredient specified by a single item ID.
    pub fn item(id: impl Display) -> Self {
        Self {
            item: Some(id.to_string()),
            tag: None,
        }
    }

    /// Creates an ingredient specified by an item tag (matches all items in the tag).
    pub fn tag(id: impl Display) -> Self {
        Self {
            item: None,
            tag: Some(id.to_string()),
        }
    }
}

impl Serialize for Ingredient {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let count = self.item.is_some() as usize + self.tag.is_some() as usize;
        let mut map = serializer.serialize_map(Some(count))?;
        if let Some(ref item) = self.item {
            map.serialize_entry("item", item)?;
        }
        if let Some(ref tag) = self.tag {
            map.serialize_entry("tag", tag)?;
        }
        map.end()
    }
}

// ── RecipeResult ─────────────────────────────────────────────────────────────

/// Represents the output of a recipe, including the item ID and quantity produced.
pub struct RecipeResult {
    pub id: String,
    pub count: u32,
}

impl RecipeResult {
    /// Creates a new recipe result with the given item ID and quantity.
    pub fn new(id: impl Display, count: u32) -> Self {
        Self {
            id: id.to_string(),
            count,
        }
    }
}

impl Serialize for RecipeResult {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("count", &self.count)?;
        map.end()
    }
}

// ── CookingType ──────────────────────────────────────────────────────────────

/// Specifies the type of cooking recipe (smelting, blasting, smoking, or campfire cooking).
pub enum CookingType {
    Smelting,
    Blasting,
    Smoking,
    CampfireCooking,
}

impl CookingType {
    /// Returns the Minecraft recipe type identifier string.
    pub fn type_str(&self) -> &'static str {
        match self {
            CookingType::Smelting => "minecraft:smelting",
            CookingType::Blasting => "minecraft:blasting",
            CookingType::Smoking => "minecraft:smoking",
            CookingType::CampfireCooking => "minecraft:campfire_cooking",
        }
    }
}
