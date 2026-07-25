//! Builders for armor trim material and pattern definitions (Minecraft 1.20+).
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//!
//! `TrimMaterial`:
//! - `asset_name` must be non-empty and a valid resource-location-shaped
//!   identifier (e.g. `"quartz"`).
//! - `ingredient` must be non-empty and a valid plain resource location
//!   (e.g. `"minecraft:quartz"`).
//! - `item_model_index` must be finite (non-`NaN`, non-infinite) — required
//!   for the value to serialize as valid JSON. Sand does **not** enforce a
//!   `0.0..=1.0` numeric range here: `item_model_index` is a pre-1.21.4
//!   legacy field (superseded by `override_armor_assets` in current
//!   Minecraft; Sand does not yet model that field, see the crate's known
//!   limitations) with no vanilla-documented bound, and real third-party
//!   tooling uses out-of-`0..1` values (e.g. `-1.0` as a sentinel) for valid
//!   purposes. A `0.0..=1.0` range was previously enforced here as an
//!   unverified opinionated guess and has been relaxed after failing to find
//!   vanilla evidence for it.
//!
//! `TrimPattern`:
//! - `asset_id` must be non-empty and a valid plain resource location.
//! - `template_item` must be non-empty and a valid plain resource location.
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

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;
use crate::validation;

// ── TrimMaterial ──────────────────────────────────────────────────────────────

/// A trim material definition (`data/<namespace>/trim_material/<id>.json`).
pub struct TrimMaterial {
    location: ResourceLocation,
    /// Asset name used to locate the trim material texture (e.g. `"quartz"`).
    asset_name: String,
    /// Item used to apply this trim (e.g. `"minecraft:quartz"`).
    ingredient: String,
    /// Model index for the trim overlay. Must be finite; Minecraft does not
    /// document a numeric range for this legacy (pre-1.21.4) field.
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

    fn validate(&self) -> SandResult<()> {
        let kind = "trim_material";
        validation::require_non_empty(&self.location, kind, "asset_name", &self.asset_name)?;
        validation::validate_resource_location_str(
            &self.location,
            kind,
            "asset_name",
            &self.asset_name,
        )?;
        validation::require_non_empty(&self.location, kind, "ingredient", &self.ingredient)?;
        validation::validate_resource_location_str(
            &self.location,
            kind,
            "ingredient",
            &self.ingredient,
        )?;
        validation::require_finite_f32(
            &self.location,
            kind,
            "item_model_index",
            self.item_model_index,
        )?;
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
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

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::TrimAssets]
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

    fn validate(&self) -> SandResult<()> {
        let kind = "trim_pattern";
        validation::require_non_empty(&self.location, kind, "asset_id", &self.asset_id)?;
        validation::validate_resource_location_str(
            &self.location,
            kind,
            "asset_id",
            &self.asset_id,
        )?;
        validation::require_non_empty(&self.location, kind, "template_item", &self.template_item)?;
        validation::validate_resource_location_str(
            &self.location,
            kind,
            "template_item",
            &self.template_item,
        )?;
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
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

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::TrimAssets]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "quartz").unwrap()
    }

    fn valid_material() -> TrimMaterial {
        TrimMaterial::new(rl())
            .asset_name("quartz")
            .ingredient("minecraft:quartz")
            .item_model_index(0.1)
    }

    fn valid_pattern() -> TrimPattern {
        TrimPattern::new(rl())
            .asset_id("minecraft:bolt")
            .template_item("minecraft:bolt_armor_trim_smithing_template")
    }

    // ── TrimMaterial ────────────────────────────────────────────────────────

    #[test]
    fn valid_trim_material_passes_validation() {
        assert!(valid_material().validate().is_ok());
    }

    #[test]
    fn empty_asset_name_is_rejected() {
        let m = TrimMaterial::new(rl()).ingredient("minecraft:quartz");
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("asset_name"), "{err}");
    }

    #[test]
    fn empty_ingredient_is_rejected() {
        let m = TrimMaterial::new(rl()).asset_name("quartz");
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("ingredient"), "{err}");
    }

    #[test]
    fn malformed_ingredient_is_rejected() {
        let m = valid_material().ingredient("Not Valid!");
        assert!(m.validate().is_err());
    }

    #[test]
    fn nan_item_model_index_is_rejected() {
        let m = valid_material().item_model_index(f32::NAN);
        assert!(m.validate().is_err());
    }

    #[test]
    fn infinite_item_model_index_is_rejected() {
        let m = valid_material().item_model_index(f32::INFINITY);
        assert!(m.validate().is_err());
    }

    /// `item_model_index` has no vanilla-documented numeric range (see the
    /// module-level `# Validation` docs) — negative and >1.0 finite values
    /// are legitimate and must be accepted, matching real third-party usage
    /// (e.g. `-1.0` as a sentinel).
    #[test]
    fn item_model_index_out_of_unit_range_is_accepted() {
        assert!(valid_material().item_model_index(-1.0).validate().is_ok());
        assert!(valid_material().item_model_index(2.5).validate().is_ok());
    }

    #[test]
    fn item_model_index_bounds_are_accepted() {
        assert!(valid_material().item_model_index(0.0).validate().is_ok());
        assert!(valid_material().item_model_index(1.0).validate().is_ok());
    }

    #[test]
    fn invalid_trim_material_fails_export() {
        let m = TrimMaterial::new(rl());
        assert!(m.try_content().is_err());
    }

    #[test]
    fn valid_trim_material_json_is_stable() {
        let m = valid_material();
        let json = m.to_json();
        assert_eq!(json["asset_name"], "quartz");
        assert_eq!(json["ingredient"], "minecraft:quartz");
        assert_eq!(json["item_model_index"], serde_json::json!(0.1_f32));
        let a = serde_json::to_string_pretty(&m.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&m.to_json()).unwrap();
        assert_eq!(a, b);
    }

    // ── TrimPattern ─────────────────────────────────────────────────────────

    #[test]
    fn valid_trim_pattern_passes_validation() {
        assert!(valid_pattern().validate().is_ok());
    }

    #[test]
    fn empty_asset_id_is_rejected() {
        let p = TrimPattern::new(rl()).template_item("minecraft:bolt_armor_trim_smithing_template");
        let err = p.validate().unwrap_err();
        assert!(err.to_string().contains("asset_id"), "{err}");
    }

    #[test]
    fn empty_template_item_is_rejected() {
        let p = TrimPattern::new(rl()).asset_id("minecraft:bolt");
        let err = p.validate().unwrap_err();
        assert!(err.to_string().contains("template_item"), "{err}");
    }

    #[test]
    fn malformed_template_item_is_rejected() {
        let p = valid_pattern().template_item("Not Valid!");
        assert!(p.validate().is_err());
    }

    #[test]
    fn invalid_trim_pattern_fails_export() {
        let p = TrimPattern::new(rl());
        assert!(p.try_content().is_err());
    }

    #[test]
    fn valid_trim_pattern_json_is_stable() {
        let p = valid_pattern().decal(true);
        let json = p.to_json();
        assert_eq!(json["asset_id"], "minecraft:bolt");
        assert_eq!(
            json["template_item"],
            "minecraft:bolt_armor_trim_smithing_template"
        );
        assert_eq!(json["decal"], true);
        let a = serde_json::to_string_pretty(&p.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&p.to_json()).unwrap();
        assert_eq!(a, b);
    }
}
