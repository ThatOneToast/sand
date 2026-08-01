//! Dimension builder for `data/<namespace>/dimension/<id>.json`.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::registry::DimensionTypeId;
use crate::resource_location::ResourceLocation;
use crate::validation;

const KIND: &str = "dimension";

#[derive(Debug, Clone)]
enum DimensionTypeReference {
    Typed(DimensionTypeId),
    Raw(String),
}

impl DimensionTypeReference {
    fn as_string(&self) -> String {
        match self {
            Self::Typed(id) => id.to_string(),
            Self::Raw(id) => id.clone(),
        }
    }
}

/// A dimension definition (`data/<namespace>/dimension/<id>.json`).
///
/// Dimensions reference a dimension type and a chunk generator. The chunk
/// generator config is complex so it is accepted as raw JSON. Use
/// [`Dimension::generator_raw`] to supply it directly.
pub struct Dimension {
    location: ResourceLocation,
    /// The dimension type ID (e.g. `"minecraft:overworld"`, `"minecraft:the_nether"`).
    dimension_type: DimensionTypeReference,
    /// The chunk generator configuration as raw JSON.
    generator: Value,
}

impl Dimension {
    /// Creates a new dimension referencing the given type and generator JSON.
    pub fn new(
        location: ResourceLocation,
        dimension_type: DimensionTypeId,
        generator: Value,
    ) -> Self {
        Self {
            location,
            dimension_type: DimensionTypeReference::Typed(dimension_type),
            generator,
        }
    }

    /// Creates a dimension with an explicitly raw dimension-type reference.
    ///
    /// Prefer [`Dimension::new`] with [`DimensionTypeId`]. This escape hatch
    /// exists for version-specific or otherwise unsupported reference syntax.
    pub fn new_raw_dimension_type(
        location: ResourceLocation,
        dimension_type: impl Into<String>,
        generator: Value,
    ) -> Self {
        Self {
            location,
            dimension_type: DimensionTypeReference::Raw(dimension_type.into()),
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
        dimension_type: DimensionTypeId,
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
        dimension_type: DimensionTypeId,
        flat_settings: Value,
    ) -> Self {
        let generator = serde_json::json!({
            "type": "minecraft:flat",
            "settings": flat_settings,
        });
        Self::new(location, dimension_type, generator)
    }

    /// Updates the dimension type.
    pub fn dimension_type(mut self, dt: DimensionTypeId) -> Self {
        self.dimension_type = DimensionTypeReference::Typed(dt);
        self
    }

    /// Updates the dimension type through the explicit raw compatibility path.
    pub fn raw_dimension_type(mut self, dt: impl Into<String>) -> Self {
        self.dimension_type = DimensionTypeReference::Raw(dt.into());
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
            "type": self.dimension_type.as_string(),
            "generator": self.generator,
        })
    }

    fn validate(&self) -> SandResult<()> {
        if let DimensionTypeReference::Raw(dt) = &self.dimension_type {
            validation::validate_resource_location_str(&self.location, KIND, "type", dt)?;
        }

        validation::require_json_object(&self.location, KIND, "generator", &self.generator)?;
        let generator_type = self.generator.get("type").and_then(Value::as_str);
        match generator_type {
            Some(ty) if !ty.trim().is_empty() => {
                validation::validate_resource_location_str(
                    &self.location,
                    KIND,
                    "generator.type",
                    ty,
                )?;
            }
            _ => {
                return Err(validation::error(
                    &self.location,
                    KIND,
                    "generator.type",
                    "generator must be a JSON object with a non-empty string `type` field",
                ));
            }
        }

        if generator_type == Some("minecraft:noise")
            && let Some(settings) = self.generator.get("settings")
        {
            if let Some(settings_str) = settings.as_str() {
                validation::validate_resource_location_str(
                    &self.location,
                    KIND,
                    "generator.settings",
                    settings_str,
                )?;
            } else {
                return Err(validation::error(
                    &self.location,
                    KIND,
                    "generator.settings",
                    "generator.settings must be a resource location string for a noise generator",
                ));
            }
        }

        Ok(())
    }

    fn component_dir(&self) -> &'static str {
        "dimension"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "skylands").unwrap()
    }

    #[test]
    fn constructors_accept_typed_dimension_type_ids() {
        let id = DimensionTypeId::minecraft("overworld").unwrap();
        let dimension = Dimension::new(location(), id, serde_json::json!({"type": "test"}));
        assert_eq!(dimension.to_json()["type"], "minecraft:overworld");

        let noise = Dimension::noise_generator(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            "minecraft:overworld",
            serde_json::json!({"type": "minecraft:fixed", "biome": "minecraft:plains"}),
        );
        assert_eq!(noise.to_json()["type"], "minecraft:overworld");
    }

    #[test]
    fn raw_dimension_type_reference_is_explicit_and_preserved() {
        let dimension = Dimension::new_raw_dimension_type(
            location(),
            "modded reference",
            serde_json::json!({"type": "test"}),
        )
        .raw_dimension_type("modded:custom");
        assert_eq!(dimension.to_json()["type"], "modded:custom");
    }

    #[test]
    fn valid_dimension_passes_validation() {
        let dimension = Dimension::new(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            serde_json::json!({"type": "minecraft:noise", "settings": "minecraft:overworld"}),
        );
        assert!(dimension.validate().is_ok());
    }

    #[test]
    fn malformed_raw_dimension_type_rejected() {
        let dimension = Dimension::new_raw_dimension_type(
            location(),
            "modded reference",
            serde_json::json!({"type": "test"}),
        );
        let err = dimension.validate().unwrap_err().to_string();
        assert!(err.contains("type"), "{err}");
    }

    #[test]
    fn empty_raw_dimension_type_rejected() {
        let dimension =
            Dimension::new_raw_dimension_type(location(), "", serde_json::json!({"type": "test"}));
        assert!(dimension.validate().is_err());
    }

    #[test]
    fn generator_raw_non_object_rejected() {
        let dimension = Dimension::new(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            serde_json::json!({"type": "test"}),
        )
        .generator_raw(serde_json::json!(["not", "an", "object"]));
        let err = dimension.validate().unwrap_err().to_string();
        assert!(err.contains("generator"), "{err}");
    }

    #[test]
    fn generator_raw_missing_type_rejected() {
        let dimension = Dimension::new(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            serde_json::json!({"type": "test"}),
        )
        .generator_raw(serde_json::json!({"settings": "minecraft:overworld"}));
        let err = dimension.validate().unwrap_err().to_string();
        assert!(err.contains("generator.type"), "{err}");
    }

    #[test]
    fn noise_generator_malformed_settings_id_rejected() {
        let dimension = Dimension::noise_generator(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            "Not A Valid Id",
            serde_json::json!({"type": "minecraft:fixed", "biome": "minecraft:plains"}),
        );
        let err = dimension.validate().unwrap_err().to_string();
        assert!(err.contains("generator.settings"), "{err}");
    }

    #[test]
    fn flat_generator_escape_hatch_still_works() {
        let dimension = Dimension::flat_generator(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            serde_json::json!({"layers": [{"block": "minecraft:bedrock", "height": 1}]}),
        );
        assert!(dimension.validate().is_ok());
        assert_eq!(dimension.to_json()["generator"]["type"], "minecraft:flat");
    }
}
