//! Builders for armor trim material and pattern definitions (Minecraft 1.20+).
//!
//! # Example
//! ```rust,ignore
//! let material = TrimMaterial::new(rl)
//!     .asset_name("quartz")
//!     .ingredient("minecraft:quartz")
//!     .item_model_index(0.1)
//!     .description(serde_json::json!({"translate": "trim_material.minecraft.quartz"}));
//!
//! let pattern = TrimPattern::new(rl)
//!     .asset_id("minecraft:bolt")
//!     .template_item("minecraft:bolt_armor_trim_smithing_template")
//!     .description(serde_json::json!({"translate": "trim_pattern.minecraft.bolt"}));
//! ```

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

// ── TrimMaterial ──────────────────────────────────────────────────────────────

/// A trim material definition (`data/<namespace>/trim_material/<id>.json`).
pub struct TrimMaterial {
    location: ResourceLocation,
    /// Asset name used to locate the trim material texture (e.g. `"quartz"`).
    asset_name: String,
    /// Item used to apply this trim (e.g. `"minecraft:quartz"`).
    ingredient: String,
    /// Model index for the trim overlay (0.0–1.0).
    item_model_index: f32,
    /// Text component for the trim tooltip description.
    description: Option<Value>,
    /// Per-armor-material overrides for the texture asset name.
    /// Keys are armor material IDs (e.g. `"minecraft:iron"`).
    override_armor_materials: Option<Value>,
}

impl TrimMaterial {
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            asset_name: String::new(),
            ingredient: String::new(),
            item_model_index: 0.0,
            description: None,
            override_armor_materials: None,
        }
    }

    pub fn asset_name(mut self, name: impl Into<String>) -> Self {
        self.asset_name = name.into();
        self
    }

    pub fn ingredient(mut self, item: impl Into<String>) -> Self {
        self.ingredient = item.into();
        self
    }

    pub fn item_model_index(mut self, index: f32) -> Self {
        self.item_model_index = index;
        self
    }

    pub fn description(mut self, desc: Value) -> Self {
        self.description = Some(desc);
        self
    }

    /// Override the asset name for specific armor materials.
    pub fn override_armor_materials(mut self, overrides: Value) -> Self {
        self.override_armor_materials = Some(overrides);
        self
    }
}

impl DatapackComponent for TrimMaterial {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "asset_name".to_string(),
            Value::String(self.asset_name.clone()),
        );
        map.insert(
            "ingredient".to_string(),
            Value::String(self.ingredient.clone()),
        );
        map.insert(
            "item_model_index".to_string(),
            serde_json::json!(self.item_model_index),
        );
        if let Some(ref desc) = self.description {
            map.insert("description".to_string(), desc.clone());
        }
        if let Some(ref overrides) = self.override_armor_materials {
            map.insert("override_armor_materials".to_string(), overrides.clone());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "trim_material"
    }
}

// ── TrimPattern ───────────────────────────────────────────────────────────────

/// A trim pattern definition (`data/<namespace>/trim_pattern/<id>.json`).
pub struct TrimPattern {
    location: ResourceLocation,
    /// Resource location of the pattern texture (e.g. `"minecraft:bolt"`).
    asset_id: String,
    /// Item that applies this pattern at a smithing table.
    template_item: String,
    /// Text component for the pattern tooltip.
    description: Option<Value>,
    /// Whether this pattern is rendered as a decal overlay.
    decal: bool,
}

impl TrimPattern {
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            asset_id: String::new(),
            template_item: String::new(),
            description: None,
            decal: false,
        }
    }

    pub fn asset_id(mut self, id: impl Into<String>) -> Self {
        self.asset_id = id.into();
        self
    }

    pub fn template_item(mut self, item: impl Into<String>) -> Self {
        self.template_item = item.into();
        self
    }

    pub fn description(mut self, desc: Value) -> Self {
        self.description = Some(desc);
        self
    }

    pub fn decal(mut self, v: bool) -> Self {
        self.decal = v;
        self
    }
}

impl DatapackComponent for TrimPattern {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("asset_id".to_string(), Value::String(self.asset_id.clone()));
        map.insert(
            "template_item".to_string(),
            Value::String(self.template_item.clone()),
        );
        if let Some(ref desc) = self.description {
            map.insert("description".to_string(), desc.clone());
        }
        map.insert("decal".to_string(), Value::Bool(self.decal));
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "trim_pattern"
    }
}
