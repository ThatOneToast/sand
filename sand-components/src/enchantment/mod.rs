//! Builder for `data/<namespace>/enchantment/` JSON files (Minecraft 1.21+).
//!
//! Enchantment definitions control how enchantments are applied, their effects,
//! costs, and which items they can appear on.
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `supported_items` must be non-empty and a valid resource location or tag
//!   reference (`#namespace:path`).
//! - `slots` must contain at least one entry; each must be a valid
//!   [`EnchantmentSlot`] name.
//! - `weight` must be in `1..=1024`.
//! - `max_level` must be in `1..=255`.
//! - `description` must be a non-null JSON value.
//! - `primary_items` and `exclusive_set`, when present, must be valid resource
//!   location or tag references.
//! - `effects`, when present, must be a JSON object.

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
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

// ── EnchantmentEffect ─────────────────────────────────────────────────────────

/// A single enchantment effect entry (simplified representation).
///
/// In Minecraft 1.21, enchantment effects are complex typed components.
/// This builder stores them as raw JSON values so users can provide
/// any valid effect definition.
#[derive(Clone)]
pub struct EnchantmentEffect {
    /// The effect type identifier (e.g. `"minecraft:damage"`, `"minecraft:knockback"`).
    pub effect_type: String,
    /// Raw effect configuration as a JSON object.
    pub config: Value,
}

impl EnchantmentEffect {
    /// Creates a new effect entry with the given type and raw JSON config.
    pub fn new(effect_type: impl Into<String>, config: Value) -> Self {
        Self {
            effect_type: effect_type.into(),
            config,
        }
    }
}

// ── EnchantmentSlot ───────────────────────────────────────────────────────────

/// Equipment slots where an enchantment is active (Minecraft Java 26.2).
///
/// Each variant maps to a lowercase slot-group name accepted by the
/// enchantment JSON schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnchantmentSlot {
    Any,
    Mainhand,
    Offhand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
}

impl EnchantmentSlot {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Mainhand => "mainhand",
            Self::Offhand => "offhand",
            Self::Hand => "hand",
            Self::Feet => "feet",
            Self::Legs => "legs",
            Self::Chest => "chest",
            Self::Head => "head",
            Self::Armor => "armor",
            Self::Body => "body",
        }
    }

    fn from_name(s: &str) -> Option<Self> {
        match s {
            "any" => Some(Self::Any),
            "mainhand" => Some(Self::Mainhand),
            "offhand" => Some(Self::Offhand),
            "hand" => Some(Self::Hand),
            "feet" => Some(Self::Feet),
            "legs" => Some(Self::Legs),
            "chest" => Some(Self::Chest),
            "head" => Some(Self::Head),
            "armor" => Some(Self::Armor),
            "body" => Some(Self::Body),
            _ => None,
        }
    }
}

impl std::fmt::Display for EnchantmentSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Enchantment ───────────────────────────────────────────────────────────────

/// An enchantment definition (`data/<namespace>/enchantment/<id>.json`).
pub struct Enchantment {
    location: ResourceLocation,
    description: Value,
    supported_items: String,
    primary_items: Option<String>,
    exclusive_set: Option<String>,
    weight: u32,
    max_level: u32,
    min_cost: EnchantmentCost,
    max_cost: EnchantmentCost,
    anvil_cost: u32,
    slots: Vec<String>,
    effects: Option<Value>,
}

impl Enchantment {
    /// Creates a new enchantment with sensible defaults.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            description: Value::String(String::new()),
            supported_items: String::new(),
            primary_items: None,
            exclusive_set: None,
            weight: 10,
            max_level: 1,
            min_cost: EnchantmentCost::new(1, 11),
            max_cost: EnchantmentCost::new(21, 11),
            anvil_cost: 2,
            slots: Vec::new(),
            effects: None,
        }
    }

    /// Sets the description as a raw JSON text component.
    pub fn description(mut self, desc: Value) -> Self {
        self.description = desc;
        self
    }

    /// Convenience: sets the description as a plain translation key string.
    pub fn description_translate(mut self, key: impl Into<String>) -> Self {
        self.description = serde_json::json!({ "translate": key.into() });
        self
    }

    /// Sets the supported items tag/list (e.g. `"#minecraft:sword_enchantable"`).
    pub fn supported_items(mut self, items: impl Into<String>) -> Self {
        self.supported_items = items.into();
        self
    }

    /// Sets the primary items (shown in enchanting table).
    pub fn primary_items(mut self, items: impl Into<String>) -> Self {
        self.primary_items = Some(items.into());
        self
    }

    /// Sets the exclusive set tag — enchantments in this tag are mutually exclusive.
    pub fn exclusive_set(mut self, tag: impl Into<String>) -> Self {
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

    /// Adds an equipment slot this enchantment is active in (raw string).
    pub fn slot(mut self, slot: impl Into<String>) -> Self {
        self.slots.push(slot.into());
        self
    }

    /// Sets all active equipment slots at once (raw strings).
    pub fn slots(mut self, slots: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.slots = slots.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Adds a typed equipment slot this enchantment is active in.
    pub fn slot_typed(mut self, slot: EnchantmentSlot) -> Self {
        self.slots.push(slot.as_str().to_string());
        self
    }

    /// Sets all active equipment slots from typed values.
    pub fn slots_typed(mut self, slots: impl IntoIterator<Item = EnchantmentSlot>) -> Self {
        self.slots = slots.into_iter().map(|s| s.as_str().to_string()).collect();
        self
    }

    /// Sets the effects map as a raw JSON object (Minecraft 1.21 component format).
    pub fn effects_raw(mut self, effects: Value) -> Self {
        self.effects = Some(effects);
        self
    }
}

impl DatapackComponent for Enchantment {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "enchantment";

        validation::require_non_empty(
            &self.location,
            kind,
            "supported_items",
            &self.supported_items,
        )?;
        validation::validate_resource_location_str(
            &self.location,
            kind,
            "supported_items",
            &self.supported_items,
        )?;

        validation::require_non_empty_collection(&self.location, kind, "slots", self.slots.len())?;
        for (i, slot) in self.slots.iter().enumerate() {
            if EnchantmentSlot::from_name(slot).is_none() {
                return Err(validation::error(
                    &self.location,
                    kind,
                    &format!("slots[{i}]"),
                    &format!(
                        "`{slot}` is not a valid enchantment slot; \
                         expected one of: any, mainhand, offhand, hand, \
                         feet, legs, chest, head, armor, body"
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

        if self.description == Value::Null {
            return Err(validation::error(
                &self.location,
                kind,
                "description",
                "must be a non-null JSON text component",
            ));
        }

        if let Some(ref pi) = self.primary_items {
            validation::validate_resource_location_str(&self.location, kind, "primary_items", pi)?;
        }

        if let Some(ref ex) = self.exclusive_set {
            validation::validate_resource_location_str(&self.location, kind, "exclusive_set", ex)?;
        }

        if let Some(ref effects) = self.effects {
            validation::require_json_object(&self.location, kind, "effects", effects)?;
        }

        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("description".to_string(), self.description.clone());
        map.insert(
            "supported_items".to_string(),
            Value::String(self.supported_items.clone()),
        );
        if let Some(ref pi) = self.primary_items {
            map.insert("primary_items".to_string(), Value::String(pi.clone()));
        }
        if let Some(ref ex) = self.exclusive_set {
            map.insert("exclusive_set".to_string(), Value::String(ex.clone()));
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
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
        if let Some(ref effects) = self.effects {
            map.insert("effects".to_string(), effects.clone());
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
            .description(serde_json::json!("Swift Step"))
            .supported_items("#minecraft:enchantable/foot_armor")
            .slot_typed(EnchantmentSlot::Feet)
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
            .description(serde_json::json!("x"))
            .slot_typed(EnchantmentSlot::Any);
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("supported_items"), "{err}");
    }

    #[test]
    fn empty_slots_is_rejected() {
        let ench = Enchantment::new(rl())
            .description(serde_json::json!("x"))
            .supported_items("#minecraft:enchantable/sword");
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("slots"), "{err}");
    }

    #[test]
    fn invalid_slot_name_is_rejected() {
        let ench = valid().slot("invalid_slot");
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
    fn invalid_supported_items_resource_is_rejected() {
        let ench = valid().supported_items("INVALID");
        assert!(ench.validate().is_err());
    }

    #[test]
    fn invalid_primary_items_resource_is_rejected() {
        let ench = valid().primary_items("bad resource");
        assert!(ench.validate().is_err());
    }

    #[test]
    fn invalid_exclusive_set_resource_is_rejected() {
        let ench = valid().exclusive_set("bad resource");
        assert!(ench.validate().is_err());
    }

    #[test]
    fn non_object_effects_is_rejected() {
        let ench = valid().effects_raw(serde_json::json!("string"));
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("effects"), "{err}");
    }

    #[test]
    fn valid_raw_effects_object_is_accepted() {
        let ench = valid().effects_raw(serde_json::json!({"key": {}}));
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn null_description_is_rejected() {
        let ench = Enchantment::new(rl())
            .description(Value::Null)
            .supported_items("#minecraft:enchantable/sword")
            .slot_typed(EnchantmentSlot::Mainhand);
        let err = ench.validate().unwrap_err();
        assert!(err.to_string().contains("description"), "{err}");
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

    #[test]
    fn valid_primary_items_tag_is_accepted() {
        let ench = valid().primary_items("#minecraft:enchantable/sword");
        assert!(ench.validate().is_ok());
    }

    #[test]
    fn valid_exclusive_set_namespaced_is_accepted() {
        let ench = valid().exclusive_set("minecraft:damage");
        assert!(ench.validate().is_ok());
    }
}
