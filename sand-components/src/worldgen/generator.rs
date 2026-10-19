//! Shared typed chunk generators for component dimensions and build-time worlds.
//!
//! Every variant here lowers into the `generator` object of a
//! `data/<namespace>/dimension/<id>.json` resource, which ships inside the
//! exported datapack and works identically in singleplayer, LAN, and any
//! vanilla-compatible server.

use crate::raw::RawJson;
use crate::registry::{BiomeId, BlockId, NoiseSettingsId};
use crate::resource_location::ResourceLocation;
use crate::{error::Result as SandResult, validation};
use sand_macros::api;
use serde_json::{Value, json};

/// A single horizontal layer in a [`Generator::Flat`] world, from the
/// bottom of the column upward.
///
/// 🌍 World (datapack).
#[derive(Debug, Clone, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::FlatLayer",
    aliases = ["sand::component::FlatLayer"],
    module = "sand::build",
    summary = "FlatLayer names one horizontal block layer in a FlatGenerator's superflat column, from bottom to top.",
    context = "FlatGenerator::new takes an ordered Vec<FlatLayer> describing the whole column.",
    minecraft = "Lowers to one entry in the flat generator's \"layers\" array (block + height).",
    use_when = ["Building a superflat world's layer stack"],
    avoid_when = ["Describing full vanilla noise terrain; use NoiseGenerator"],
    fields(block = "The block this layer is made of.", height = "How many blocks tall this layer is (must be > 0)."),
    example = "FlatLayer::new(ResourceLocation::new(\"minecraft\", \"stone\").unwrap(), 3);"
)]
pub struct FlatLayer {
    /// Block resource location, e.g. `"minecraft:stone"`.
    pub block: BlockId,
    /// Number of blocks this layer occupies (must be `> 0`).
    pub height: u32,
}

impl FlatLayer {
    #[api(
        registry = sand_api_contract,
        path = "sand::build::FlatLayer::new",
    aliases = ["sand::component::FlatLayer::new"],
        module = "sand::build",
        summary = "Creates a flat-world layer of the given block and height.",
        context = "Called once per layer when assembling a FlatGenerator's layer stack.",
        minecraft = "Produces one entry in the generated flat generator's \"layers\" array.",
        use_when = ["Adding a layer to a superflat world"],
        avoid_when = ["Choosing the overall biome; use FlatGenerator::biome"],
        params(block = "The block this layer is made of.", height = "The layer's height in blocks."),
        returns = "A new FlatLayer.",
        example = "FlatLayer::new(ResourceLocation::new(\"minecraft\", \"dirt\").unwrap(), 2);"
    )]
    pub fn new(block: impl Into<BlockId>, height: u32) -> Self {
        Self {
            block: block.into(),
            height,
        }
    }
}

/// Superflat ("flat") world generator settings.
///
/// 🌍 World (datapack) — lowers to a `minecraft:flat` chunk generator.
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::FlatGenerator",
    aliases = ["sand::component::FlatGenerator"],
    module = "sand::build",
    fields(layers = "Bottom-to-top block layers.", biome_id = "The single biome used across the world.", structures = "Whether to allow default structure sets."),
    summary = "FlatGenerator describes vanilla superflat world generation: a fixed layer stack, biome, and optional structures.",
    context = "Wrapped by Generator::Flat when assigned to a Dimension; the common dev/test-profile generator.",
    minecraft = "Lowers to a minecraft:flat chunk generator in the dimension resource.",
    use_when = ["Building a fast-to-generate dev or test world"],
    avoid_when = ["Building full vanilla noise terrain; use NoiseGenerator"],
    example = "FlatGenerator::new(vec![FlatLayer::new(ResourceLocation::new(\"minecraft\", \"grass_block\").unwrap(), 1)]);"
)]
pub struct FlatGenerator {
    /// Bottom-to-top layer stack, validated on dimension export.
    pub layers: Vec<FlatLayer>,
    /// The biome used for every column.
    pub biome_id: BiomeId,
    /// Whether to allow Minecraft's default structure sets.
    pub structures: bool,
}

impl FlatGenerator {
    /// Starts a flat generator with the given bottom-to-top layer stack.
    /// Defaults to the `minecraft:plains` biome with structure generation
    /// disabled (the vanilla superflat default).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::FlatGenerator::new",
    aliases = ["sand::component::FlatGenerator::new"],
        module = "sand::build",
        summary = "Starts a flat generator with the given bottom-to-top layer stack.",
        context = "Defaults to the minecraft:plains biome with structure generation disabled, matching vanilla's superflat default.",
        minecraft = "Produces the flat generator's \"layers\" array in generation order.",
        use_when = ["Defining a superflat world's block columns"],
        avoid_when = ["Choosing noise-based terrain; use NoiseGenerator"],
        params(layers = "The bottom-to-top layer stack."),
        returns = "A new FlatGenerator.",
        example = "FlatGenerator::new(vec![FlatLayer::new(ResourceLocation::new(\"minecraft\", \"stone\").unwrap(), 1)]);"
    )]
    pub fn new(layers: Vec<FlatLayer>) -> Self {
        Self {
            layers,
            biome_id: BiomeId::minecraft("plains").expect("valid built-in id"),
            structures: false,
        }
    }

    /// Sets the single biome used across the whole flat world.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::FlatGenerator::biome",
    aliases = ["sand::component::FlatGenerator::biome"],
        module = "sand::build",
        summary = "Sets the single biome used across the whole flat world.",
        context = "Flat worlds use one biome for every column; this overrides the minecraft:plains default.",
        minecraft = "Populates the flat generator settings' \"biome\" field.",
        use_when = ["Choosing a non-default biome for a flat world"],
        avoid_when = ["Varying biome per-column; flat worlds cannot do this"],
        params(biome = "The biome every column uses."),
        returns = "This generator with the biome overridden.",
        example = "FlatGenerator::new(vec![]).biome(ResourceLocation::new(\"minecraft\", \"desert\").unwrap());"
    )]
    pub fn biome(mut self, biome: impl Into<BiomeId>) -> Self {
        self.biome_id = biome.into();
        self
    }

    /// Enables vanilla structure generation (villages, strongholds, etc.)
    /// on top of the flat terrain.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::FlatGenerator::with_structures",
    aliases = ["sand::component::FlatGenerator::with_structures"],
        module = "sand::build",
        summary = "Enables or disables vanilla structure generation on top of the flat terrain.",
        context = "Vanilla superflat worlds disable structures by default; this opts back in.",
        minecraft = "Allows the default structure sets by omitting structure_overrides; false exports an empty override list.",
        use_when = ["Allowing villages/strongholds/etc. to generate in a flat world"],
        avoid_when = ["Choosing the layer stack itself; use FlatGenerator::new"],
        params(enabled = "Whether vanilla structures should generate."),
        returns = "This generator with structure generation set.",
        example = "FlatGenerator::new(vec![]).with_structures(true);"
    )]
    pub fn with_structures(mut self, enabled: bool) -> Self {
        self.structures = enabled;
        self
    }
}

/// Reference to the noise settings a [`Generator::Noise`] world uses.
///
/// 🌍 World (datapack).
#[derive(Debug, Clone, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::NoiseSettingsRef",
    aliases = ["sand::component::NoiseSettingsRef"],
    module = "sand::build",
    summary = "NoiseSettingsRef names which noise_settings resource a NoiseGenerator uses: a vanilla preset or a project-authored custom resource.",
    context = "Selected via NoiseGenerator::vanilla or NoiseGenerator::custom_settings.",
    minecraft = "Lowers to the noise generator's \"settings\" resource location string.",
    use_when = ["Choosing vanilla Overworld/Nether/End/Amplified noise shaping", "Referencing a hand-authored noise_settings resource"],
    avoid_when = ["Selecting biome placement; use BiomeSource"],
    variants(Vanilla = "One of vanilla's built-in worldgen/noise_settings presets.", Custom = "A project-authored data/<namespace>/worldgen/noise_settings/<path>.json resource, referenced but not generated by Sand."),
    variant_fields(Vanilla = ["Which vanilla preset to use."], Custom = ["The custom noise settings resource location."]),
    example = "NoiseSettingsRef::Vanilla(VanillaNoiseSettings::Overworld);"
)]
pub enum NoiseSettingsRef {
    /// One of vanilla's built-in `worldgen/noise_settings` presets.
    Vanilla(VanillaNoiseSettings),
    /// A project-authored `data/<namespace>/worldgen/noise_settings/<path>.json`
    /// resource. Sand does not generate this file — author it directly (see
    /// the "Custom dimensions" book chapter) and reference it here.
    Custom(NoiseSettingsId),
}

/// Vanilla's built-in noise settings presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::VanillaNoiseSettings",
    aliases = ["sand::component::VanillaNoiseSettings"],
    module = "sand::build",
    summary = "VanillaNoiseSettings enumerates vanilla's built-in worldgen/noise_settings presets.",
    context = "Selected via NoiseGenerator::vanilla to reuse a known Minecraft terrain shape.",
    minecraft = "Each variant maps to one vanilla noise_settings resource location under the minecraft namespace.",
    use_when = ["Reusing vanilla Overworld/Nether/End/Amplified/Caves/FloatingIslands terrain shaping"],
    avoid_when = ["Authoring a fully custom noise settings resource; use NoiseSettingsRef::Custom"],
    variants(Overworld = "Vanilla's standard Overworld noise settings.", LargeBiomes = "Vanilla's Large Biomes noise settings.", Amplified = "Vanilla's Amplified noise settings (dramatic terrain).", Nether = "Vanilla's Nether noise settings.", End = "Vanilla's End noise settings.", Caves = "Vanilla's Caves noise settings.", FloatingIslands = "Vanilla's Floating Islands noise settings."),
    example = "VanillaNoiseSettings::Overworld;"
)]
pub enum VanillaNoiseSettings {
    Overworld,
    LargeBiomes,
    Amplified,
    Nether,
    End,
    Caves,
    FloatingIslands,
}

impl VanillaNoiseSettings {
    #[api(
        registry = sand_api_contract,
        path = "sand::build::VanillaNoiseSettings::resource_location",
    aliases = ["sand::component::VanillaNoiseSettings::resource_location"],
        module = "sand::build",
        summary = "Returns the vanilla resource location for this noise settings preset.",
        context = "Used by generator lowering to populate the noise generator's \"settings\" field.",
        minecraft = "Matches the vanilla minecraft:<preset> worldgen/noise_settings resource location.",
        use_when = ["Looking up which resource location a preset maps to"],
        avoid_when = ["Choosing biome placement; use BiomeSource"],
        returns = "The preset's resource location.",
        example = "assert_eq!(VanillaNoiseSettings::Overworld.resource_location().to_string(), \"minecraft:overworld\");"
    )]
    pub fn resource_location(self) -> ResourceLocation {
        let path = match self {
            VanillaNoiseSettings::Overworld => "overworld",
            VanillaNoiseSettings::LargeBiomes => "large_biomes",
            VanillaNoiseSettings::Amplified => "amplified",
            VanillaNoiseSettings::Nether => "nether",
            VanillaNoiseSettings::End => "end",
            VanillaNoiseSettings::Caves => "caves",
            VanillaNoiseSettings::FloatingIslands => "floating_islands",
        };
        ResourceLocation::new("minecraft", path).expect("valid built-in id")
    }
}

/// Full vanilla noise generation.
///
/// 🌍 World (datapack) — lowers to a `minecraft:noise` chunk generator.
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::NoiseGenerator",
    aliases = ["sand::component::NoiseGenerator"],
    module = "sand::build",
    fields(settings = "Terrain-shaping settings reference.", source = "How terrain columns select biomes."),
    summary = "NoiseGenerator describes full vanilla-style noise-based terrain generation for a dimension.",
    context = "Wrapped by Generator::Noise when assigned to a Dimension; the typical release-profile generator.",
    minecraft = "Lowers to a minecraft:noise chunk generator in the dimension resource.",
    use_when = ["Building a shipped-quality world with real terrain shaping"],
    avoid_when = ["Building a fast dev/test world; use FlatGenerator or Generator::Void"],
    example = "NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld);"
)]
pub struct NoiseGenerator {
    /// Terrain-density settings resource.
    pub settings: NoiseSettingsRef,
    /// Biome placement, independent of terrain shaping.
    pub source: BiomeSource,
}

/// How a noise generator selects biomes per-column.
///
/// 🌍 World (datapack).
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::BiomeSource",
    aliases = ["sand::component::BiomeSource"],
    module = "sand::build",
    summary = "BiomeSource selects how a NoiseGenerator assigns a biome to each column.",
    context = "Set implicitly to Vanilla by NoiseGenerator::vanilla/custom_settings, or overridden via .single_biome(...).",
    minecraft = "Lowers to the noise generator's \"biome_source\" object.",
    use_when = ["Choosing standard multi-noise biome variety or a single fixed biome"],
    avoid_when = ["Choosing the terrain shape itself; use NoiseSettingsRef"],
    variants(Vanilla = "Vanilla biome placement matching the terrain preset (Overworld, Nether, or End).", Fixed = "Every column uses the same biome.", Checkerboard = "Alternate a nonempty ordered biome list at a scale in 0..=62.", Raw = "Explicit unsupported biome-source object."),
    variant_fields(Fixed = ["The single biome every column uses."], Checkerboard(biomes = "Nonempty ordered biome list.", scale = "Minecraft checkerboard scale in 0..=62."), Raw = ["Structurally validated biome-source JSON."]),
    example = "BiomeSource::Fixed(ResourceLocation::new(\"minecraft\", \"desert\").unwrap().into());"
)]
pub enum BiomeSource {
    /// Biome placement matching the noise settings: Nether uses its multi-noise
    /// preset, End uses the End biome source, and other settings use the
    /// Overworld multi-noise preset. Custom settings default to Overworld.
    Vanilla,
    /// Every column uses the same biome (vanilla's "single biome" preset,
    /// e.g. for a desert-only or ocean-only world).
    Fixed(BiomeId),
    /// Alternate biomes horizontally in declaration order. The list must be
    /// nonempty and scale must be in Minecraft's codec range `0..=62`.
    Checkerboard {
        /// Nonempty ordered biome references.
        biomes: Vec<BiomeId>,
        /// Checkerboard scale, in `0..=62` (vanilla's default is 2).
        scale: i32,
    },
    /// Explicit unsupported biome-source JSON. Only object/type shape is checked.
    Raw(RawJson),
}

impl NoiseGenerator {
    /// A noise generator using one of vanilla's built-in presets with
    /// matching vanilla biome placement.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::NoiseGenerator::vanilla",
    aliases = ["sand::component::NoiseGenerator::vanilla"],
        module = "sand::build",
        summary = "Creates a noise generator using one of vanilla's built-in presets with matching vanilla biome placement.",
        context = "The common release-profile choice: real vanilla terrain with vanilla biome variety.",
        minecraft = "Lowers to a minecraft:noise generator referencing the given vanilla noise_settings and the matching Overworld, Nether, or End biome source.",
        use_when = ["Building standard vanilla terrain and biome variety"],
        avoid_when = ["Restricting the world to one biome; chain .single_biome(...)"],
        params(settings = "Which vanilla noise settings preset to use."),
        returns = "A new NoiseGenerator.",
        example = "NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld);"
    )]
    pub fn vanilla(settings: VanillaNoiseSettings) -> Self {
        Self {
            settings: NoiseSettingsRef::Vanilla(settings),
            source: BiomeSource::Vanilla,
        }
    }

    /// A noise generator referencing a project-authored noise settings
    /// resource (see [`NoiseSettingsRef::Custom`]).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::NoiseGenerator::custom_settings",
    aliases = ["sand::component::NoiseGenerator::custom_settings"],
        module = "sand::build",
        summary = "Creates a noise generator referencing a project-authored noise settings resource.",
        context = "For worlds needing custom density functions or terrain shaping beyond the vanilla presets.",
        minecraft = "Lowers to a minecraft:noise generator whose \"settings\" references the given custom resource.",
        use_when = ["Referencing hand-authored worldgen/noise_settings JSON"],
        avoid_when = ["Reusing a vanilla preset; use NoiseGenerator::vanilla"],
        params(settings = "The custom noise settings resource location."),
        returns = "A new NoiseGenerator.",
        example = "NoiseGenerator::custom_settings(ResourceLocation::new(\"my_pack\", \"custom_overworld\").unwrap());"
    )]
    pub fn custom_settings(settings: impl Into<NoiseSettingsId>) -> Self {
        Self {
            settings: NoiseSettingsRef::Custom(settings.into()),
            source: BiomeSource::Vanilla,
        }
    }

    /// Overrides biome placement so every column uses a single fixed biome.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::NoiseGenerator::single_biome",
    aliases = ["sand::component::NoiseGenerator::single_biome"],
        module = "sand::build",
        summary = "Overrides biome placement so every column in this dimension uses one fixed biome.",
        context = "Vanilla's \"single biome\" preset, useful for e.g. a desert-only or ocean-only world.",
        minecraft = "Lowers to a minecraft:fixed biome_source referencing the given biome.",
        use_when = ["Building a world with exactly one biome everywhere"],
        avoid_when = ["Keeping standard vanilla biome variety; the default"],
        params(biome = "The biome every column uses."),
        returns = "This generator with biome placement fixed.",
        example = "NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld).single_biome(ResourceLocation::new(\"minecraft\", \"desert\").unwrap());"
    )]
    pub fn single_biome(mut self, biome: impl Into<BiomeId>) -> Self {
        self.source = BiomeSource::Fixed(biome.into());
        self
    }
    /// Replaces biome placement while preserving the terrain settings.
    #[api(
        registry = sand_api_contract, path = "sand::build::NoiseGenerator::biome_source",
        aliases = ["sand::component::NoiseGenerator::biome_source"], module = "sand::build",
        summary = "Chooses biome placement independently of terrain shaping.",
        context = "Accepts fixed, automatic vanilla, checkerboard, or explicit raw biome sources.",
        minecraft = "Replaces the noise generator's biome_source object.",
        use_when = ["Selecting typed checkerboard or custom biome placement"],
        avoid_when = ["Changing terrain density; select noise settings instead"],
        params(source = "The replacement biome source."), returns = "This noise generator with updated biome placement.",
        example = "NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld).biome_source(BiomeSource::Vanilla);"
    )]
    pub fn biome_source(mut self, source: BiomeSource) -> Self {
        self.source = source;
        self
    }
}

/// The chunk generator for a dimension.
///
/// 🌍 World (datapack) — every variant lowers into the generated
/// `data/<namespace>/dimension/<id>.json` resource's `generator` object.
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::Generator",
    aliases = ["sand::component::Generator"],
    module = "sand::build",
    summary = "Generator selects which chunk generator a Dimension uses: flat, void, noise, debug, or an explicit raw object.",
    context = "Assigned to a Dimension via Dimension::generator; every variant lowers into the dimension resource's \"generator\" object.",
    minecraft = "Lowers into the dimension resource's \"generator\" object; the concrete JSON shape depends on the chosen variant.",
    use_when = ["Choosing how a dimension's terrain is generated"],
    avoid_when = ["Configuring which dimension slot or type this applies to; use DimensionSlot/DimensionType"],
    variants(Flat = "Superflat terrain — fast to generate, ideal for dev/test profiles.", Void = "Nothing but air. Fastest possible generation.", Noise = "Full vanilla or custom noise-based terrain generation.", Debug = "Vanilla debug block-state world.", Raw = "Explicit unsupported generator JSON object."),
    variant_fields(Flat = ["The flat generator configuration."], Noise = ["The noise generator configuration."], Raw = ["Structurally validated generator JSON object."]),
    example = "Generator::Flat(FlatGenerator::new(vec![]));"
)]
pub enum Generator {
    /// Superflat terrain — fast to generate, ideal for `dev`/`test`
    /// profiles.
    Flat(FlatGenerator),
    /// Nothing but air (and the void). Fastest possible generation.
    Void,
    /// Full vanilla (or custom) noise-based terrain generation — the
    /// typical `release` profile choice.
    Noise(NoiseGenerator),
    /// Vanilla debug generator, containing the block-state showcase.
    Debug,
    /// Explicit unsupported generator JSON. Minecraft requires an object with
    /// a resource-location `type`, not a reference to a separate generator file.
    Raw(RawJson),
}

impl Generator {
    /// A convenience void-world flat generator: an empty layer stack with no
    /// structures, using the vanilla `minecraft:the_void` biome.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::Generator::void",
    aliases = ["sand::component::Generator::void"],
        module = "sand::build",
        summary = "Convenience constructor for a void-world generator.",
        context = "Equivalent to Generator::Void; provided for readability at call sites that build generators fluently.",
        minecraft = "Lowers to a minecraft:flat generator with the minecraft:the_void biome and no layers.",
        use_when = ["Building a void world"],
        avoid_when = ["Building a superflat world with visible terrain; use FlatGenerator"],
        returns = "Generator::Void.",
        example = "let generator = Generator::void();"
    )]
    pub fn void() -> Self {
        Generator::Void
    }

    pub(crate) fn validate(&self, location: &ResourceLocation) -> SandResult<()> {
        let fail =
            |field: &str, message: &str| validation::error(location, "dimension", field, message);
        match self {
            Self::Flat(flat) => {
                if flat.layers.is_empty() {
                    return Err(fail(
                        "generator.settings.layers",
                        "FlatGenerator has no layers — a flat world needs at least one",
                    ));
                }
                for (index, layer) in flat.layers.iter().enumerate() {
                    if layer.height == 0 {
                        return Err(fail(
                            &format!("generator.settings.layers[{index}].height"),
                            &format!("FlatLayer for block '{}' has height 0", layer.block),
                        ));
                    }
                }
                let total: u64 = flat.layers.iter().map(|l| u64::from(l.height)).sum();
                if total > 384 {
                    return Err(fail(
                        "generator.settings.layers",
                        &format!(
                            "FlatGenerator layers total {total} blocks tall, exceeding the maximum build height of 384"
                        ),
                    ));
                }
            }
            Self::Noise(noise) => match &noise.source {
                BiomeSource::Checkerboard { biomes, scale } => {
                    if biomes.is_empty() {
                        return Err(fail(
                            "generator.biome_source.biomes",
                            "checkerboard biome list must not be empty",
                        ));
                    }
                    if !(0..=62).contains(scale) {
                        return Err(fail(
                            "generator.biome_source.scale",
                            "checkerboard scale must be in 0..=62",
                        ));
                    }
                }
                BiomeSource::Raw(raw) => validate_raw_type(
                    location,
                    "generator.biome_source",
                    &raw.clone().into_value(),
                )?,
                BiomeSource::Vanilla | BiomeSource::Fixed(_) => {}
            },
            Self::Raw(raw) => validate_raw_type(location, "generator", &raw.clone().into_value())?,
            Self::Void | Self::Debug => {}
        }
        Ok(())
    }

    pub(crate) fn to_json(&self) -> Value {
        match self {
            Generator::Flat(flat) => {
                let mut generator = json!({
                "type": "minecraft:flat",
                "settings": {
                    "biome": flat.biome_id.to_string(),
                    "lakes": false,
                    "features": false,
                    "layers": flat.layers.iter().map(|l| json!({
                        "block": l.block.to_string(),
                        "height": l.height,
                    })).collect::<Vec<_>>(),
                    "structure_overrides": [],
                }
                });
                if flat.structures {
                    generator["settings"]
                        .as_object_mut()
                        .expect("settings object")
                        .remove("structure_overrides");
                }
                generator
            }
            Generator::Void => json!({
                "type": "minecraft:flat",
                "settings": {
                    "biome": "minecraft:the_void",
                    "lakes": false,
                    "features": false,
                    "layers": [],
                    "structure_overrides": [],
                }
            }),
            Generator::Noise(noise) => {
                let settings_id = match &noise.settings {
                    NoiseSettingsRef::Vanilla(v) => v.resource_location().to_string(),
                    NoiseSettingsRef::Custom(rl) => rl.to_string(),
                };
                let mut obj = json!({
                    "type": "minecraft:noise",
                    "settings": settings_id,
                });
                match &noise.source {
                    BiomeSource::Vanilla => {
                        obj["biome_source"] = match &noise.settings {
                            NoiseSettingsRef::Vanilla(VanillaNoiseSettings::Nether) => {
                                json!({ "type": "minecraft:multi_noise", "preset": "minecraft:nether" })
                            }
                            NoiseSettingsRef::Vanilla(VanillaNoiseSettings::End) => {
                                json!({ "type": "minecraft:the_end" })
                            }
                            _ => {
                                json!({ "type": "minecraft:multi_noise", "preset": "minecraft:overworld" })
                            }
                        };
                    }
                    BiomeSource::Checkerboard { biomes, scale } => {
                        obj["biome_source"] = json!({"type": "minecraft:checkerboard", "biomes": biomes.iter().map(ToString::to_string).collect::<Vec<_>>(), "scale": scale});
                    }
                    BiomeSource::Raw(raw) => obj["biome_source"] = raw.clone().into_value(),
                    BiomeSource::Fixed(biome) => {
                        obj["biome_source"] = json!({
                            "type": "minecraft:fixed",
                            "biome": biome.to_string(),
                        });
                    }
                }
                obj
            }
            Generator::Debug => json!({"type": "minecraft:debug"}),
            Generator::Raw(raw) => raw.clone().into_value(),
        }
    }
}

fn validate_raw_type(location: &ResourceLocation, field: &str, value: &Value) -> SandResult<()> {
    validation::require_json_object(location, "dimension", field, value)?;
    let type_field = format!("{field}.type");
    match value.get("type").and_then(Value::as_str) {
        Some(id) => {
            validation::validate_resource_location_str(location, "dimension", &type_field, id)
        }
        None => Err(validation::error(
            location,
            "dimension",
            &type_field,
            "expected a resource-location type string",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl(ns: &str, path: &str) -> ResourceLocation {
        ResourceLocation::new(ns, path).unwrap()
    }

    #[test]
    fn flat_generator_lowers_layers_in_order() {
        let generated = Generator::Flat(FlatGenerator::new(vec![
            FlatLayer::new(rl("minecraft", "bedrock"), 1),
            FlatLayer::new(rl("minecraft", "dirt"), 2),
            FlatLayer::new(rl("minecraft", "grass_block"), 1),
        ]));
        let json = generated.to_json();
        assert_eq!(json["type"], "minecraft:flat");
        assert_eq!(json["settings"]["layers"][0]["block"], "minecraft:bedrock");
        assert_eq!(json["settings"]["layers"][0]["height"], 1);
        assert_eq!(
            json["settings"]["layers"][2]["block"],
            "minecraft:grass_block"
        );
        assert_eq!(json["settings"]["biome"], "minecraft:plains");
    }

    #[test]
    fn void_generator_has_no_layers_and_the_void_biome() {
        let json = Generator::Void.to_json();
        assert_eq!(json["settings"]["biome"], "minecraft:the_void");
        assert_eq!(json["settings"]["layers"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn noise_generator_references_vanilla_overworld_settings() {
        let generated = Generator::Noise(NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld));
        let json = generated.to_json();
        assert_eq!(json["type"], "minecraft:noise");
        assert_eq!(json["settings"], "minecraft:overworld");
        assert_eq!(json["biome_source"]["type"], "minecraft:multi_noise");
    }

    #[test]
    fn single_biome_generator_uses_a_fixed_biome_source() {
        let generated = Generator::Noise(
            NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld)
                .single_biome(rl("minecraft", "desert")),
        );
        let json = generated.to_json();
        assert_eq!(json["biome_source"]["type"], "minecraft:fixed");
        assert_eq!(json["biome_source"]["biome"], "minecraft:desert");
    }

    #[test]
    fn dev_flat_and_release_noise_generators_produce_different_json() {
        let dev = Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
            rl("minecraft", "grass_block"),
            1,
        )]))
        .to_json();
        let release =
            Generator::Noise(NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld)).to_json();
        assert_ne!(dev["type"], release["type"]);
    }
}
