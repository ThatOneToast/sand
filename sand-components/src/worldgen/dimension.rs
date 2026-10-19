//! Dimension resources sharing the same typed generators as build-time worlds.
use super::generator::{FlatGenerator, Generator, NoiseGenerator};
use crate::{
    ResourceLocation, component::DatapackComponent, error::Result as SandResult,
    registry::DimensionTypeId, validation,
};
use serde_json::Value;

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

/// A dimension resource with a typed chunk generator.
///
/// The generator models are shared with `sand::build`: flat, void, noise, and
/// debug worlds use the same lowering and structural validation in either path.
/// Use [`Generator::Raw`] only for unsupported generator schemas. Registry IDs
/// validate identifier syntax, not the existence of resources supplied by other packs.
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::component::Dimension",
    aliases = ["sand::prelude::Dimension"], module = "sand::component",
    summary = "A dimension resource using the shared typed chunk-generator model.",
    context = "References a DimensionTypeId and the same Generator used by build-time world configuration.",
    minecraft = "Writes data/<namespace>/dimension/<path>.json after validating the generator's supported invariants.",
    use_when = ["Registering a custom datapack dimension"],
    avoid_when = ["Configuring host server properties"],
    example = "use sand::component::Dimension;",
)]
pub struct Dimension {
    location: ResourceLocation,
    dimension_type: DimensionTypeReference,
    generator: Generator,
}

impl Dimension {
    /// Creates a dimension with a typed dimension-type reference and chunk generator.
    ///
    /// Use Generator::Raw for unsupported schemas; normal flat and noise worlds need no JSON.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Dimension::new",
        aliases = ["sand::prelude::Dimension::new"], module = "sand::component",
        summary = "Creates a dimension with a typed dimension-type reference and chunk generator.", context = "Use Generator::Raw for unsupported schemas; normal flat and noise worlds need no JSON.",
        minecraft = "Writes the dimension type and generator object; export validates supported generator shapes.",
        use_when = ["Configuring a datapack dimension with shared world-generation models"],
        avoid_when = ["Changing server-only properties"],
        params(location = "The output resource identity.", dimension_type = "The dimension-type resource.", generator = "The shared typed chunk generator."), returns = "The dimension with the requested configuration.",
        example = "fn demo(id: sand::ResourceLocation, ty: sand::registry::DimensionTypeId) { let _ = sand::component::Dimension::new(id, ty, sand::component::Generator::Void); }",
    )]
    pub fn new(
        location: ResourceLocation,
        dimension_type: DimensionTypeId,
        generator: Generator,
    ) -> Self {
        Self {
            location,
            dimension_type: DimensionTypeReference::Typed(dimension_type),
            generator,
        }
    }

    /// Creates a dimension with an explicit raw dimension-type reference.
    ///
    /// The reference is syntax-validated on export. Prefer new with DimensionTypeId for ordinary authoring.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Dimension::new_raw_dimension_type",
        aliases = ["sand::prelude::Dimension::new_raw_dimension_type"], module = "sand::component",
        summary = "Creates a dimension with an explicit raw dimension-type reference.", context = "The reference is syntax-validated on export. Prefer new with DimensionTypeId for ordinary authoring.",
        minecraft = "Writes the dimension type and generator object; export validates supported generator shapes.",
        use_when = ["Configuring a datapack dimension with shared world-generation models"],
        avoid_when = ["Changing server-only properties"],
        params(location = "The output resource identity.", dimension_type = "The explicit raw dimension-type reference.", generator = "The shared typed chunk generator."), returns = "The dimension with the requested configuration.",
        example = "fn demo(id: sand::ResourceLocation, ty: String, generator: sand::component::Generator) { let _ = sand::component::Dimension::new_raw_dimension_type(id, ty, generator); }",
    )]
    pub fn new_raw_dimension_type(
        location: ResourceLocation,
        dimension_type: impl Into<String>,
        generator: Generator,
    ) -> Self {
        Self {
            location,
            dimension_type: DimensionTypeReference::Raw(dimension_type.into()),
            generator,
        }
    }

    /// Creates a dimension using typed noise terrain and biome placement.
    ///
    /// NoiseGenerator selects vanilla or custom settings and supports fixed, checkerboard, automatic vanilla, or raw biome sources.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Dimension::noise_generator",
        aliases = ["sand::prelude::Dimension::noise_generator"], module = "sand::component",
        summary = "Creates a dimension using typed noise terrain and biome placement.", context = "NoiseGenerator selects vanilla or custom settings and supports fixed, checkerboard, automatic vanilla, or raw biome sources.",
        minecraft = "Writes the dimension type and generator object; export validates supported generator shapes.",
        use_when = ["Configuring a datapack dimension with shared world-generation models"],
        avoid_when = ["Changing server-only properties"],
        params(location = "The output resource identity.", dimension_type = "The dimension-type resource.", generator = "Noise terrain settings and biome placement."), returns = "The dimension with the requested configuration.",
        example = "fn demo(id: sand::ResourceLocation, ty: sand::registry::DimensionTypeId, generator: sand::component::NoiseGenerator) { let _ = sand::component::Dimension::noise_generator(id, ty, generator); }",
    )]
    pub fn noise_generator(
        location: ResourceLocation,
        dimension_type: DimensionTypeId,
        generator: NoiseGenerator,
    ) -> Self {
        Self::new(location, dimension_type, Generator::Noise(generator))
    }

    /// Creates a dimension using typed superflat layers, biome, and structure settings.
    ///
    /// Layers are ordered bottom to top and validated on export. Use Generator::Void for an intentionally empty world.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Dimension::flat_generator",
        aliases = ["sand::prelude::Dimension::flat_generator"], module = "sand::component",
        summary = "Creates a dimension using typed superflat layers, biome, and structure settings.", context = "Layers are ordered bottom to top and validated on export. Use Generator::Void for an intentionally empty world.",
        minecraft = "Writes the dimension type and generator object; export validates supported generator shapes.",
        use_when = ["Configuring a datapack dimension with shared world-generation models"],
        avoid_when = ["Changing server-only properties"],
        params(location = "The output resource identity.", dimension_type = "The dimension-type resource.", generator = "The superflat layer stack and options."), returns = "The dimension with the requested configuration.",
        example = "fn demo(id: sand::ResourceLocation, ty: sand::registry::DimensionTypeId, generator: sand::component::FlatGenerator) { let _ = sand::component::Dimension::flat_generator(id, ty, generator); }",
    )]
    pub fn flat_generator(
        location: ResourceLocation,
        dimension_type: DimensionTypeId,
        generator: FlatGenerator,
    ) -> Self {
        Self::new(location, dimension_type, Generator::Flat(generator))
    }

    /// Replaces the dimension-type reference.
    ///
    /// The generator remains unchanged; the referenced type determines dimension properties such as lighting and height.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Dimension::dimension_type",
        aliases = ["sand::prelude::Dimension::dimension_type"], module = "sand::component",
        summary = "Replaces the dimension-type reference.", context = "The generator remains unchanged; the referenced type determines dimension properties such as lighting and height.",
        minecraft = "Writes the dimension type and generator object; export validates supported generator shapes.",
        use_when = ["Configuring a datapack dimension with shared world-generation models"],
        avoid_when = ["Changing server-only properties"],
        params(dt = "The replacement dimension-type resource."), returns = "The dimension with the requested configuration.",
        example = "fn demo(dim: sand::component::Dimension, ty: sand::registry::DimensionTypeId) { let _ = dim.dimension_type(ty); }",
    )]
    pub fn dimension_type(mut self, dt: DimensionTypeId) -> Self {
        self.dimension_type = DimensionTypeReference::Typed(dt);
        self
    }

    /// Replaces the dimension type through an explicit raw reference.
    ///
    /// Export checks resource-location syntax. Prefer dimension_type with DimensionTypeId.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Dimension::raw_dimension_type",
        aliases = ["sand::prelude::Dimension::raw_dimension_type"], module = "sand::component",
        summary = "Replaces the dimension type through an explicit raw reference.", context = "Export checks resource-location syntax. Prefer dimension_type with DimensionTypeId.",
        minecraft = "Writes the dimension type and generator object; export validates supported generator shapes.",
        use_when = ["Configuring a datapack dimension with shared world-generation models"],
        avoid_when = ["Changing server-only properties"],
        params(dt = "The raw dimension-type reference."), returns = "The dimension with the requested configuration.",
        example = "fn demo(dim: sand::component::Dimension, ty: String) { let _ = dim.raw_dimension_type(ty); }",
    )]
    pub fn raw_dimension_type(mut self, dt: impl Into<String>) -> Self {
        self.dimension_type = DimensionTypeReference::Raw(dt.into());
        self
    }

    /// Replaces this dimension's chunk generator.
    ///
    /// Both typed and explicit Generator::Raw values replace the previous configuration completely.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Dimension::generator",
        aliases = ["sand::prelude::Dimension::generator"], module = "sand::component",
        summary = "Replaces this dimension's chunk generator.", context = "Both typed and explicit Generator::Raw values replace the previous configuration completely.",
        minecraft = "Writes the dimension type and generator object; export validates supported generator shapes.",
        use_when = ["Configuring a datapack dimension with shared world-generation models"],
        avoid_when = ["Changing server-only properties"],
        params(generator = "The replacement shared generator."), returns = "The dimension with the requested configuration.",
        example = "fn demo(dim: sand::component::Dimension) { let _ = dim.generator(sand::component::Generator::Void); }",
    )]
    pub fn generator(mut self, generator: Generator) -> Self {
        self.generator = generator;
        self
    }
}

impl DatapackComponent for Dimension {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }
    fn to_json(&self) -> Value {
        serde_json::json!({"type": self.dimension_type.as_string(), "generator": self.generator.to_json()})
    }
    fn validate(&self) -> SandResult<()> {
        if let DimensionTypeReference::Raw(dt) = &self.dimension_type {
            validation::validate_resource_location_str(&self.location, KIND, "type", dt)?;
        }
        self.generator.validate(&self.location)
    }
    fn component_dir(&self) -> &'static str {
        "dimension"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::generator::{BiomeSource, FlatLayer, VanillaNoiseSettings};
    use crate::{BiomeId, BlockId, NoiseSettingsId, RawJson};
    use serde_json::json;

    fn dimension(generator: Generator) -> Dimension {
        Dimension::new(
            ResourceLocation::new("test", "world").unwrap(),
            DimensionTypeId::minecraft("overworld").unwrap(),
            generator,
        )
    }

    #[test]
    fn typed_noise_references_and_checkerboard_order_are_preserved() {
        let dim = dimension(Generator::Noise(
            NoiseGenerator::custom_settings(NoiseSettingsId::custom(
                ResourceLocation::new("test", "terrain").unwrap(),
            ))
            .biome_source(BiomeSource::Checkerboard {
                biomes: vec![
                    BiomeId::minecraft("desert").unwrap(),
                    BiomeId::minecraft("plains").unwrap(),
                ],
                scale: 2,
            }),
        ));
        dim.validate().unwrap();
        assert_eq!(
            dim.to_json(),
            json!({
                "type": "minecraft:overworld", "generator": {
                    "type": "minecraft:noise", "settings": "test:terrain",
                    "biome_source": {"type": "minecraft:checkerboard", "biomes": ["minecraft:desert", "minecraft:plains"], "scale": 2},
                }
            })
        );
    }

    #[test]
    fn checkerboard_empty_list_and_codec_bounds_fail_with_field_context() {
        for (biomes, scale, field) in [
            (vec![], 2, "biomes"),
            (vec![BiomeId::minecraft("plains").unwrap()], -1, "scale"),
            (vec![BiomeId::minecraft("plains").unwrap()], 63, "scale"),
        ] {
            let dim = dimension(Generator::Noise(
                NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld)
                    .biome_source(BiomeSource::Checkerboard { biomes, scale }),
            ));
            let error = dim.validate().unwrap_err().to_string();
            assert!(error.contains("test:world"), "{error}");
            assert!(
                error.contains(&format!("generator.biome_source.{field}")),
                "{error}"
            );
        }
        for scale in [0, 62] {
            dimension(Generator::Noise(
                NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld).biome_source(
                    BiomeSource::Checkerboard {
                        biomes: vec![BiomeId::minecraft("plains").unwrap()],
                        scale,
                    },
                ),
            ))
            .validate()
            .unwrap();
        }
    }

    #[test]
    fn component_flat_generators_share_structural_validation_with_build_worlds() {
        for (heights, field) in [
            (vec![], "generator.settings.layers"),
            (vec![0], "generator.settings.layers[0].height"),
            (vec![u32::MAX, 2], "generator.settings.layers"),
            (vec![4065], "generator.settings.layers"),
        ] {
            let dim = dimension(Generator::Flat(FlatGenerator::new(
                heights
                    .into_iter()
                    .map(|height| FlatLayer::new(BlockId::minecraft("stone").unwrap(), height))
                    .collect(),
            )));
            let error = dim.validate().unwrap_err().to_string();
            assert!(error.contains(field), "{error}");
        }
        dimension(Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
            BlockId::minecraft("stone").unwrap(),
            4064,
        )])))
        .validate()
        .unwrap();
        dimension(Generator::Void).validate().unwrap();
    }

    #[test]
    fn raw_generator_requires_an_object_and_valid_type_but_preserves_inline_settings() {
        for payload in [
            json!("test:generator"),
            json!([]),
            json!({}),
            json!({"type": "Invalid Type"}),
        ] {
            assert!(
                dimension(Generator::Raw(RawJson::new(payload)))
                    .validate()
                    .is_err()
            );
        }
        let raw = json!({"type": "minecraft:noise", "settings": {"custom": "inline"}, "biome_source": {"type": "test:custom"}});
        let dim = dimension(Generator::Raw(RawJson::new(raw.clone())));
        dim.validate().unwrap();
        assert_eq!(dim.to_json()["generator"], raw);
    }

    #[test]
    fn raw_biome_source_has_the_same_explicit_structural_boundary() {
        for payload in [json!(null), json!({}), json!({"type": "Invalid"})] {
            let dim = dimension(Generator::Noise(
                NoiseGenerator::vanilla(VanillaNoiseSettings::Nether)
                    .biome_source(BiomeSource::Raw(RawJson::new(payload))),
            ));
            let error = dim.validate().unwrap_err().to_string();
            assert!(error.contains("generator.biome_source"), "{error}");
        }
    }

    #[test]
    fn generator_replacement_and_raw_dimension_references_are_explicit() {
        let dim = Dimension::new_raw_dimension_type(
            ResourceLocation::new("test", "world").unwrap(),
            "Invalid Type",
            Generator::Void,
        );
        assert!(dim.validate().is_err());
        let dim = dim
            .raw_dimension_type("test:dimension_type")
            .generator(Generator::Debug);
        dim.validate().unwrap();
        assert_eq!(
            dim.to_json(),
            json!({"type": "test:dimension_type", "generator": {"type": "minecraft:debug"}})
        );
    }
}
