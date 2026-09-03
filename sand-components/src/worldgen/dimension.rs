//! Dimension builder for `data/<namespace>/dimension/<id>.json`.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Dimension",
    aliases = ["sand::prelude::Dimension"],
    module = "sand::component",
    summary = "A dimension definition (`data/<namespace>/dimension/<id>.json`).",
    context = "A dimension definition (`data/<namespace>/dimension/<id>.json`). Dimensions reference a dimension type and a chunk generator. The chunk generator config is complex, so it is accepted through the explicit [`RawJson`] escape hatch. Use [`Dimension::generator_raw`] to replace it.",
    minecraft = "Dimensions reference a dimension type and a chunk generator. The chunk generator config is complex, so it is accepted through the explicit [`RawJson`] escape hatch. Use [`Dimension::generator_raw`] to replace it.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Dimension;",
)]
/// A dimension definition (`data/<namespace>/dimension/<id>.json`).
///
/// Dimensions reference a dimension type and a chunk generator. The chunk
/// generator config is complex, so it is accepted through the explicit
/// [`RawJson`] escape hatch. Use [`Dimension::generator_raw`] to replace it.
pub struct Dimension {
    location: ResourceLocation,
    /// The dimension type ID (e.g. `"minecraft:overworld"`, `"minecraft:the_nether"`).
    dimension_type: DimensionTypeReference,
    /// The chunk generator configuration as raw JSON.
    generator: Value,
}

impl Dimension {
    /// Creates a new dimension referencing the given type and raw generator JSON.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dimension::new",
        aliases = ["sand::prelude::Dimension::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new dimension referencing the given type and raw generator JSON.",
        context = "Creates a new dimension referencing the given type and raw generator JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new dimension referencing the given type and raw generator JSON.", dimension_type = "`dimension_type` provides the typed Minecraft resource identifier used to create a new dimension referencing the given type and raw generator JSON.", generator = "`generator` is used when creating a new dimension referencing the given type and raw generator JSON."),
        returns = "A `Dimension` representing a new dimension referencing the given type and raw generator JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, dimension_type: sand::registry::DimensionTypeId, generator: sand::component::RawJson)  {\n    let dimension = sand::component::Dimension::new(location, dimension_type, generator);\n}",
    )]
    pub fn new(
        location: ResourceLocation,
        dimension_type: DimensionTypeId,
        generator: RawJson,
    ) -> Self {
        Self {
            location,
            dimension_type: DimensionTypeReference::Typed(dimension_type),
            generator: generator.into_value(),
        }
    }

    /// Creates a dimension with an explicitly raw dimension-type reference.
    ///
    /// Prefer [`Dimension::new`] with [`DimensionTypeId`]. This escape hatch
    /// exists for version-specific or otherwise unsupported reference syntax.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dimension::new_raw_dimension_type",
        aliases = ["sand::prelude::Dimension::new_raw_dimension_type"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a dimension with an explicitly raw dimension-type reference.",
        context = "Creates a dimension with an explicitly raw dimension-type reference. Prefer [`Dimension::new`] with [`DimensionTypeId`]. This escape hatch exists for version-specific or otherwise unsupported reference syntax.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Prefer [`Dimension::new`] with [`DimensionTypeId`]. This escape hatch exists for version-specific or otherwise unsupported reference syntax."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a dimension with an explicitly raw dimension-type reference.", dimension_type = "`dimension_type` is used when creating a dimension with an explicitly raw dimension-type reference.", generator = "`generator` is used when creating a dimension with an explicitly raw dimension-type reference."),
        returns = "A `Dimension` representing a dimension with an explicitly raw dimension-type reference.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, dimension_type: impl Into < String >, generator: sand::component::RawJson)  {\n    let dimension = sand::component::Dimension::new_raw_dimension_type(location, dimension_type, generator);\n}",
    )]
    pub fn new_raw_dimension_type(
        location: ResourceLocation,
        dimension_type: impl Into<String>,
        generator: RawJson,
    ) -> Self {
        Self {
            location,
            dimension_type: DimensionTypeReference::Raw(dimension_type.into()),
            generator: generator.into_value(),
        }
    }

    /// Convenience: create with a noise-based generator pointing to a noise_settings ID.
    ///
    /// `biome_source` should wrap a raw JSON biome source object, e.g.:
    /// ```json
    /// { "type": "minecraft:fixed", "biome": "minecraft:plains" }
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dimension::noise_generator",
        aliases = ["sand::prelude::Dimension::noise_generator"],
        module = "sand::component",
        kind = "method",
        summary = "Convenience: create with a noise-based generator pointing to a noise_settings ID.",
        context = "Convenience: create with a noise-based generator pointing to a noise_settings ID. `biome_source` should wrap a raw JSON biome source object, e.g.:",
        minecraft = "`biome_source` should wrap a raw JSON biome source object, e.g.:",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` names the generated dimension resource.", dimension_type = "`dimension_type` selects the dimension-type resource.", noise_settings = "`noise_settings` selects the noise settings used by the generator.", biome_source = "`biome_source` should wrap a raw JSON biome source object, e.g.:"),
        returns = "A `Dimension` describing a noise-based generator pointing to a noise_settings ID.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, dimension_type: sand::registry::DimensionTypeId, noise_settings: impl Into < String >, biome_source: sand::component::RawJson)  {\n    let dimension = sand::component::Dimension::noise_generator(location, dimension_type, noise_settings, biome_source);\n}",
    )]
    pub fn noise_generator(
        location: ResourceLocation,
        dimension_type: DimensionTypeId,
        noise_settings: impl Into<String>,
        biome_source: RawJson,
    ) -> Self {
        let generator = serde_json::json!({
            "type": "minecraft:noise",
            "settings": noise_settings.into(),
            "biome_source": biome_source.into_value(),
        });
        Self::new(location, dimension_type, RawJson::new(generator))
    }

    /// Convenience: create with a flat (superflat) generator.
    ///
    /// `flat_settings` wraps the raw JSON settings for `minecraft:flat`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dimension::flat_generator",
        aliases = ["sand::prelude::Dimension::flat_generator"],
        module = "sand::component",
        kind = "method",
        summary = "Convenience: create with a flat (superflat) generator.",
        context = "Convenience: create with a flat (superflat) generator. `flat_settings` wraps the raw JSON settings for `minecraft:flat`.",
        minecraft = "`flat_settings` wraps the raw JSON settings for `minecraft:flat`.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to convenience create with a flat (superflat) generator.", dimension_type = "`dimension_type` provides the typed Minecraft resource identifier used to convenience create with a flat (superflat) generator.", flat_settings = "`flat_settings` wraps the raw JSON settings for `minecraft:flat`."),
        returns = "A `Dimension` describing a flat (superflat) generator.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, dimension_type: sand::registry::DimensionTypeId, flat_settings: sand::component::RawJson)  {\n    let dimension = sand::component::Dimension::flat_generator(location, dimension_type, flat_settings);\n}",
    )]
    pub fn flat_generator(
        location: ResourceLocation,
        dimension_type: DimensionTypeId,
        flat_settings: RawJson,
    ) -> Self {
        let generator = serde_json::json!({
            "type": "minecraft:flat",
            "settings": flat_settings.into_value(),
        });
        Self::new(location, dimension_type, RawJson::new(generator))
    }

    /// Updates the dimension type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dimension::dimension_type",
        aliases = ["sand::prelude::Dimension::dimension_type"],
        module = "sand::component",
        kind = "method",
        summary = "Updates the dimension type.",
        context = "Updates the dimension type. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(dt = "`dt` provides the typed Minecraft resource identifier used to update the dimension type."),
        returns = "The `Dimension` value with the documented change applied to update the dimension type.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_value: sand::component::Dimension, dt: sand::registry::DimensionTypeId)  {\n    let updated_dimension = dimension_value.dimension_type(dt);\n}",
    )]
    pub fn dimension_type(mut self, dt: DimensionTypeId) -> Self {
        self.dimension_type = DimensionTypeReference::Typed(dt);
        self
    }

    /// Updates the dimension type through the explicit raw compatibility path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dimension::raw_dimension_type",
        aliases = ["sand::prelude::Dimension::raw_dimension_type"],
        module = "sand::component",
        kind = "method",
        summary = "Updates the dimension type through the explicit raw compatibility path.",
        context = "Updates the dimension type through the explicit raw compatibility path. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(dt = "`dt` is used to update the dimension type through the explicit raw compatibility path."),
        returns = "The `Dimension` value with the documented change applied to update the dimension type through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_value: sand::component::Dimension, dt: impl Into < String >)  {\n    let updated_dimension = dimension_value.raw_dimension_type(dt);\n}",
    )]
    pub fn raw_dimension_type(mut self, dt: impl Into<String>) -> Self {
        self.dimension_type = DimensionTypeReference::Raw(dt.into());
        self
    }

    /// Replaces the generator with a raw JSON value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dimension::generator_raw",
        aliases = ["sand::prelude::Dimension::generator_raw"],
        module = "sand::component",
        kind = "method",
        summary = "Replaces the generator with a raw JSON value.",
        context = "Replaces the generator with a raw JSON value. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(generator = "`generator` provides the replacement generator when the generator with a raw JSON value."),
        returns = "The `Dimension` value with the documented change applied to replace the generator with a raw JSON value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_value: sand::component::Dimension, generator: sand::component::RawJson)  {\n    let updated_dimension = dimension_value.generator_raw(generator);\n}",
    )]
    pub fn generator_raw(mut self, generator: RawJson) -> Self {
        self.generator = generator.into_value();
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
        let dimension = Dimension::new(
            location(),
            id,
            RawJson::new(serde_json::json!({"type": "test"})),
        );
        assert_eq!(dimension.to_json()["type"], "minecraft:overworld");

        let noise = Dimension::noise_generator(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            "minecraft:overworld",
            RawJson::new(
                serde_json::json!({"type": "minecraft:fixed", "biome": "minecraft:plains"}),
            ),
        );
        assert_eq!(noise.to_json()["type"], "minecraft:overworld");
    }

    #[test]
    fn raw_dimension_type_reference_is_explicit_and_preserved() {
        let dimension = Dimension::new_raw_dimension_type(
            location(),
            "modded reference",
            RawJson::new(serde_json::json!({"type": "test"})),
        )
        .raw_dimension_type("modded:custom");
        assert_eq!(dimension.to_json()["type"], "modded:custom");
    }

    #[test]
    fn valid_dimension_passes_validation() {
        let dimension = Dimension::new(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            RawJson::new(
                serde_json::json!({"type": "minecraft:noise", "settings": "minecraft:overworld"}),
            ),
        );
        assert!(dimension.validate().is_ok());
    }

    #[test]
    fn malformed_raw_dimension_type_rejected() {
        let dimension = Dimension::new_raw_dimension_type(
            location(),
            "modded reference",
            RawJson::new(serde_json::json!({"type": "test"})),
        );
        let err = dimension.validate().unwrap_err().to_string();
        assert!(err.contains("type"), "{err}");
    }

    #[test]
    fn empty_raw_dimension_type_rejected() {
        let dimension = Dimension::new_raw_dimension_type(
            location(),
            "",
            RawJson::new(serde_json::json!({"type": "test"})),
        );
        assert!(dimension.validate().is_err());
    }

    #[test]
    fn generator_raw_non_object_rejected() {
        let dimension = Dimension::new(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            RawJson::new(serde_json::json!({"type": "test"})),
        )
        .generator_raw(RawJson::new(serde_json::json!(["not", "an", "object"])));
        let err = dimension.validate().unwrap_err().to_string();
        assert!(err.contains("generator"), "{err}");
    }

    #[test]
    fn generator_raw_missing_type_rejected() {
        let dimension = Dimension::new(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            RawJson::new(serde_json::json!({"type": "test"})),
        )
        .generator_raw(RawJson::new(
            serde_json::json!({"settings": "minecraft:overworld"}),
        ));
        let err = dimension.validate().unwrap_err().to_string();
        assert!(err.contains("generator.type"), "{err}");
    }

    #[test]
    fn noise_generator_malformed_settings_id_rejected() {
        let dimension = Dimension::noise_generator(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            "Not A Valid Id",
            RawJson::new(
                serde_json::json!({"type": "minecraft:fixed", "biome": "minecraft:plains"}),
            ),
        );
        let err = dimension.validate().unwrap_err().to_string();
        assert!(err.contains("generator.settings"), "{err}");
    }

    #[test]
    fn flat_generator_escape_hatch_still_works() {
        let dimension = Dimension::flat_generator(
            location(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            RawJson::new(
                serde_json::json!({"layers": [{"block": "minecraft:bedrock", "height": 1}]}),
            ),
        );
        assert!(dimension.validate().is_ok());
        assert_eq!(dimension.to_json()["generator"]["type"], "minecraft:flat");
    }
}
