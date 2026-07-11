//! Shared types used across all recipe variants.

use std::fmt::Display;

use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, Serializer};

// ── Ingredient ───────────────────────────────────────────────────────────────

/// Represents a recipe ingredient that can be specified by item ID or item tag.
pub struct Ingredient {
    pub item: Option<String>,
    pub tag: Option<String>,
    alternatives: Vec<Ingredient>,
}

impl Ingredient {
    /// Creates an ingredient specified by a single item ID.
    pub fn item(id: impl Display) -> Self {
        Self {
            item: Some(id.to_string()),
            tag: None,
            alternatives: Vec::new(),
        }
    }

    /// Creates an ingredient specified by an item tag (matches all items in the tag).
    pub fn tag(id: impl Display) -> Self {
        Self {
            item: None,
            tag: Some(id.to_string()),
            alternatives: Vec::new(),
        }
    }

    /// Creates an ingredient that matches any of the supplied alternatives.
    /// Modern recipe JSON represents alternatives as an array of ingredient
    /// values, where item IDs and tag IDs are both strings.
    pub fn alternatives(alternatives: impl IntoIterator<Item = Ingredient>) -> Self {
        Self {
            item: None,
            tag: None,
            alternatives: alternatives.into_iter().collect(),
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            item: None,
            tag: None,
            alternatives: Vec::new(),
        }
    }

    /// Returns `true` if this ingredient has no item, tag, or alternatives
    /// (an invalid state that would fail serialization).
    pub fn is_empty(&self) -> bool {
        self.item.is_none()
            && self.tag.is_none()
            && (self.alternatives.is_empty() || self.alternatives.iter().all(|a| a.is_empty()))
    }
}

impl Serialize for Ingredient {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !self.alternatives.is_empty() {
            let mut seq = serializer.serialize_seq(Some(self.alternatives.len()))?;
            for ingredient in &self.alternatives {
                seq.serialize_element(ingredient)?;
            }
            return seq.end();
        }
        if let Some(ref item) = self.item {
            return serializer.serialize_str(item);
        }
        if let Some(ref tag) = self.tag {
            return serializer.serialize_str(&format!("#{tag}"));
        }
        Err(serde::ser::Error::custom(
            "recipe ingredient cannot be empty",
        ))
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Ingredient, RecipeResult};

    #[test]
    fn serializes_modern_ingredient_forms() {
        assert_eq!(
            serde_json::to_value(Ingredient::item("minecraft:oak_planks")).unwrap(),
            json!("minecraft:oak_planks")
        );
        assert_eq!(
            serde_json::to_value(Ingredient::tag("minecraft:planks")).unwrap(),
            json!("#minecraft:planks")
        );
        assert_eq!(
            serde_json::to_value(Ingredient::alternatives([
                Ingredient::item("minecraft:oak_planks"),
                Ingredient::tag("minecraft:logs"),
            ]))
            .unwrap(),
            json!(["minecraft:oak_planks", "#minecraft:logs"])
        );
    }

    #[test]
    fn serializes_modern_recipe_result() {
        assert_eq!(
            serde_json::to_value(RecipeResult::new("powers:reinforced_shield", 1)).unwrap(),
            json!({ "id": "powers:reinforced_shield", "count": 1 })
        );
    }
}
