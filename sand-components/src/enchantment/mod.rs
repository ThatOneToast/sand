//! Builder for `data/<namespace>/enchantment/` JSON files (Minecraft 1.21+).
//!
//! Enchantment definitions control how enchantments are applied, their effects,
//! costs, and which items they can appear on.
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `supported_items` must be set to a typed [`ItemId`] or `TagId<ItemId>`.
//! - `slots` must contain at least one entry; each is a typed
//!   [`EquipmentSlotGroup`] or a raw slot name added via [`Enchantment::raw_slot`]
//!   / [`Enchantment::raw_slots`].
//! - `weight` must be in `1..=1024`.
//! - `max_level` must be in `1..=255`.
//! - `description` must be set, either as a typed [`TextComponent`] or a
//!   [`RawJson`] escape hatch.
//! - `primary_items` and `exclusive_set`, when present, must be typed
//!   item/tag or enchantment/tag references.
//! - typed and raw effect entries share one `effects` map; the whole-map raw
//!   escape hatch ([`Enchantment::raw_effects`]) must be a JSON object.
//!
//! # Typed effect coverage
//!
//! Minecraft 1.21's enchantment effect schema is large and version-sensitive.
//! Sand currently models the common "value effect" shape used by
//! `minecraft:damage`, `minecraft:knockback`, and `minecraft:armor_effectiveness`
//! — each of which wraps a [`LevelBasedValue`] behind an
//! [`EnchantmentValueOperation`] (`minecraft:add` / `minecraft:set`). Any other
//! effect component (or a `minecraft:multiply`/`minecraft:all_of`/etc. operation)
//! is available through [`Enchantment::raw_effect_component`] or
//! [`Enchantment::raw_effects`].
//!
//! # Example
//! ```rust
//! use sand_components::enchantment::{
//!     Enchantment, EnchantmentValueOperation, LevelBasedValue,
//! };
//! use sand_components::registry::{ItemId, TagId};
//! use sand_components::{DatapackComponent, EquipmentSlotGroup, ResourceLocation};
//! use sand_commands::TextComponent;
//!
//! let rl = ResourceLocation::new("mypack", "swift_step").unwrap();
//! let enchantment = Enchantment::new(rl)
//!     .description(TextComponent::translate("enchantment.mypack.swift_step"))
//!     .supported_items(TagId::<ItemId>::minecraft("enchantable/foot_armor").unwrap())
//!     .slot(EquipmentSlotGroup::Feet)
//!     .knockback_effect(
//!         EnchantmentValueOperation::Add,
//!         LevelBasedValue::Linear { base: 0.5, per_level_above_first: 0.25 },
//!     );
//! assert!(enchantment.validate().is_ok());
//! ```

use std::collections::BTreeMap;

use sand_commands::{CommandProfile, TextComponent};
use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::item::EquipmentSlotGroup;
use crate::raw::RawJson;
use crate::registry::{EnchantmentEffectComponentId, EnchantmentId, ItemId, TagId};
use crate::resource_location::ResourceLocation;
use crate::validation;

// ── EnchantmentCost ───────────────────────────────────────────────────────────

/// The level cost configuration for enchanting (min or max enchanting-table cost).
#[derive(Clone)]
pub struct EnchantmentCost {
    /// Base cost at enchantment level 1.
    pub base: u32,
    /// Additional cost added per enchantment level above 1.
    pub per_level_above_first: u32,
}

impl EnchantmentCost {
    /// Creates a new cost with the given base and per-level values.
    pub fn new(base: u32, per_level_above_first: u32) -> Self {
        Self {
            base,
            per_level_above_first,
        }
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "base": self.base,
            "per_level_above_first": self.per_level_above_first,
        })
    }
}

// ── ItemOrTag / EnchantmentOrTag ─────────────────────────────────────────────

/// A typed reference to a single item or an item tag, used by
/// [`Enchantment::supported_items`] and [`Enchantment::primary_items`].
///
/// Construct one from an [`ItemId`] or `TagId<ItemId>` — both convert
/// automatically via `Into`.
#[derive(Debug, Clone)]
pub struct ItemOrTag(ItemOrTagRepr);

#[derive(Debug, Clone)]
enum ItemOrTagRepr {
    Item(ItemId),
    Tag(TagId<ItemId>),
}

impl From<ItemId> for ItemOrTag {
    fn from(id: ItemId) -> Self {
        Self(ItemOrTagRepr::Item(id))
    }
}

impl From<TagId<ItemId>> for ItemOrTag {
    fn from(tag: TagId<ItemId>) -> Self {
        Self(ItemOrTagRepr::Tag(tag))
    }
}

impl ItemOrTag {
    fn to_json_string(&self) -> String {
        match &self.0 {
            ItemOrTagRepr::Item(id) => id.to_string(),
            ItemOrTagRepr::Tag(tag) => tag.to_tag_string(),
        }
    }
}

/// A typed reference to a single enchantment or an enchantment tag, used by
/// [`Enchantment::exclusive_set`].
///
/// Construct one from an [`EnchantmentId`] or `TagId<EnchantmentId>` — both
/// convert automatically via `Into`.
#[derive(Debug, Clone)]
pub struct EnchantmentOrTag(EnchantmentOrTagRepr);

#[derive(Debug, Clone)]
enum EnchantmentOrTagRepr {
    Enchantment(EnchantmentId),
    Tag(TagId<EnchantmentId>),
}

impl From<EnchantmentId> for EnchantmentOrTag {
    fn from(id: EnchantmentId) -> Self {
        Self(EnchantmentOrTagRepr::Enchantment(id))
    }
}

impl From<TagId<EnchantmentId>> for EnchantmentOrTag {
    fn from(tag: TagId<EnchantmentId>) -> Self {
        Self(EnchantmentOrTagRepr::Tag(tag))
    }
}

impl EnchantmentOrTag {
    fn to_json_string(&self) -> String {
        match &self.0 {
            EnchantmentOrTagRepr::Enchantment(id) => id.to_string(),
            EnchantmentOrTagRepr::Tag(tag) => tag.to_tag_string(),
        }
    }
}

// ── EnchantmentDescription ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum EnchantmentDescription {
    Typed(Box<TextComponent>),
    Raw(RawJson),
}

impl EnchantmentDescription {
    fn validate(&self, location: &ResourceLocation, kind: &str, field: &str) -> SandResult<()> {
        if let Self::Typed(text) = self {
            text.validate_at_path(&CommandProfile::unprofiled(), field)
                .map_err(|error| {
                    validation::error(
                        location,
                        kind,
                        &error.field,
                        &format!("error[{}] {}", error.code, error.message),
                    )
                })?;
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        match self {
            Self::Typed(text) => serde_json::from_str(&text.to_string())
                .expect("TextComponent always renders structurally valid JSON"),
            Self::Raw(raw) => raw.as_value().clone(),
        }
    }
}

// ── EnchantmentValueOperation / LevelBasedValue ──────────────────────────────

/// How a typed value effect combines with any existing value for the same
/// effect component (Minecraft's `ValueEffect` operation kinds).
///
/// `minecraft:multiply`, `minecraft:remove_binomial`, and `minecraft:all_of`
/// are not modelled yet; use [`Enchantment::raw_effect_component`] for those.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnchantmentValueOperation {
    /// `minecraft:add` — adds the level-based value to the existing value.
    Add,
    /// `minecraft:set` — overwrites the existing value.
    Set,
}

impl EnchantmentValueOperation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Add => "minecraft:add",
            Self::Set => "minecraft:set",
        }
    }
}

/// A level-scaled numeric value used by typed enchantment value effects
/// (Minecraft's `LevelBasedValue`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelBasedValue {
    /// A value that does not depend on enchantment level. Serializes as a
    /// bare JSON number (Minecraft's documented shorthand).
    Constant(f64),
    /// `minecraft:linear` — `base + per_level_above_first * (level - 1)`.
    Linear {
        base: f64,
        per_level_above_first: f64,
    },
}

impl LevelBasedValue {
    fn validate(&self, location: &ResourceLocation, kind: &str, field: &str) -> SandResult<()> {
        match self {
            Self::Constant(v) => {
                if !v.is_finite() {
                    return Err(validation::error(
                        location,
                        kind,
                        field,
                        &format!("{field} must be finite; received {v}"),
                    ));
                }
            }
            Self::Linear {
                base,
                per_level_above_first,
            } => {
                if !base.is_finite() {
                    return Err(validation::error(
                        location,
                        kind,
                        field,
                        &format!("{field}.base must be finite; received {base}"),
                    ));
                }
                if !per_level_above_first.is_finite() {
                    return Err(validation::error(
                        location,
                        kind,
                        field,
                        &format!(
                            "{field}.per_level_above_first must be finite; received {per_level_above_first}"
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    fn to_json(self) -> Value {
        match self {
            Self::Constant(v) => serde_json::json!(v),
            Self::Linear {
                base,
                per_level_above_first,
            } => serde_json::json!({
                "type": "minecraft:linear",
                "base": base,
                "per_level_above_first": per_level_above_first,
            }),
        }
    }
}

/// One entry in an effect component's array — either a typed value effect or
/// a raw JSON escape-hatch entry.
#[derive(Debug, Clone)]
enum EffectEntry {
    Value {
        operation: EnchantmentValueOperation,
        value: LevelBasedValue,
    },
    Raw(Value),
}

impl EffectEntry {
    fn validate(&self, location: &ResourceLocation, kind: &str, field: &str) -> SandResult<()> {
        if let Self::Value { value, .. } = self {
            value.validate(location, kind, field)?;
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        match self {
            Self::Value { operation, value } => serde_json::json!({
                "effect": {
                    "type": operation.as_str(),
                    "value": value.to_json(),
                }
            }),
            Self::Raw(value) => value.clone(),
        }
    }
}

// ── Enchantment ───────────────────────────────────────────────────────────────

/// A single equipment-slot entry: typed [`EquipmentSlotGroup`] or a raw name.
#[derive(Debug, Clone)]
enum SlotEntry {
    Typed(EquipmentSlotGroup),
    Raw(String),
}

impl SlotEntry {
    fn as_json_string(&self) -> String {
        match self {
            Self::Typed(slot) => slot.as_str().to_string(),
            Self::Raw(name) => name.clone(),
        }
    }

    fn known_name(&self) -> bool {
        match self {
            Self::Typed(_) => true,
            Self::Raw(name) => matches!(
                name.as_str(),
                "any"
                    | "mainhand"
                    | "offhand"
                    | "hand"
                    | "feet"
                    | "legs"
                    | "chest"
                    | "head"
                    | "armor"
                    | "body"
            ),
        }
    }
}

/// An enchantment definition (`data/<namespace>/enchantment/<id>.json`).
pub struct Enchantment {
    location: ResourceLocation,
    description: Option<EnchantmentDescription>,
    supported_items: Option<ItemOrTag>,
    primary_items: Option<ItemOrTag>,
    exclusive_set: Option<EnchantmentOrTag>,
    weight: u32,
    max_level: u32,
    min_cost: EnchantmentCost,
    max_cost: EnchantmentCost,
    anvil_cost: u32,
    slots: Vec<SlotEntry>,
    effects: BTreeMap<String, Vec<EffectEntry>>,
    raw_effects: Option<RawJson>,
}

impl Enchantment {
    /// Creates a new enchantment with sensible defaults.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            description: None,
            supported_items: None,
            primary_items: None,
            exclusive_set: None,
            weight: 10,
            max_level: 1,
            min_cost: EnchantmentCost::new(1, 11),
            max_cost: EnchantmentCost::new(21, 11),
            anvil_cost: 2,
            slots: Vec::new(),
            effects: BTreeMap::new(),
            raw_effects: None,
        }
    }

    /// Sets the description as a typed text component.
    pub fn description(mut self, desc: TextComponent) -> Self {
        self.description = Some(EnchantmentDescription::Typed(Box::new(desc)));
        self
    }

    /// Convenience: sets the description as a plain translation key.
    pub fn description_translate(mut self, key: impl Into<String>) -> Self {
        self.description = Some(EnchantmentDescription::Typed(Box::new(
            TextComponent::translate(key),
        )));
        self
    }

    /// Use a raw JSON text component when the typed text API cannot represent it.
    pub fn raw_description(mut self, desc: RawJson) -> Self {
        self.description = Some(EnchantmentDescription::Raw(desc));
        self
    }

    /// Sets the supported items — the items this enchantment can be applied
    /// to (e.g. any sword, or a specific item). Accepts an [`ItemId`] or a
    /// `TagId<ItemId>`.
    pub fn supported_items(mut self, items: impl Into<ItemOrTag>) -> Self {
        self.supported_items = Some(items.into());
        self
    }

    /// Sets the primary items — the subset of supported items this
    /// enchantment appears for at an enchanting table. Accepts an [`ItemId`]
    /// or a `TagId<ItemId>`.
    pub fn primary_items(mut self, items: impl Into<ItemOrTag>) -> Self {
        self.primary_items = Some(items.into());
        self
    }

    /// Sets the exclusive set — enchantments sharing this set (or tag) are
    /// mutually exclusive. Accepts an [`EnchantmentId`] or a
    /// `TagId<EnchantmentId>`.
    pub fn exclusive_set(mut self, tag: impl Into<EnchantmentOrTag>) -> Self {
        self.exclusive_set = Some(tag.into());
        self
    }

    /// Sets the enchantment weight (higher = more common, 1–1024).
    pub fn weight(mut self, w: u32) -> Self {
        self.weight = w;
        self
    }

    /// Sets the maximum enchantment level (1–255).
    pub fn max_level(mut self, lvl: u32) -> Self {
        self.max_level = lvl;
        self
    }

    /// Sets the minimum enchanting-table cost.
    pub fn min_cost(mut self, cost: EnchantmentCost) -> Self {
        self.min_cost = cost;
        self
    }

    /// Sets the maximum enchanting-table cost.
    pub fn max_cost(mut self, cost: EnchantmentCost) -> Self {
        self.max_cost = cost;
        self
    }

    /// Sets the anvil cost (XP levels).
    pub fn anvil_cost(mut self, cost: u32) -> Self {
        self.anvil_cost = cost;
        self
    }

    /// Adds a typed equipment slot this enchantment is active in.
    pub fn slot(mut self, slot: EquipmentSlotGroup) -> Self {
        self.slots.push(SlotEntry::Typed(slot));
        self
    }

    /// Sets all active equipment slots from typed values.
    pub fn slots(mut self, slots: impl IntoIterator<Item = EquipmentSlotGroup>) -> Self {
        self.slots = slots.into_iter().map(SlotEntry::Typed).collect();
        self
    }

    /// Adds a raw equipment slot name — an escape hatch for slot groups not
    /// yet represented by [`EquipmentSlotGroup`].
    pub fn raw_slot(mut self, slot: impl Into<String>) -> Self {
        self.slots.push(SlotEntry::Raw(slot.into()));
        self
    }

    /// Sets all active equipment slots from raw names — an escape hatch for
    /// slot groups not yet represented by [`EquipmentSlotGroup`].
    pub fn raw_slots(mut self, slots: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.slots = slots
            .into_iter()
            .map(|s| SlotEntry::Raw(s.into()))
            .collect();
        self
    }

    /// Adds a typed `minecraft:damage` value effect (used by Sharpness,
    /// Smite, Bane of Arthropods, Impaling, Power).
    pub fn damage_effect(
        self,
        operation: EnchantmentValueOperation,
        value: LevelBasedValue,
    ) -> Self {
        self.value_effect(EnchantmentEffectComponentId::damage(), operation, value)
    }

    /// Adds a typed `minecraft:knockback` value effect (used by Knockback,
    /// Punch).
    pub fn knockback_effect(
        self,
        operation: EnchantmentValueOperation,
        value: LevelBasedValue,
    ) -> Self {
        self.value_effect(EnchantmentEffectComponentId::knockback(), operation, value)
    }

    /// Adds a typed `minecraft:armor_effectiveness` value effect (used by
    /// Breach).
    pub fn armor_effectiveness_effect(
        self,
        operation: EnchantmentValueOperation,
        value: LevelBasedValue,
    ) -> Self {
        self.value_effect(
            EnchantmentEffectComponentId::armor_effectiveness(),
            operation,
            value,
        )
    }

    /// Adds a typed value effect under an arbitrary effect component ID.
    /// Prefer the dedicated `*_effect` helpers for the well-known vanilla
    /// value effects; use this for custom namespaced components that share
    /// the same `{"effect": {"type": ..., "value": ...}}` shape.
    pub fn value_effect(
        mut self,
        component: EnchantmentEffectComponentId,
        operation: EnchantmentValueOperation,
        value: LevelBasedValue,
    ) -> Self {
        self.effects
            .entry(component.to_string())
            .or_default()
            .push(EffectEntry::Value { operation, value });
        self
    }

    /// Adds a raw JSON effect entry under a typed effect component ID — an
    /// escape hatch for effect shapes Sand does not yet model (e.g.
    /// `minecraft:attributes`, `minecraft:all_of`, or custom/modded effects).
    pub fn raw_effect_component(
        mut self,
        component: EnchantmentEffectComponentId,
        value: RawJson,
    ) -> Self {
        self.effects
            .entry(component.to_string())
            .or_default()
            .push(EffectEntry::Raw(value.as_value().clone()));
        self
    }

    /// Sets the entire effects map as raw JSON, replacing any typed or
    /// per-component raw entries added so far. An escape hatch for whole
    /// custom effect maps.
    pub fn raw_effects(mut self, effects: RawJson) -> Self {
        self.raw_effects = Some(effects);
        self
    }
}

impl DatapackComponent for Enchantment {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "enchantment";

        if self.supported_items.is_none() {
            return Err(validation::error(
                &self.location,
                kind,
                "supported_items",
                "supported_items must be set",
            ));
        }

        validation::require_non_empty_collection(&self.location, kind, "slots", self.slots.len())?;
        for (i, slot) in self.slots.iter().enumerate() {
            if !slot.known_name() {
                return Err(validation::error(
                    &self.location,
                    kind,
                    &format!("slots[{i}]"),
                    &format!(
                        "`{}` is not a valid enchantment slot; \
                         expected one of: any, mainhand, offhand, hand, \
                         feet, legs, chest, head, armor, body",
                        slot.as_json_string()
                    ),
                ));
            }
        }

        validation::require_u32_in_range(&self.location, kind, "weight", self.weight, 1, 1024)?;
        validation::require_u32_in_range(
            &self.location,
            kind,
            "max_level",
            self.max_level,
            1,
            255,
        )?;

        match &self.description {
            Some(description) => description.validate(&self.location, kind, "description")?,
            None => {
                return Err(validation::error(
                    &self.location,
                    kind,
                    "description",
                    "description must be set",
                ));
            }
        }

        if let Some(raw) = &self.raw_effects {
            validation::require_json_object(&self.location, kind, "effects", raw.as_value())?;
        } else {
            for (component, entries) in &self.effects {
                for (i, entry) in entries.iter().enumerate() {
                    entry.validate(&self.location, kind, &format!("effects.{component}[{i}]"))?;
                }
            }
        }

        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        if let Some(description) = &self.description {
            map.insert("description".to_string(), description.to_json());
        }
        if let Some(supported_items) = &self.supported_items {
            map.insert(
                "supported_items".to_string(),
                Value::String(supported_items.to_json_string()),
            );
        }
        if let Some(primary_items) = &self.primary_items {
            map.insert(
                "primary_items".to_string(),
                Value::String(primary_items.to_json_string()),
            );
        }
        if let Some(exclusive_set) = &self.exclusive_set {
            map.insert(
                "exclusive_set".to_string(),
                Value::String(exclusive_set.to_json_string()),
            );
        }
        map.insert("weight".to_string(), Value::Number(self.weight.into()));
        map.insert(
            "max_level".to_string(),
            Value::Number(self.max_level.into()),
        );
        map.insert("min_cost".to_string(), self.min_cost.to_json());
        map.insert("max_cost".to_string(), self.max_cost.to_json());
        map.insert(
            "anvil_cost".to_string(),
            Value::Number(self.anvil_cost.into()),
        );
        map.insert(
            "slots".to_string(),
            Value::Array(
                self.slots
                    .iter()
                    .map(|s| Value::String(s.as_json_string()))
                    .collect(),
            ),
        );
        if let Some(raw) = &self.raw_effects {
            map.insert("effects".to_string(), raw.as_value().clone());
        } else if !self.effects.is_empty() {
            let effects = self
                .effects
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        Value::Array(v.iter().map(EffectEntry::to_json).collect()),
                    )
                })
                .collect();
            map.insert("effects".to_string(), Value::Object(effects));
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "enchantment"
    }

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::Enchantments]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "swift_step").unwrap()
    }

    fn valid() -> Enchantment {
        Enchantment::new(rl())
            .description_translate("enchantment.test.swift_step")
            .supported_items(TagId::<ItemId>::minecraft("enchantable/foot_armor").unwrap())
            .slot(EquipmentSlotGroup::Feet)
    }

    #[test]
    fn valid_minimal_enchantment_exports_deterministic_json() {
        let ench = valid();
        assert!(ench.validate().is_ok());
        let a = serde_json::to_string_pretty(&ench.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&ench.to_json()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn missing_supported_items_is_rejected() {
        let ench = Enchantment::new(rl())
            .description_translate("x")
            .slot(EquipmentSlotGroup::Any);
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("supported_items"), "{err}");
    }

    #[test]
    fn empty_slots_is_rejected() {
        let ench = Enchantment::new(rl())
            .description_translate("x")
            .supported_items(TagId::<ItemId>::minecraft("enchantable/sword").unwrap());
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("slots"), "{err}");
    }

    #[test]
    fn invalid_raw_slot_name_is_rejected() {
        let ench = valid().raw_slot("invalid_slot");
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("slots["), "{err}");
    }

    #[test]
    fn weight_zero_is_rejected() {
        let ench = valid().weight(0);
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("weight"), "{err}");
    }

    #[test]
    fn weight_1025_is_rejected() {
        let ench = valid().weight(1025);
        assert!(ench.validate().is_err());
    }

    #[test]
    fn weight_one_is_accepted() {
        let ench = valid().weight(1);
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn weight_1024_is_accepted() {
        let ench = valid().weight(1024);
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn max_level_zero_is_rejected() {
        let ench = valid().max_level(0);
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("max_level"), "{err}");
    }

    #[test]
    fn max_level_256_is_rejected() {
        let ench = valid().max_level(256);
        assert!(ench.validate().is_err());
    }

    #[test]
    fn max_level_one_is_accepted() {
        let ench = valid().max_level(1);
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn max_level_255_is_accepted() {
        let ench = valid().max_level(255);
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn supported_items_accepts_plain_item_id() {
        let ench = valid().supported_items(ItemId::minecraft("diamond_boots").unwrap());
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn primary_items_accepts_tag_reference() {
        let ench = valid().primary_items(TagId::<ItemId>::minecraft("enchantable/sword").unwrap());
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn exclusive_set_accepts_tag_reference() {
        let ench = valid()
            .exclusive_set(TagId::<EnchantmentId>::minecraft("exclusive_set/boots").unwrap());
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn exclusive_set_accepts_plain_enchantment_id() {
        let ench = valid().exclusive_set(EnchantmentId::minecraft("damage").unwrap());
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn malformed_item_id_is_rejected_at_construction() {
        assert!(ItemId::minecraft("Not Valid!").is_err());
    }

    #[test]
    fn malformed_tag_is_rejected_at_construction() {
        assert!(TagId::<ItemId>::minecraft("Not Valid!").is_err());
    }

    #[test]
    fn non_object_raw_effects_is_rejected() {
        let ench = valid().raw_effects(RawJson::new(serde_json::json!("string")));
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("effects"), "{err}");
    }

    #[test]
    fn valid_raw_effects_object_is_accepted() {
        let ench = valid().raw_effects(RawJson::new(serde_json::json!({"key": []})));
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn missing_description_is_rejected() {
        let ench = Enchantment::new(rl())
            .supported_items(TagId::<ItemId>::minecraft("enchantable/sword").unwrap())
            .slot(EquipmentSlotGroup::Mainhand);
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("description"), "{err}");
    }

    #[test]
    fn invalid_typed_description_fails_export() {
        let ench = valid().description(TextComponent::translate(" \t ").color_hex("#12FG00"));
        let error = ench.try_content().unwrap_err().to_string();
        assert!(error.contains("description"), "{error}");
    }

    #[test]
    fn raw_description_is_preserved() {
        let ench = valid().raw_description(RawJson::new(serde_json::json!({"mymod:text": true})));
        assert!(ench.validate().is_ok());
        assert_eq!(ench.to_json()["description"]["mymod:text"], true);
    }

    #[test]
    fn valid_enchantment_json_is_stable() {
        let ench = valid();
        let json = ench.to_json();
        assert_eq!(json["supported_items"], "#minecraft:enchantable/foot_armor");
        assert_eq!(json["weight"], 10);
        assert_eq!(json["max_level"], 1);
        assert_eq!(json["slots"][0], "feet");
    }

    #[test]
    fn invalid_enchantment_fails_export() {
        let ench = Enchantment::new(rl());
        assert!(ench.try_content().is_err());
    }

    // ── Typed effect slice ─────────────────────────────────────────────────

    #[test]
    fn knockback_effect_matches_vanilla_shape() {
        let ench = valid().knockback_effect(
            EnchantmentValueOperation::Add,
            LevelBasedValue::Linear {
                base: 1.0,
                per_level_above_first: 1.0,
            },
        );
        assert!(ench.validate().is_ok());
        let json = ench.to_json();
        assert_eq!(
            json["effects"]["minecraft:knockback"],
            serde_json::json!([{
                "effect": {
                    "type": "minecraft:add",
                    "value": {
                        "type": "minecraft:linear",
                        "base": 1.0,
                        "per_level_above_first": 1.0,
                    }
                }
            }])
        );
    }

    #[test]
    fn damage_effect_matches_vanilla_sharpness_shape() {
        let ench = valid().damage_effect(
            EnchantmentValueOperation::Add,
            LevelBasedValue::Linear {
                base: 1.0,
                per_level_above_first: 0.5,
            },
        );
        let json = ench.to_json();
        assert_eq!(
            json["effects"]["minecraft:damage"],
            serde_json::json!([{
                "effect": {
                    "type": "minecraft:add",
                    "value": {
                        "type": "minecraft:linear",
                        "base": 1.0,
                        "per_level_above_first": 0.5,
                    }
                }
            }])
        );
    }

    #[test]
    fn armor_effectiveness_effect_matches_vanilla_breach_shape() {
        let ench = valid().armor_effectiveness_effect(
            EnchantmentValueOperation::Add,
            LevelBasedValue::Linear {
                base: -0.15,
                per_level_above_first: -0.15,
            },
        );
        let json = ench.to_json();
        assert_eq!(
            json["effects"]["minecraft:armor_effectiveness"],
            serde_json::json!([{
                "effect": {
                    "type": "minecraft:add",
                    "value": {
                        "type": "minecraft:linear",
                        "base": -0.15,
                        "per_level_above_first": -0.15,
                    }
                }
            }])
        );
    }

    #[test]
    fn constant_level_based_value_serializes_as_bare_number() {
        let ench = valid().knockback_effect(
            EnchantmentValueOperation::Set,
            LevelBasedValue::Constant(1.0),
        );
        let json = ench.to_json();
        assert_eq!(
            json["effects"]["minecraft:knockback"][0]["effect"]["value"],
            serde_json::json!(1.0)
        );
    }

    #[test]
    fn non_finite_level_based_value_is_rejected() {
        let ench = valid().knockback_effect(
            EnchantmentValueOperation::Add,
            LevelBasedValue::Constant(f64::NAN),
        );
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("effects"), "{err}");
    }

    #[test]
    fn typed_and_raw_effect_components_can_be_combined() {
        let ench = valid()
            .knockback_effect(
                EnchantmentValueOperation::Add,
                LevelBasedValue::Constant(1.0),
            )
            .raw_effect_component(
                EnchantmentEffectComponentId::custom(
                    ResourceLocation::new("mymod", "custom_effect").unwrap(),
                ),
                RawJson::new(serde_json::json!({"value": 5})),
            );
        assert!(ench.validate().is_ok());
        let json = ench.to_json();
        assert_eq!(
            json["effects"]["minecraft:knockback"][0]["effect"]["type"],
            "minecraft:add"
        );
        assert_eq!(json["effects"]["mymod:custom_effect"][0]["value"], 5);
    }

    #[test]
    fn raw_effects_override_replaces_typed_and_raw_component_entries() {
        let ench = valid()
            .knockback_effect(
                EnchantmentValueOperation::Add,
                LevelBasedValue::Constant(1.0),
            )
            .raw_effects(RawJson::new(serde_json::json!({"minecraft:custom": []})));
        let json = ench.to_json();
        assert_eq!(json["effects"], serde_json::json!({"minecraft:custom": []}));
    }

    #[test]
    fn custom_effect_component_id_allows_namespaced_ids() {
        let id = EnchantmentEffectComponentId::custom(
            ResourceLocation::new("mymod", "custom_effect").unwrap(),
        );
        assert_eq!(id.to_string(), "mymod:custom_effect");
    }

    #[test]
    fn well_known_effect_component_ids_are_minecraft_namespaced() {
        assert_eq!(
            EnchantmentEffectComponentId::damage().to_string(),
            "minecraft:damage"
        );
        assert_eq!(
            EnchantmentEffectComponentId::knockback().to_string(),
            "minecraft:knockback"
        );
        assert_eq!(
            EnchantmentEffectComponentId::armor_effectiveness().to_string(),
            "minecraft:armor_effectiveness"
        );
    }
}
