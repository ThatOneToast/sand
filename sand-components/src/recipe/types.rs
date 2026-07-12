//! Shared types used across all recipe variants.

use std::fmt::Display;

use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, Serializer};

use crate::error::{Result as SandResult, SandError};
use crate::registry::{ItemId, TagId};
use crate::resource_location::ResourceLocation;

/// Converts a validated item identifier into the representation used by recipes.
///
/// Implemented for [`ItemId`] and [`ResourceLocation`]. `sand-core` also
/// implements it for its generated vanilla `Item` enum without introducing a
/// dependency from `sand-components` back to `sand-core`.
pub trait IntoRecipeItemId {
    fn into_recipe_item_id(self) -> ItemId;
}

impl IntoRecipeItemId for ItemId {
    fn into_recipe_item_id(self) -> ItemId {
        self
    }
}

impl IntoRecipeItemId for &ItemId {
    fn into_recipe_item_id(self) -> ItemId {
        self.clone()
    }
}

impl IntoRecipeItemId for ResourceLocation {
    fn into_recipe_item_id(self) -> ItemId {
        self.into()
    }
}

impl IntoRecipeItemId for &ResourceLocation {
    fn into_recipe_item_id(self) -> ItemId {
        self.clone().into()
    }
}

// ── Ingredient ───────────────────────────────────────────────────────────────

/// Represents a recipe ingredient that can be specified by item ID or item tag.
pub struct Ingredient {
    pub item: Option<String>,
    pub tag: Option<String>,
    alternatives: Vec<Ingredient>,
}

impl Ingredient {
    /// Creates an item ingredient through Sand's validated item-ID boundary.
    pub fn item_id(id: impl IntoRecipeItemId) -> Self {
        Self::raw_item(id.into_recipe_item_id().to_string())
    }

    /// Creates an item-tag ingredient. The `ItemId` marker prevents block or
    /// other registry tags from being passed accidentally.
    pub fn item_tag(id: TagId<ItemId>) -> Self {
        Self::raw_tag(id.to_string())
    }

    /// Creates an item ingredient from an unchecked compatibility string.
    ///
    /// Prefer [`Ingredient::item_id`]. This escape hatch remains available for
    /// future or modded identifiers that cannot yet use Sand's typed registry.
    pub fn raw_item(id: impl Into<String>) -> Self {
        Self {
            item: Some(id.into()),
            tag: None,
            alternatives: Vec::new(),
        }
    }

    /// Creates an item-tag ingredient from an unchecked compatibility string.
    /// Prefer [`Ingredient::item_tag`].
    pub fn raw_tag(id: impl Into<String>) -> Self {
        Self {
            item: None,
            tag: Some(id.into()),
            alternatives: Vec::new(),
        }
    }

    /// Legacy unchecked compatibility constructor. Prefer [`Ingredient::item_id`]
    /// or make raw intent explicit with [`Ingredient::raw_item`].
    #[doc(hidden)]
    pub fn item(id: impl Display) -> Self {
        Self::raw_item(id.to_string())
    }

    /// Legacy unchecked compatibility constructor. Prefer [`Ingredient::item_tag`]
    /// or make raw intent explicit with [`Ingredient::raw_tag`].
    #[doc(hidden)]
    pub fn tag(id: impl Display) -> Self {
        Self::raw_tag(id.to_string())
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

    pub(crate) fn validate_at(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        let forms = usize::from(self.item.is_some())
            + usize::from(self.tag.is_some())
            + usize::from(!self.alternatives.is_empty());
        if forms > 1 {
            return Err(validation(
                location,
                field,
                "ingredient must use exactly one of item, tag, or alternatives",
            ));
        }
        if self.item.as_deref().is_some_and(str::is_empty) {
            return Err(validation(
                location,
                field,
                "ingredient item id must not be empty",
            ));
        }
        if self.tag.as_deref().is_some_and(str::is_empty) {
            return Err(validation(
                location,
                field,
                "ingredient tag id must not be empty",
            ));
        }
        if self.item.is_none() && self.tag.is_none() && self.alternatives.is_empty() {
            return Err(validation(location, field, "ingredient must not be empty"));
        }
        for (index, alternative) in self.alternatives.iter().enumerate() {
            alternative.validate_at(location, &format!("{field}.alternatives[{index}]"))?;
        }
        Ok(())
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
    /// Creates a recipe result through Sand's validated item-ID boundary.
    pub fn item(id: impl IntoRecipeItemId, count: u32) -> Self {
        Self::raw(id.into_recipe_item_id().to_string(), count)
    }

    /// Creates a recipe result from an unchecked compatibility string.
    pub fn raw(id: impl Into<String>, count: u32) -> Self {
        Self {
            id: id.into(),
            count,
        }
    }

    /// Legacy unchecked compatibility constructor. Prefer [`RecipeResult::item`]
    /// or make raw intent explicit with [`RecipeResult::raw`].
    #[doc(hidden)]
    pub fn new(id: impl Display, count: u32) -> Self {
        Self::raw(id.to_string(), count)
    }

    pub(crate) fn validate_at(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        if self.id.is_empty() {
            return Err(validation(
                location,
                &format!("{field}.id"),
                "recipe result item id must not be empty",
            ));
        }
        if self.count == 0 {
            return Err(validation(
                location,
                &format!("{field}.count"),
                "recipe result count must be at least 1",
            ));
        }
        Ok(())
    }
}

fn validation(location: &ResourceLocation, field: &str, message: &str) -> SandError {
    SandError::ComponentValidation {
        location: location.clone(),
        kind: "recipe".to_string(),
        field: field.to_string(),
        message: message.to_string(),
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
    use crate::registry::{ItemId, TagId};

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

    #[test]
    fn typed_item_tag_and_result_match_legacy_json() {
        let item = ItemId::minecraft("oak_planks").unwrap();
        assert_eq!(
            serde_json::to_value(Ingredient::item_id(item)).unwrap(),
            json!("minecraft:oak_planks")
        );

        let tag: TagId<ItemId> = TagId::minecraft("planks").unwrap();
        assert_eq!(
            serde_json::to_value(Ingredient::item_tag(tag)).unwrap(),
            json!("#minecraft:planks")
        );

        let result = RecipeResult::item(ItemId::minecraft("diamond").unwrap(), 1);
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            json!({ "id": "minecraft:diamond", "count": 1 })
        );
    }

    #[test]
    fn explicit_raw_compatibility_paths_preserve_json() {
        assert_eq!(
            serde_json::to_value(Ingredient::raw_item("future:item")).unwrap(),
            json!("future:item")
        );
        assert_eq!(
            serde_json::to_value(Ingredient::raw_tag("future:tag")).unwrap(),
            json!("#future:tag")
        );
        assert_eq!(
            serde_json::to_value(RecipeResult::raw("future:result", 2)).unwrap(),
            json!({ "id": "future:result", "count": 2 })
        );
    }
}
