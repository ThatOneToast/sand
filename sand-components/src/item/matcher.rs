//! [`ItemMatcher`] — detection semantics for items, distinct from
//! [`crate::item::stack::ItemStack`].
//!
//! A matcher expresses a partial or exact *condition* used to recognize an
//! item; it never represents a concrete item that exists. Converting a
//! matcher to a specific consumer (an advancement trigger field, a general
//! predicate, a recipe ingredient, a loot condition, ...) is fallible and
//! version-profile aware — see [`ItemMatcher::try_render_for`] and
//! [`ItemMatcherConsumer`].

use serde_json::Value;

use crate::advancement::AdvancementItemConsumer;
use crate::error::{Result as SandResult, SandError};
use crate::item::CustomData;
use crate::predicates::{IntRange, ItemPredicate};
use crate::raw::RawJson;
use crate::registry::{EnchantmentId, ItemId};
use crate::resource_location::ResourceLocation;

/// A single enchantment constraint within an [`ItemMatcher`].
#[derive(Debug, Clone)]
struct EnchantmentMatch {
    id: EnchantmentId,
    levels: Option<IntRange>,
}

/// Detection semantics for an item.
///
/// Distinguishes **exact** component/custom-data equality from **partial**
/// matching so intent is never overloaded onto one method whose meaning
/// silently varies by consumer:
///
/// - [`custom_data_exact`](Self::custom_data_exact) — the item's
///   `minecraft:custom_data` must equal `data` exactly (no extra keys).
/// - [`custom_data_partial`](Self::custom_data_partial) — the item's
///   `minecraft:custom_data` must contain the given key (any other keys are
///   ignored). This is the correct semantics for "is this a Sand custom item
///   of kind X" and is what [`crate::item::CustomItem::custom_data`]-tagged
///   items should be matched with.
///
/// A matcher retains this intent until it is converted for a specific target
/// via [`ItemMatcher::try_render_for`] — see that method's docs for the
/// conversion contract.
///
/// # Example
/// ```rust
/// use sand_components::item::matcher::ItemMatcher;
/// use sand_components::predicates::IntRange;
/// use sand_components::registry::ItemId;
///
/// let matcher = ItemMatcher::item(ItemId::minecraft("bow").unwrap())
///     .custom_data_partial("special_bow")
///     .damage_range(IntRange::at_most(50));
/// assert!(matcher.has_component_constraints());
/// ```
#[derive(Debug, Clone, Default)]
pub struct ItemMatcher {
    items: Vec<String>,
    count: Option<IntRange>,
    exact_custom_data: Option<CustomData>,
    partial_custom_data_keys: Vec<String>,
    enchantments: Vec<EnchantmentMatch>,
    damage_range: Option<IntRange>,
    exact_raw_components: Option<RawJson>,
    partial_raw_predicates: Option<RawJson>,
}

impl ItemMatcher {
    /// Match a specific item ID, with no further constraints.
    pub fn item(id: ItemId) -> Self {
        Self {
            items: vec![id.to_string()],
            ..Default::default()
        }
    }

    /// Match any of the supplied item IDs.
    pub fn any_of(ids: impl IntoIterator<Item = ItemId>) -> Self {
        Self {
            items: ids.into_iter().map(|id| id.to_string()).collect(),
            ..Default::default()
        }
    }

    /// Require the item stack's count to fall within `range`.
    pub fn count(mut self, range: IntRange) -> Self {
        self.count = Some(range);
        self
    }

    /// Require **exact** equality of the item's `minecraft:custom_data`
    /// component against `data`. Unlike [`custom_data_partial`](Self::custom_data_partial),
    /// any additional unrelated custom-data key on the item causes this to
    /// NOT match.
    pub fn custom_data_exact(mut self, data: CustomData) -> Self {
        self.exact_custom_data = Some(data);
        self
    }

    /// Require a named key in the item's `minecraft:custom_data` component to
    /// be present and truthy, ignoring any other keys the item carries. This
    /// is the correct semantics for detecting a
    /// [`CustomItem::custom_data`](crate::item::CustomItem::custom_data)-tagged
    /// item. Calling this multiple times ANDs the keys together.
    pub fn custom_data_partial(mut self, key: impl Into<String>) -> Self {
        self.partial_custom_data_keys.push(key.into());
        self
    }

    /// Require the item to carry the given enchantment, at any level.
    pub fn enchantment(mut self, id: EnchantmentId) -> Self {
        self.enchantments
            .push(EnchantmentMatch { id, levels: None });
        self
    }

    /// Require the item to carry the given enchantment within `levels`.
    pub fn enchantment_levels(mut self, id: EnchantmentId, levels: IntRange) -> Self {
        self.enchantments.push(EnchantmentMatch {
            id,
            levels: Some(levels),
        });
        self
    }

    /// Require the item's current `minecraft:damage` value to fall within `range`.
    pub fn damage_range(mut self, range: IntRange) -> Self {
        self.damage_range = Some(range);
        self
    }

    /// Raw escape hatch — require **exact** equality against arbitrary
    /// component JSON, merged into the same `components` bag
    /// [`custom_data_exact`](Self::custom_data_exact) uses.
    pub fn raw_components_exact(mut self, value: RawJson) -> Self {
        self.exact_raw_components = Some(value);
        self
    }

    /// Raw escape hatch — require **partial** match against arbitrary
    /// sub-predicate JSON, merged into the same `predicates` bag
    /// [`custom_data_partial`](Self::custom_data_partial) uses.
    pub fn raw_predicates_partial(mut self, value: RawJson) -> Self {
        self.partial_raw_predicates = Some(value);
        self
    }

    /// The item IDs this matcher accepts.
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// `true` if this matcher constrains anything beyond item ID/count — i.e.
    /// it inspects data components in some form (exact or partial).
    pub fn has_component_constraints(&self) -> bool {
        self.exact_custom_data.is_some()
            || !self.partial_custom_data_keys.is_empty()
            || !self.enchantments.is_empty()
            || self.damage_range.is_some()
            || self.exact_raw_components.is_some()
            || self.partial_raw_predicates.is_some()
    }

    /// Render this matcher as a [`predicates::ItemPredicate`](crate::predicates::ItemPredicate)
    /// for a specific consuming surface and Minecraft version profile.
    ///
    /// This is the single conversion seam every consumer-facing matcher
    /// conversion goes through — advancement item filters
    /// ([`try_into_advancement_predicate`](Self::try_into_advancement_predicate)),
    /// general predicates, loot conditions, and inventory/execute conditions
    /// all resolve here rather than each inspecting `ItemMatcher`'s internal
    /// state independently.
    ///
    /// # Errors
    ///
    /// Fails with [`SandError::ComponentValidation`] rather than weakening
    /// the matcher when:
    /// - this matcher has any component constraint (custom data, enchantment,
    ///   damage range, or raw component/predicate) and `caps` targets a
    ///   pre-1.20.5 profile that doesn't support item components;
    /// - [`custom_data_exact`](Self::custom_data_exact) was set to a
    ///   [`CustomData::Raw`] payload, which has no general SNBT-to-JSON
    ///   conversion.
    ///
    /// `caps == None` is treated the same as a fully item-component-capable
    /// modern profile, matching the `VersionCaps::all_enabled()` convention
    /// used elsewhere in Sand (e.g. [`crate::advancement::AdvancementSchemaFamily::for_caps`]).
    pub fn try_render_for(
        &self,
        consumer: ItemMatcherConsumer,
        caps: Option<&sand_version::VersionCaps>,
    ) -> SandResult<ItemPredicate> {
        if self.has_component_constraints()
            && !caps.is_none_or(|c| c.supports(sand_version::ComponentFeature::ItemComponents))
        {
            return Err(unsupported_legacy_item_filter(consumer));
        }
        if self
            .exact_raw_components
            .as_ref()
            .is_some_and(|raw| !raw.as_value().is_object())
        {
            return Err(invalid_raw_filter(consumer, "components"));
        }
        if self
            .partial_raw_predicates
            .as_ref()
            .is_some_and(|raw| !raw.as_value().is_object())
        {
            return Err(invalid_raw_filter(consumer, "predicates"));
        }

        let mut pred = ItemPredicate::new();
        for item in &self.items {
            pred = pred.item(item.clone());
        }
        if let Some(count) = self.count {
            pred = pred.count(count);
        }
        for key in &self.partial_custom_data_keys {
            pred = pred.custom_data_key(key.clone());
        }

        let mut exact_components = serde_json::Map::new();
        if let Some(ref data) = self.exact_custom_data {
            if self.exact_raw_components.as_ref().is_some_and(|raw| {
                raw.as_value()
                    .as_object()
                    .is_some_and(|object| object.contains_key("minecraft:custom_data"))
            }) {
                return Err(overlapping_raw_filter(
                    consumer,
                    "components.minecraft:custom_data",
                ));
            }
            let json = data
                .to_json()
                .ok_or_else(|| unsupported_raw_custom_data(consumer))?;
            exact_components.insert("minecraft:custom_data".to_string(), json);
        }
        if let Some(ref raw) = self.exact_raw_components
            && let Value::Object(obj) = raw.as_value()
        {
            for (k, v) in obj {
                exact_components.insert(k.clone(), v.clone());
            }
        }
        if !exact_components.is_empty() {
            pred = pred.raw_components(RawJson::new(Value::Object(exact_components)));
        }

        let mut partial_predicates = serde_json::Map::new();
        if let Some(raw) = &self.partial_raw_predicates
            && let Some(object) = raw.as_value().as_object()
        {
            for (key, typed_present) in [
                (
                    "minecraft:custom_data",
                    !self.partial_custom_data_keys.is_empty(),
                ),
                ("minecraft:enchantments", !self.enchantments.is_empty()),
                ("minecraft:damage", self.damage_range.is_some()),
            ] {
                if typed_present && object.contains_key(key) {
                    return Err(overlapping_raw_filter(
                        consumer,
                        &format!("predicates.{key}"),
                    ));
                }
            }
        }
        if !self.enchantments.is_empty() {
            let arr: Vec<Value> = self
                .enchantments
                .iter()
                .map(|e| {
                    let mut m = serde_json::Map::new();
                    m.insert("enchantment".to_string(), Value::String(e.id.to_string()));
                    if let Some(levels) = e.levels {
                        m.insert(
                            "levels".to_string(),
                            serde_json::to_value(levels).unwrap_or(Value::Null),
                        );
                    }
                    Value::Object(m)
                })
                .collect();
            partial_predicates.insert("minecraft:enchantments".to_string(), Value::Array(arr));
        }
        if let Some(range) = self.damage_range {
            partial_predicates.insert(
                "minecraft:damage".to_string(),
                serde_json::json!({ "damage": range }),
            );
        }
        if let Some(ref raw) = self.partial_raw_predicates
            && let Value::Object(obj) = raw.as_value()
        {
            for (k, v) in obj {
                partial_predicates.insert(k.clone(), v.clone());
            }
        }
        if !partial_predicates.is_empty() {
            pred = pred.raw_predicates(RawJson::new(Value::Object(partial_predicates)));
        }

        Ok(pred)
    }

    /// Convenience wrapper for the advancement seam PR #237 introduced.
    /// Identical to
    /// `try_render_for(ItemMatcherConsumer::Advancement { consumer }, caps)`.
    pub fn try_into_advancement_predicate(
        &self,
        consumer: AdvancementItemConsumer,
        caps: Option<&sand_version::VersionCaps>,
    ) -> SandResult<ItemPredicate> {
        self.try_render_for(ItemMatcherConsumer::Advancement { consumer }, caps)
    }
}

/// Converts a value into an [`ItemMatcher`].
///
/// Implemented for `ItemMatcher` itself so APIs that accept
/// `impl IntoItemMatcher` can also accept an already-built matcher directly.
pub trait IntoItemMatcher {
    fn into_item_matcher(self) -> ItemMatcher;
}

impl IntoItemMatcher for ItemMatcher {
    fn into_item_matcher(self) -> ItemMatcher {
        self
    }
}

/// Fallible conversion into a [`predicates::ItemPredicate`](crate::predicates::ItemPredicate)
/// for a specific consumer and version profile.
///
/// Mirrors [`ItemMatcher::try_render_for`] as a trait so generic code can be
/// written over anything that can produce a predicate, not just `ItemMatcher`
/// directly.
pub trait TryIntoItemPredicate {
    fn try_into_item_predicate(
        &self,
        consumer: ItemMatcherConsumer,
        caps: Option<&sand_version::VersionCaps>,
    ) -> SandResult<ItemPredicate>;
}

impl TryIntoItemPredicate for ItemMatcher {
    fn try_into_item_predicate(
        &self,
        consumer: ItemMatcherConsumer,
        caps: Option<&sand_version::VersionCaps>,
    ) -> SandResult<ItemPredicate> {
        self.try_render_for(consumer, caps)
    }
}

/// Which consuming surface an [`ItemMatcher`] is being converted for.
///
/// Named per-surface (rather than a single boolean or an untyped string) so
/// diagnostics can identify exactly which consumer rejected an unsupported
/// matcher, and so each consumer's conversion rules stay in one match arm
/// instead of scattered `if` checks. [`Self::Advancement`] wraps
/// [`AdvancementItemConsumer`], the narrowly-scoped precursor to this enum
/// PR #237 introduced for `placed_block`/`item_used_on_block` — this is the
/// generalization that seam was documented as a placeholder for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemMatcherConsumer {
    /// An advancement trigger's item/tool filter field.
    Advancement { consumer: AdvancementItemConsumer },
    /// A general-purpose item predicate (selectors, execute conditions,
    /// entity equipment checks, ...).
    Predicate,
    /// A recipe ingredient slot. Vanilla recipe ingredients match only by
    /// item ID or tag in every Minecraft version Sand targets — any matcher
    /// with [`ItemMatcher::has_component_constraints`] always fails here,
    /// mirroring [`crate::recipe::Ingredient::custom_item`]'s existing
    /// "always error" behavior rather than silently degrading to
    /// base-item-only matching.
    RecipeIngredient,
    /// An inventory/execute item condition (e.g. `execute if items entity`).
    InventoryCondition,
    /// A loot table condition.
    LootCondition,
}

impl ItemMatcherConsumer {
    /// A human-readable label for this consumer, used in diagnostics.
    pub fn label(self) -> String {
        match self {
            Self::Advancement { consumer } => {
                format!("advancement trigger `{}`", consumer.trigger_id())
            }
            Self::Predicate => "item predicate".to_string(),
            Self::RecipeIngredient => "recipe ingredient".to_string(),
            Self::InventoryCondition => "inventory/execute item condition".to_string(),
            Self::LootCondition => "loot table condition".to_string(),
        }
    }
}

impl From<AdvancementItemConsumer> for ItemMatcherConsumer {
    fn from(consumer: AdvancementItemConsumer) -> Self {
        Self::Advancement { consumer }
    }
}

fn item_matcher_location() -> ResourceLocation {
    ResourceLocation::new("sand", "item_matcher").expect("static resource location is always valid")
}

/// Shared diagnostic for requesting item-component matching (custom data,
/// enchantments, damage, or raw component/predicate constraints) on a
/// pre-1.20.5 profile that doesn't support item components.
///
/// This is the same failure [`crate::advancement::AdvancementTrigger::render_for`]
/// raises for `placed_block`/`item_used_on_block` item filters on
/// [`crate::advancement::AdvancementSchemaFamily::Legacy`] — that call site now
/// delegates here so the diagnostic text and the underlying capability check
/// have exactly one source, instead of two parallel implementations that
/// could drift.
pub(crate) fn unsupported_legacy_item_filter(consumer: ItemMatcherConsumer) -> SandError {
    SandError::ComponentValidation {
        location: item_matcher_location(),
        kind: consumer.label(),
        field: "components".to_string(),
        message: format!(
            "{} requested item-component matching (custom data, enchantments, damage, or raw \
             component/predicate constraints), but the target Minecraft profile is a \
             pre-item-component profile (predates 1.20.5). Sand's item predicate model only \
             renders the `components`/`predicates` schema, which this profile does not \
             recognize. Target a supported item-component profile (every currently-supported \
             1.20.5+ and 26.x profile), drop the component constraint and match by item ID \
             only, or use a manually-verified legacy predicate/raw JSON escape hatch.",
            consumer.label()
        ),
    }
}

fn unsupported_raw_custom_data(consumer: ItemMatcherConsumer) -> SandError {
    SandError::ComponentValidation {
        location: item_matcher_location(),
        kind: consumer.label(),
        field: "custom_data".to_string(),
        message: format!(
            "{} requested exact `minecraft:custom_data` matching against a raw-SNBT \
             `CustomData::Raw` payload, which has no general SNBT-to-JSON conversion. Use \
             `CustomData::Marker` for exact marker-key matching, or \
             `ItemMatcher::raw_components_exact(...)` with the equivalent JSON directly.",
            consumer.label()
        ),
    }
}

fn overlapping_raw_filter(consumer: ItemMatcherConsumer, field: &str) -> SandError {
    SandError::ComponentValidation {
        location: item_matcher_location(),
        kind: consumer.label(),
        field: field.to_string(),
        message: format!(
            "{} supplies both a typed filter and a raw value for `{field}`; the raw value would overwrite and silently weaken the typed request. Combine the constraint in one representation.",
            consumer.label()
        ),
    }
}

fn invalid_raw_filter(consumer: ItemMatcherConsumer, field: &str) -> SandError {
    SandError::ComponentValidation {
        location: item_matcher_location(),
        kind: consumer.label(),
        field: field.to_string(),
        message: format!(
            "{} supplied raw `{field}` as a non-object JSON value; item predicate `{field}` must be an object and cannot be dropped or broadened",
            consumer.label()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::CustomData;

    fn id(path: &str) -> ItemId {
        ItemId::minecraft(path).unwrap()
    }

    fn modern_caps() -> sand_version::VersionCaps {
        sand_version::VersionCaps::all_enabled()
    }

    fn legacy_caps() -> sand_version::VersionCaps {
        sand_version::VersionCaps::all_disabled()
    }

    #[test]
    fn exact_item_match_renders_items_array() {
        let matcher = ItemMatcher::item(id("bow"));
        let pred = matcher
            .try_render_for(ItemMatcherConsumer::Predicate, Some(&modern_caps()))
            .unwrap();
        assert_eq!(pred.items, Some(vec!["minecraft:bow".to_string()]));
    }

    #[test]
    fn exact_custom_data_renders_under_components_not_predicates() {
        let matcher =
            ItemMatcher::item(id("bow")).custom_data_exact(CustomData::marker("special_bow"));
        let pred = matcher
            .try_render_for(ItemMatcherConsumer::Predicate, Some(&modern_caps()))
            .unwrap();
        let json = serde_json::to_value(&pred).unwrap();
        assert_eq!(
            json["components"]["minecraft:custom_data"],
            serde_json::json!({ "special_bow": true })
        );
        assert!(json.get("predicates").is_none());
    }

    #[test]
    fn partial_custom_data_renders_under_predicates_not_components() {
        let matcher = ItemMatcher::item(id("bow")).custom_data_partial("special_bow");
        let pred = matcher
            .try_render_for(ItemMatcherConsumer::Predicate, Some(&modern_caps()))
            .unwrap();
        let json = serde_json::to_value(&pred).unwrap();
        assert!(json.get("components").is_none());
        assert_eq!(
            json["predicates"]["minecraft:custom_data"],
            "{special_bow:1b}"
        );
    }

    #[test]
    fn raw_partial_custom_data_cannot_overwrite_typed_partial_filter() {
        let matcher = ItemMatcher::item(id("bow"))
            .custom_data_partial("special_bow")
            .raw_predicates_partial(RawJson::new(serde_json::json!({
                "minecraft:custom_data": "{other:1b}"
            })));
        let error = matcher
            .try_render_for(ItemMatcherConsumer::Predicate, Some(&modern_caps()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("overwrite"), "{error}");
        assert!(error.contains("minecraft:custom_data"), "{error}");
    }

    #[test]
    fn raw_exact_custom_data_cannot_overwrite_typed_exact_filter() {
        let matcher = ItemMatcher::item(id("bow"))
            .custom_data_exact(CustomData::marker("special_bow"))
            .raw_components_exact(RawJson::new(serde_json::json!({
                "minecraft:custom_data": {"other": true}
            })));
        let error = matcher
            .try_render_for(ItemMatcherConsumer::Predicate, Some(&modern_caps()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("overwrite"), "{error}");
        assert!(error.contains("minecraft:custom_data"), "{error}");
    }

    #[test]
    fn non_object_raw_filters_fail_instead_of_disappearing() {
        for matcher in [
            ItemMatcher::item(id("bow"))
                .raw_components_exact(RawJson::new(serde_json::json!(["invalid"]))),
            ItemMatcher::item(id("bow"))
                .raw_predicates_partial(RawJson::new(serde_json::json!("invalid"))),
        ] {
            let error = matcher
                .try_render_for(ItemMatcherConsumer::Predicate, Some(&modern_caps()))
                .unwrap_err()
                .to_string();
            assert!(error.contains("non-object"), "{error}");
            assert!(error.contains("cannot be dropped"), "{error}");
        }
    }

    #[test]
    fn enchantment_match_renders_partial_predicate() {
        let matcher = ItemMatcher::item(id("bow")).enchantment_levels(
            EnchantmentId::minecraft("power").unwrap(),
            IntRange::at_least(1),
        );
        let pred = matcher
            .try_render_for(ItemMatcherConsumer::Predicate, Some(&modern_caps()))
            .unwrap();
        let json = serde_json::to_value(&pred).unwrap();
        assert_eq!(
            json["predicates"]["minecraft:enchantments"][0]["enchantment"],
            "minecraft:power"
        );
        assert_eq!(
            json["predicates"]["minecraft:enchantments"][0]["levels"]["min"],
            1
        );
    }

    #[test]
    fn damage_range_renders_partial_predicate() {
        let matcher = ItemMatcher::item(id("bow")).damage_range(IntRange::at_most(50));
        let pred = matcher
            .try_render_for(ItemMatcherConsumer::Predicate, Some(&modern_caps()))
            .unwrap();
        let json = serde_json::to_value(&pred).unwrap();
        assert_eq!(json["predicates"]["minecraft:damage"]["damage"]["max"], 50);
    }

    #[test]
    fn unsupported_consumer_version_combination_fails_not_weakens() {
        let matcher = ItemMatcher::item(id("bow")).custom_data_partial("special_bow");
        let err = matcher
            .try_render_for(ItemMatcherConsumer::Predicate, Some(&legacy_caps()))
            .expect_err("component constraint on a legacy profile must fail");
        assert!(err.to_string().contains("item predicate"));
        assert!(err.to_string().contains("1.20.5"));
    }

    #[test]
    fn item_id_only_matcher_succeeds_on_legacy_profile() {
        // No component constraints — base item matching alone is not
        // gated by the item-component system.
        let matcher = ItemMatcher::item(id("bow"));
        assert!(
            matcher
                .try_render_for(ItemMatcherConsumer::Predicate, Some(&legacy_caps()))
                .is_ok()
        );
    }

    #[test]
    fn raw_snbt_exact_custom_data_fails_clearly() {
        use crate::raw::RawSnbt;
        let matcher =
            ItemMatcher::item(id("bow")).custom_data_exact(CustomData::raw(RawSnbt::new("{a:1b}")));
        let err = matcher
            .try_render_for(ItemMatcherConsumer::Predicate, Some(&modern_caps()))
            .expect_err("raw SNBT custom data has no exact JSON form");
        assert!(err.to_string().contains("custom_data"));
    }

    #[test]
    fn advancement_consumer_label_names_the_trigger() {
        let consumer = ItemMatcherConsumer::Advancement {
            consumer: AdvancementItemConsumer::PlacedBlockTool,
        };
        assert!(consumer.label().contains("minecraft:placed_block"));
    }

    #[test]
    fn none_caps_behaves_like_fully_modern_profile() {
        let matcher = ItemMatcher::item(id("bow")).custom_data_partial("special_bow");
        assert!(
            matcher
                .try_render_for(ItemMatcherConsumer::Predicate, None)
                .is_ok()
        );
    }
}
