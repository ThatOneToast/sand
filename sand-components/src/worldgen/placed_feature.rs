//! Placed feature builder for `data/<namespace>/worldgen/placed_feature/<id>.json`.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

/// A placed feature definition (`data/<namespace>/worldgen/placed_feature/<id>.json`).
///
/// Placed features reference a configured feature and a list of placement
/// modifiers that determine where and how often they generate in the world.
pub struct PlacedFeature {
    location: ResourceLocation,
    /// The ID of the configured feature to place (e.g. `"minecraft:oak"`).
    feature: String,
    /// Placement modifier entries as raw JSON objects.
    placement: Vec<Value>,
}

impl PlacedFeature {
    /// Creates a new placed feature with the given resource location and feature ID.
    pub fn new(location: ResourceLocation, feature: impl Into<String>) -> Self {
        Self {
            location,
            feature: feature.into(),
            placement: Vec::new(),
        }
    }

    /// Adds a placement modifier as a raw JSON object.
    ///
    /// # Example
    /// ```rust,ignore
    /// use serde_json::json;
    /// feature.placement_modifier(json!({ "type": "minecraft:count", "count": 5 }));
    /// ```
    pub fn placement_modifier(mut self, modifier: Value) -> Self {
        self.placement.push(modifier);
        self
    }

    /// Sets all placement modifiers at once from an iterator of raw JSON values.
    pub fn placement(mut self, modifiers: impl IntoIterator<Item = Value>) -> Self {
        self.placement = modifiers.into_iter().collect();
        self
    }
}

impl DatapackComponent for PlacedFeature {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "feature": self.feature,
            "placement": self.placement,
        })
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/placed_feature"
    }
}
