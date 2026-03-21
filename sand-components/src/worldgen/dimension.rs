//! Dimension builder for `data/<namespace>/dimension/<id>.json`.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

/// A dimension definition (`data/<namespace>/dimension/<id>.json`).
///
/// Dimensions reference a dimension type and a chunk generator. The chunk
/// generator config is complex so it is accepted as raw JSON. Use
/// [`Dimension::generator_raw`] to supply it directly.
pub struct Dimension {
    location: ResourceLocation,
    /// The dimension type ID (e.g. `"minecraft:overworld"`, `"minecraft:the_nether"`).
    dimension_type: String,
    /// The chunk generator configuration as raw JSON.
    generator: Value,
}

impl Dimension {
    /// Creates a new dimension referencing the given type and generator JSON.
    pub fn new(
        location: ResourceLocation,
        dimension_type: impl Into<String>,
        generator: Value,
    ) -> Self {
        Self {
            location,
            dimension_type: dimension_type.into(),
            generator,
        }
    }

    /// Convenience: create with a noise-based generator pointing to a noise_settings ID.
    ///
    /// `biome_source` should be a raw JSON biome source object, e.g.:
    /// ```json
    /// { "type": "minecraft:fixed", "biome": "minecraft:plains" }
    /// ```
    pub fn noise_generator(
        location: ResourceLocation,
        dimension_type: impl Into<String>,
        noise_settings: impl Into<String>,
        biome_source: Value,
    ) -> Self {
        let generator = serde_json::json!({
            "type": "minecraft:noise",
            "settings": noise_settings.into(),
            "biome_source": biome_source,
        });
        Self::new(location, dimension_type, generator)
    }

    /// Convenience: create with a flat (superflat) generator.
    ///
    /// `flat_settings` is the raw JSON settings for `minecraft:flat`.
    pub fn flat_generator(
        location: ResourceLocation,
        dimension_type: impl Into<String>,
        flat_settings: Value,
    ) -> Self {
        let generator = serde_json::json!({
            "type": "minecraft:flat",
            "settings": flat_settings,
        });
        Self::new(location, dimension_type, generator)
    }

    /// Updates the dimension type.
    pub fn dimension_type(mut self, dt: impl Into<String>) -> Self {
        self.dimension_type = dt.into();
        self
    }

    /// Replaces the generator with a raw JSON value.
    pub fn generator_raw(mut self, generator: Value) -> Self {
        self.generator = generator;
        self
    }
}

impl DatapackComponent for Dimension {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "type": self.dimension_type,
            "generator": self.generator,
        })
    }

    fn component_dir(&self) -> &'static str {
        "dimension"
    }
}
