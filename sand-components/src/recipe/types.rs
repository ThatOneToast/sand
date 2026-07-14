//! Shared types used across all recipe variants.

use std::fmt::Display;

use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, Serializer};
use serde_json::Value;

use crate::error::{Result as SandResult, SandError};
use crate::item::CustomItem;
use crate::item::matcher::{ItemMatcher, ItemMatcherConsumer};
use crate::item::stack::ItemStack;
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
#[derive(Debug)]
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

    /// Attempt to build a recipe ingredient that matches a specific
    /// [`CustomItem`] exactly, including its data components (e.g. its
    /// [`custom_data`](CustomItem::custom_data) identity marker).
    ///
    /// # This always returns an error
    ///
    /// Minecraft's vanilla crafting recipe `Ingredient` schema — shaped,
    /// shapeless, cooking, stonecutting, and smithing recipes alike —
    /// matches only by item ID or item tag, in every Minecraft version Sand
    /// currently targets (the legacy `1.18`–`1.21.11` series and the `26.x`
    /// calendar series). There is no component predicate in the recipe
    /// ingredient schema; component predicates exist only in *predicate*/
    /// advancement JSON (see [`crate::predicates::ItemPredicate`]), which the
    /// crafting grid does not consult when deciding whether an item fills an
    /// ingredient slot.
    ///
    /// Silently degrading to an item-ID-only ingredient would let *any*
    /// item of the same base type (e.g. a plain `minecraft:white_wool`)
    /// satisfy a recipe meant to require this specific custom item — the
    /// exact identity loss this crate is designed to prevent. So this always
    /// fails with [`SandError::ComponentValidation`] describing the missing
    /// capability instead of emitting a misleading ingredient.
    ///
    /// If matching by base item type alone is acceptable, use
    /// [`Ingredient::item_id`] or [`Ingredient::item`] directly and enforce
    /// component identity elsewhere (e.g. a function that runs after
    /// crafting and checks `minecraft:custom_data` on the result).
    pub fn custom_item(item: &CustomItem) -> SandResult<Self> {
        Err(SandError::ComponentValidation {
            location: ResourceLocation::new("sand", "recipe_ingredient")
                .expect("fixed 'sand:recipe_ingredient' sentinel location is valid"),
            kind: "recipe_ingredient".to_string(),
            field: "ingredient".to_string(),
            message: format!(
                "component-aware recipe ingredients are not supported by any Minecraft \
                 version Sand currently targets — vanilla recipe ingredients match only \
                 by item ID or tag, never by data components, so `{}` cannot be used as \
                 an exact-match ingredient without silently degrading to matching its \
                 base item (`{}`) alone. Use Ingredient::item_id or Ingredient::item to \
                 match by base item type explicitly, and verify component identity \
                 (e.g. minecraft:custom_data) elsewhere, such as a function that runs \
                 after crafting.",
                item.base_id(),
                item.base_id(),
            ),
        })
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

// ── ItemMatcher / ItemStack conversions (#229) ──────────────────────────────

/// Fallible conversion into a recipe [`Ingredient`].
///
/// Vanilla recipe ingredients match only by item ID or item tag in every
/// Minecraft version Sand targets — there is no component predicate in the
/// ingredient schema. A matcher that constrains anything beyond item ID
/// ([`ItemMatcher::has_component_constraints`]) always fails here rather than
/// silently degrading to base-item-only matching, matching
/// [`Ingredient::custom_item`]'s existing behavior.
pub trait TryIntoIngredient {
    fn try_into_ingredient(&self) -> SandResult<Ingredient>;
}

impl TryIntoIngredient for ItemMatcher {
    fn try_into_ingredient(&self) -> SandResult<Ingredient> {
        if self.has_component_constraints() {
            return Err(SandError::ComponentValidation {
                location: ResourceLocation::new("sand", "recipe_ingredient")
                    .expect("fixed 'sand:recipe_ingredient' sentinel location is valid"),
                kind: ItemMatcherConsumer::RecipeIngredient.label(),
                field: "ingredient".to_string(),
                message: "this matcher constrains item data components (custom data, \
                    enchantments, damage, or a raw component/predicate), but vanilla recipe \
                    ingredients match only by item ID or tag in every Minecraft version Sand \
                    targets. Converting this matcher would silently degrade it to matching \
                    any item of the same base type. Use an item-ID-only ItemMatcher for the \
                    ingredient, and verify component identity elsewhere (e.g. a function \
                    that runs after crafting)."
                    .to_string(),
            });
        }
        match self.items() {
            [] => Err(SandError::ComponentValidation {
                location: ResourceLocation::new("sand", "recipe_ingredient")
                    .expect("fixed 'sand:recipe_ingredient' sentinel location is valid"),
                kind: ItemMatcherConsumer::RecipeIngredient.label(),
                field: "ingredient".to_string(),
                message: "matcher has no item IDs to convert into an ingredient".to_string(),
            }),
            [single] => Ok(Ingredient::raw_item(single.clone())),
            multiple => Ok(Ingredient::alternatives(
                multiple.iter().cloned().map(Ingredient::raw_item),
            )),
        }
    }
}

/// Fallible conversion into a component-bearing recipe [`RecipeResult`].
///
/// Preserves every representable typed/raw data component the source
/// [`ItemStack`] carries — never reduces it to its base item ID alone. See
/// [`ItemStack::stack_components`] for what can fail (e.g. a raw component
/// whose value has no general SNBT-to-JSON conversion).
pub trait TryIntoRecipeResult {
    fn try_into_recipe_result(&self, count: u32) -> SandResult<RecipeResult>;
}

impl TryIntoRecipeResult for ItemStack {
    fn try_into_recipe_result(&self, count: u32) -> SandResult<RecipeResult> {
        let stack = self.stack_components()?;
        let (id, components) = stack.into_parts();
        Ok(RecipeResult {
            id,
            count,
            components,
        })
    }
}

// ── RecipeResult ─────────────────────────────────────────────────────────────

/// Represents the output of a recipe, including the item ID, quantity
/// produced, and (optionally) the data components a component-bearing
/// [`CustomItem`] result must carry — e.g. `minecraft:custom_data`,
/// `minecraft:item_name`, enchantment glint overrides, and so on.
///
/// Component-free results (built via [`RecipeResult::item`], [`RecipeResult::raw`],
/// or [`RecipeResult::new`]) serialize exactly as before:
/// `{"id": ..., "count": ...}` — no empty `"components"` object is ever emitted.
///
/// Component-bearing results (built via [`RecipeResult::custom_item`] or
/// [`RecipeResult::from_custom_item`]) additionally serialize a `"components"`
/// object built from the source [`CustomItem`]'s typed and raw component
/// state — never from `CustomItem`'s command item-stack `Display` string.
#[derive(Debug)]
pub struct RecipeResult {
    pub id: String,
    pub count: u32,
    components: Vec<(String, Value)>,
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
            components: Vec::new(),
        }
    }

    /// Legacy unchecked compatibility constructor. Prefer [`RecipeResult::item`]
    /// or make raw intent explicit with [`RecipeResult::raw`].
    #[doc(hidden)]
    pub fn new(id: impl Display, count: u32) -> Self {
        Self::raw(id.to_string(), count)
    }

    pub(crate) fn empty() -> Self {
        Self::raw(String::new(), 1)
    }

    /// Build a component-bearing recipe result from a [`CustomItem`], defaulting
    /// to a count of `1`. Use [`RecipeResult::from_custom_item`] to select a
    /// different (positive) count.
    ///
    /// Preserves the item's base ID and every representable typed/raw data
    /// component — it never reduces the item to its base ID alone. Fails with
    /// a descriptive [`SandError`] if a component cannot be safely represented
    /// as structured JSON (see [`CustomItem::stack_components`]).
    pub fn custom_item(item: &CustomItem) -> SandResult<Self> {
        Self::from_custom_item(item, 1)
    }

    /// Build a component-bearing recipe result from a [`CustomItem`] with an
    /// explicit result `count`.
    pub fn from_custom_item(item: &CustomItem, count: u32) -> SandResult<Self> {
        let stack = item.stack_components()?;
        let (id, components) = stack.into_parts();
        Ok(Self {
            id,
            count,
            components,
        })
    }

    /// `true` if this result carries one or more data components.
    pub fn has_components(&self) -> bool {
        !self.components.is_empty()
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

impl TryFrom<CustomItem> for RecipeResult {
    type Error = SandError;

    /// Converts with a default count of `1`. Use [`RecipeResult::from_custom_item`]
    /// to select a different count.
    fn try_from(item: CustomItem) -> SandResult<Self> {
        RecipeResult::custom_item(&item)
    }
}

impl TryFrom<&CustomItem> for RecipeResult {
    type Error = SandError;

    /// Converts with a default count of `1`. Use [`RecipeResult::from_custom_item`]
    /// to select a different count.
    fn try_from(item: &CustomItem) -> SandResult<Self> {
        RecipeResult::custom_item(item)
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
        let has_components = !self.components.is_empty();
        let mut map = serializer.serialize_map(Some(if has_components { 3 } else { 2 }))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("count", &self.count)?;
        if has_components {
            let components: serde_json::Map<String, Value> =
                self.components.iter().cloned().collect();
            map.serialize_entry("components", &components)?;
        }
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

    // ── Component-bearing RecipeResult (#226) ────────────────────────────────

    use crate::item::CustomItem;

    fn elevator() -> CustomItem {
        CustomItem::new("minecraft:white_wool")
            .custom_data("elevator_block_item")
            .component(crate::item::ItemComponent::EnchantmentGlintOverride(true))
            .item_name(
                sand_commands::TextComponent::literal("Elevator Block")
                    .bold(true)
                    .color(sand_commands::ChatColor::Aqua),
            )
    }

    #[test]
    fn component_free_result_omits_components_key() {
        let result = RecipeResult::item(ItemId::minecraft("diamond").unwrap(), 1);
        assert!(!result.has_components());
        let value = serde_json::to_value(&result).unwrap();
        assert_eq!(value, json!({ "id": "minecraft:diamond", "count": 1 }));
        assert!(value.get("components").is_none());
    }

    #[test]
    fn custom_item_result_defaults_to_count_one() {
        let result = RecipeResult::custom_item(&elevator()).unwrap();
        assert_eq!(result.id, "minecraft:white_wool");
        assert_eq!(result.count, 1);
        assert!(result.has_components());
    }

    #[test]
    fn from_custom_item_selects_a_non_default_count() {
        let result = RecipeResult::from_custom_item(&elevator(), 5).unwrap();
        assert_eq!(result.count, 5);
    }

    #[test]
    fn custom_item_result_preserves_marker_glint_and_item_name() {
        let result = RecipeResult::custom_item(&elevator()).unwrap();
        let value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["id"], "minecraft:white_wool");
        assert_eq!(value["count"], 1);
        assert_eq!(
            value["components"]["minecraft:custom_data"],
            json!({ "elevator_block_item": true })
        );
        assert_eq!(
            value["components"]["minecraft:enchantment_glint_override"],
            json!(true)
        );
        assert_eq!(
            value["components"]["minecraft:item_name"]["text"],
            "Elevator Block"
        );
        assert_eq!(value["components"]["minecraft:item_name"]["bold"], true);
    }

    #[test]
    fn try_from_custom_item_matches_custom_item_constructor() {
        let a = RecipeResult::custom_item(&elevator()).unwrap();
        let b: RecipeResult = (&elevator()).try_into().unwrap();
        let c: RecipeResult = elevator().try_into().unwrap();
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&b).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&c).unwrap()
        );
    }

    #[test]
    fn custom_item_result_never_silently_drops_raw_snbt_only_components() {
        let item = CustomItem::new("minecraft:bow").with_raw_component(
            crate::raw::RawComponent::new("bundle_contents", "{items:[]}"),
        );
        let err = RecipeResult::custom_item(&item)
            .expect_err("SNBT-only raw component must fail, not silently vanish");
        assert!(err.to_string().contains("bundle_contents"));
    }

    #[test]
    fn custom_item_result_accepts_raw_component_that_is_valid_json() {
        let item = CustomItem::new("minecraft:bow")
            .with_raw_component(crate::raw::RawComponent::new("modded:widget", "{\"a\":1}"));
        let result = RecipeResult::custom_item(&item).unwrap();
        let value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["components"]["modded:widget"], json!({ "a": 1 }));
    }

    #[test]
    fn zero_count_still_fails_validation_for_custom_item_results() {
        use crate::resource_location::ResourceLocation;
        let result = RecipeResult::from_custom_item(&elevator(), 0).unwrap();
        let loc = ResourceLocation::new("test", "zero").unwrap();
        let err = result
            .validate_at(&loc, "result")
            .expect_err("zero count must fail even for component-bearing results");
        assert!(err.to_string().contains("result.count"));
    }

    // ── Component-aware ingredient capability (#226) ─────────────────────────

    #[test]
    fn ingredient_custom_item_always_returns_a_capability_error() {
        let err = Ingredient::custom_item(&elevator())
            .expect_err("Minecraft recipe ingredients cannot match by component");
        let msg = err.to_string();
        assert!(msg.contains("minecraft:white_wool"));
        assert!(msg.contains("component"));
    }

    #[test]
    fn ingredient_base_item_matching_remains_available_and_unchanged() {
        // Explicit base-item-only matching still works and is unaffected by
        // the fact that component-aware matching is unsupported.
        assert_eq!(
            serde_json::to_value(Ingredient::item("minecraft:white_wool")).unwrap(),
            json!("minecraft:white_wool")
        );
    }

    // ── ItemMatcher / ItemStack conversions (#229) ───────────────────────────

    use super::{TryIntoIngredient, TryIntoRecipeResult};
    use crate::item::matcher::ItemMatcher;
    use crate::item::stack::ItemStack;

    #[test]
    fn item_id_only_matcher_converts_to_ingredient() {
        let matcher = ItemMatcher::item(ItemId::minecraft("oak_planks").unwrap());
        let ingredient = matcher.try_into_ingredient().unwrap();
        assert_eq!(
            serde_json::to_value(ingredient).unwrap(),
            json!("minecraft:oak_planks")
        );
    }

    #[test]
    fn multi_item_matcher_converts_to_alternatives() {
        let matcher = ItemMatcher::any_of([
            ItemId::minecraft("oak_planks").unwrap(),
            ItemId::minecraft("spruce_planks").unwrap(),
        ]);
        let ingredient = matcher.try_into_ingredient().unwrap();
        assert_eq!(
            serde_json::to_value(ingredient).unwrap(),
            json!(["minecraft:oak_planks", "minecraft:spruce_planks"])
        );
    }

    #[test]
    fn component_constrained_matcher_never_silently_becomes_base_item_ingredient() {
        let matcher = ItemMatcher::item(ItemId::minecraft("white_wool").unwrap())
            .custom_data_partial("elevator_block_item");
        let err = matcher
            .try_into_ingredient()
            .expect_err("component-constrained matcher must not convert to an ingredient");
        let msg = err.to_string();
        assert!(msg.contains("recipe ingredient"));
        assert!(msg.contains("component"));
    }

    #[test]
    fn item_stack_converts_to_component_bearing_recipe_result() {
        use crate::item::{CustomData, ItemComponent};

        let stack = ItemStack::new(ItemId::minecraft("white_wool").unwrap()).component(
            ItemComponent::CustomData(CustomData::marker("elevator_block_item")),
        );
        let result = stack.try_into_recipe_result(1).unwrap();
        let value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["id"], "minecraft:white_wool");
        assert_eq!(
            value["components"]["minecraft:custom_data"],
            json!({ "elevator_block_item": true })
        );
    }

    #[test]
    fn item_stack_recipe_result_matches_equivalent_custom_item_result() {
        use crate::item::{CustomData, ItemComponent};

        let stack = ItemStack::new(ItemId::minecraft("white_wool").unwrap()).component(
            ItemComponent::CustomData(CustomData::marker("elevator_block_item")),
        );
        let via_stack = stack.try_into_recipe_result(1).unwrap();
        let via_custom_item = RecipeResult::custom_item(&elevator_base_only()).unwrap();
        assert_eq!(
            serde_json::to_value(&via_stack).unwrap(),
            serde_json::to_value(&via_custom_item).unwrap()
        );
    }

    fn elevator_base_only() -> CustomItem {
        CustomItem::new("minecraft:white_wool").custom_data("elevator_block_item")
    }
}
