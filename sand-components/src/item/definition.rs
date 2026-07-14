//! [`CustomItemDefinition`] — a reusable source of truth for one custom
//! item's identity, shared across every representation that identity needs
//! to appear in.

use crate::error::Result as SandResult;
use crate::item::matcher::{ItemMatcher, ItemMatcherConsumer};
use crate::item::stack::ItemStack;
use crate::item::{CustomData, CustomItem, ItemComponent};
use crate::predicates::ItemPredicate;
use crate::registry::ItemId;

/// A reusable custom item definition that owns one item's identity: its base
/// Minecraft item ID, its `custom_data` marker, and any always-present
/// components.
///
/// Every representation produced from a `CustomItemDefinition` — a give-able
/// [`ItemStack`], a detection [`ItemMatcher`], or an adapter into an existing
/// API (predicate/recipe result) — shares the same base item ID and marker,
/// so a definition's identity never has to be repeated (and cannot drift)
/// across the surfaces that consume it.
///
/// # Example
/// ```rust
/// use sand_components::item::definition::CustomItemDefinition;
/// use sand_components::item::ItemComponent;
/// use sand_components::registry::ItemId;
///
/// let special_bow = CustomItemDefinition::new(ItemId::minecraft("bow").unwrap())
///     .marker("special_bow");
///
/// let stack = special_bow.stack(1);
/// let matcher = special_bow.matcher();
///
/// assert_eq!(stack.id().to_string(), "minecraft:bow");
/// assert_eq!(matcher.items(), &["minecraft:bow".to_string()]);
/// ```
#[derive(Debug, Clone)]
pub struct CustomItemDefinition {
    base_item: ItemId,
    marker: Option<String>,
    components: Vec<ItemComponent>,
}

impl CustomItemDefinition {
    /// Create a definition for `base_item` with no marker and no components.
    pub fn new(base_item: ItemId) -> Self {
        Self {
            base_item,
            marker: None,
            components: Vec::new(),
        }
    }

    /// Set this definition's `custom_data` marker key (e.g. `"special_bow"`).
    ///
    /// Every [`ItemStack`]/[`ItemMatcher`] this definition produces carries
    /// this same marker — [`stack`](Self::stack) sets it as an exact
    /// `custom_data` component, and [`matcher`](Self::matcher) matches it
    /// with **partial** semantics (see [`ItemMatcher::custom_data_partial`]),
    /// the correct way to detect "is this one of my custom items" without
    /// rejecting items other packs have added unrelated custom-data keys to.
    pub fn marker(mut self, key: impl Into<String>) -> Self {
        self.marker = Some(key.into());
        self
    }

    /// Add an always-present typed component to this definition. Present on
    /// every [`stack`](Self::stack) this definition produces.
    pub fn component(mut self, component: ItemComponent) -> Self {
        self.components.push(component);
        self
    }

    /// The base Minecraft item ID this definition is built on.
    pub fn base_item(&self) -> &ItemId {
        &self.base_item
    }

    /// The `custom_data` marker key, if set.
    pub fn marker_key(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    /// Build a concrete [`ItemStack`] of `count` for this definition, with
    /// this definition's marker and always-present components applied.
    pub fn stack(&self, count: u32) -> ItemStack {
        let mut stack = ItemStack::new(self.base_item.clone()).count(count);
        if let Some(ref key) = self.marker {
            stack = stack.component(ItemComponent::CustomData(CustomData::marker(key.clone())));
        }
        for component in &self.components {
            stack = stack.component(component.clone());
        }
        stack
    }

    /// Build a detection [`ItemMatcher`] for this definition: matches the
    /// base item ID and, if a marker was set, partially matches its
    /// `custom_data` key (ignoring any other custom-data keys the item may
    /// carry — see [`ItemMatcher::custom_data_partial`]).
    pub fn matcher(&self) -> ItemMatcher {
        let mut matcher = ItemMatcher::item(self.base_item.clone());
        if let Some(ref key) = self.marker {
            matcher = matcher.custom_data_partial(key.clone());
        }
        matcher
    }

    /// Adapter into the existing [`predicates::ItemPredicate`](crate::predicates::ItemPredicate)
    /// API for a specific consumer and version profile. Equivalent to
    /// `self.matcher().try_render_for(consumer, caps)`.
    pub fn try_item_predicate(
        &self,
        consumer: ItemMatcherConsumer,
        caps: Option<&sand_version::VersionCaps>,
    ) -> SandResult<ItemPredicate> {
        self.matcher().try_render_for(consumer, caps)
    }

    /// Adapter into the existing [`recipe::RecipeResult`](crate::recipe::RecipeResult)
    /// API — a component-bearing recipe result of `count`, preserving this
    /// definition's marker and components in full (see
    /// [`crate::recipe::RecipeResult::from_custom_item`]).
    pub fn try_recipe_result(&self, count: u32) -> SandResult<crate::recipe::RecipeResult> {
        crate::recipe::RecipeResult::from_custom_item(
            &self.stack(count).custom_item().clone(),
            count,
        )
    }

    /// This definition's stack rendered as a [`CustomItem`], for interop with
    /// APIs that still take the older `CustomItem` type directly (e.g.
    /// [`CustomItem::item_predicate`], `sand_core::custom_item_ext`).
    pub fn as_custom_item(&self) -> CustomItem {
        self.stack(1).custom_item().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::matcher::ItemMatcherConsumer;
    use sand_commands::TextComponent;

    fn definition() -> CustomItemDefinition {
        CustomItemDefinition::new(ItemId::minecraft("bow").unwrap())
            .marker("special_bow")
            .component(ItemComponent::item_name(TextComponent::literal(
                "Special Bow",
            )))
    }

    #[test]
    fn stack_and_matcher_share_base_item_and_marker() {
        let def = definition();
        let stack = def.stack(1);
        let matcher = def.matcher();

        assert_eq!(stack.id().to_string(), "minecraft:bow");
        assert_eq!(matcher.items(), &["minecraft:bow".to_string()]);

        let stack_json = stack.stack_components().unwrap();
        assert_eq!(
            stack_json
                .components()
                .iter()
                .find(|(k, _)| k == "minecraft:custom_data")
                .unwrap()
                .1,
            serde_json::json!({ "special_bow": true })
        );
    }

    #[test]
    fn matcher_uses_partial_custom_data_not_exact() {
        let def = definition();
        let pred = def
            .matcher()
            .try_render_for(ItemMatcherConsumer::Predicate, None)
            .unwrap();
        let json = serde_json::to_value(&pred).unwrap();
        assert_eq!(
            json["predicates"]["minecraft:custom_data"],
            "{special_bow:1b}"
        );
        assert!(json.get("components").is_none());
    }

    #[test]
    fn try_item_predicate_matches_direct_matcher_conversion() {
        let def = definition();
        let via_definition = def
            .try_item_predicate(ItemMatcherConsumer::Predicate, None)
            .unwrap();
        let via_matcher = def
            .matcher()
            .try_render_for(ItemMatcherConsumer::Predicate, None)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&via_definition).unwrap(),
            serde_json::to_value(&via_matcher).unwrap()
        );
    }

    #[test]
    fn try_recipe_result_preserves_marker_and_components() {
        let def = definition();
        let result = def.try_recipe_result(1).unwrap();
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["id"], "minecraft:bow");
        assert_eq!(
            json["components"]["minecraft:custom_data"],
            serde_json::json!({ "special_bow": true })
        );
        assert_eq!(
            json["components"]["minecraft:item_name"]["text"],
            "Special Bow"
        );
    }

    #[test]
    fn no_marker_definition_matcher_has_no_component_constraints() {
        let def = CustomItemDefinition::new(ItemId::minecraft("stick").unwrap());
        assert!(!def.matcher().has_component_constraints());
    }
}
