//! Builder for `data/<namespace>/enchantment/` JSON files (Minecraft 1.21+).
//!
//! Enchantment definitions control how enchantments are applied, their effects,
//! costs, and which items they can appear on.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

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

// ── Enchantment ───────────────────────────────────────────────────────────────

/// An enchantment definition (`data/<namespace>/enchantment/<id>.json`).
pub struct Enchantment {
    location: ResourceLocation,
    /// Human-readable description (text component as raw JSON).
    description: Value,
    /// Supported items tag or list (e.g. `"#minecraft:sword_enchantable"`).
    supported_items: String,
    /// Primary items tag or list — items shown in the enchanting table.
    primary_items: Option<String>,
    /// Exclusivity tag — enchantments in the same tag cannot coexist.
    exclusive_set: Option<String>,
    /// Weight determining how often this enchantment appears (1–1024).
    weight: u32,
    /// Maximum enchantment level (1–255).
    max_level: u32,
    /// Minimum enchanting-table cost per level.
    min_cost: EnchantmentCost,
    /// Maximum enchanting-table cost per level.
    max_cost: EnchantmentCost,
    /// Anvil cost (XP levels consumed when combining/applying).
    anvil_cost: u32,
    /// Equipment slots this enchantment is active in.
    slots: Vec<String>,
    /// Raw effects map as a JSON object (complex per-1.21 format).
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

    /// Adds an equipment slot this enchantment is active in (e.g. `"mainhand"`, `"armor"`).
    pub fn slot(mut self, slot: impl Into<String>) -> Self {
        self.slots.push(slot.into());
        self
    }

    /// Sets all active equipment slots at once.
    pub fn slots(mut self, slots: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.slots = slots.into_iter().map(|s| s.into()).collect();
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
}
