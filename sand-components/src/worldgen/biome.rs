//! Biome builder for `data/<namespace>/worldgen/biome/<id>.json`.
//!
//! This module types the biome-effects ambient sound reference, the
//! `temperature_modifier` field, the per-generation-step `features`
//! list-of-lists, and the step-grouped `carvers` map — see
//! [`BiomeEffects::ambient_sound`], [`Biome::temperature_modifier`],
//! [`Biome::feature`], [`Biome::carver_step`], [`Biome::spawners`], and
//! [`Biome::spawn_costs`]. Explicit raw setters support unsupported schemas.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::{ConfiguredCarverId, ConfiguredFeatureId, EntityTypeId, SoundEventId};
use crate::resource_location::ResourceLocation;
use crate::validation;
use crate::worldgen::structure::{GenerationStep, MobCategory, SpawnEntry};

/// Maximum value for an RGB integer color field (`0xFFFFFF`).
const MAX_RGB_COLOR: u32 = 0xFF_FFFF;

const KIND: &str = "worldgen/biome";

/// Number of vanilla feature-generation decoration steps — the fixed length
/// of the `features` list-of-lists array. See [`GenerationStep`].
const GENERATION_STEP_COUNT: usize = 11;

/// Population-density cost for one entity type in a biome.
///
/// Minecraft compares nearby spawn charges with the energy budget when deciding
/// whether another mob may spawn. Both values must be finite JSON numbers;
/// Minecraft's codec imposes no sign restriction. Assign costs by entity type
/// with [`Biome::spawn_costs`].
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::component::SpawnCost",
    aliases = ["sand::prelude::SpawnCost"], module = "sand::component",
    summary = "A biome's population-density budget and charge for one entity type.",
    context = "Used in Biome::spawn_costs alongside typed EntityTypeId keys.",
    minecraft = "Serializes energy_budget and charge; non-finite values fail export.",
    use_when = ["Controlling biome mob population density"],
    avoid_when = ["Choosing spawn weights or group sizes; use SpawnEntry"],
    example = "use sand::component::SpawnCost; let cost = SpawnCost::new(1.0, 0.2);",
)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpawnCost {
    energy_budget: f64,
    charge: f64,
}

impl SpawnCost {
    /// Sets the density budget and per-mob charge. Export rejects NaN and infinity.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::SpawnCost::new",
        aliases = ["sand::prelude::SpawnCost::new"], module = "sand::component",
        summary = "Creates a biome spawn-density cost.",
        context = "Pass this value in the entity-keyed table accepted by Biome::spawn_costs.",
        minecraft = "Writes energy_budget and charge as finite double-precision JSON numbers.",
        use_when = ["Configuring density limits for a biome's entity type"],
        avoid_when = ["Setting the relative spawn probability; use SpawnEntry"],
        params(energy_budget = "The finite density budget.", charge = "The finite per-mob charge."),
        returns = "A cost validated when its biome is exported.",
        example = "use sand::component::SpawnCost; let cost = SpawnCost::new(1.0, 0.2);",
    )]
    pub fn new(energy_budget: f64, charge: f64) -> Self {
        Self {
            energy_budget,
            charge,
        }
    }

    fn to_json(self) -> Value {
        serde_json::json!({"energy_budget": self.energy_budget, "charge": self.charge})
    }

    fn validate(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        for (name, value) in [
            ("energy_budget", self.energy_budget),
            ("charge", self.charge),
        ] {
            if !value.is_finite() {
                return Err(validation::error(
                    location,
                    KIND,
                    &format!("{field}.{name}"),
                    "must be finite",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum SpawnValue<T> {
    Typed(T),
    Raw(Value),
}

// ── AmbientSoundReference ────────────────────────────────────────────────────

/// How a [`BiomeEffects::ambient_sound`] reference was supplied.
#[derive(Debug, Clone)]
enum AmbientSoundReference {
    Typed(SoundEventId),
    Raw(String),
}

impl AmbientSoundReference {
    fn as_string(&self) -> String {
        match self {
            Self::Typed(id) => id.to_string(),
            Self::Raw(id) => id.clone(),
        }
    }
}

// ── FeaturesValue ─────────────────────────────────────────────────────────────

/// How [`Biome::features`] was supplied.
#[derive(Debug, Clone)]
enum FeaturesValue {
    /// Fixed-length (`GENERATION_STEP_COUNT`) per-step lists of typed
    /// configured-feature references, indexed by [`GenerationStep::index`].
    Typed(Vec<Vec<ConfiguredFeatureId>>),
    Raw(Value),
}

// ── TemperatureModifier ──────────────────────────────────────────────────────

/// Typed `temperature_modifier` value for a [`Biome`].
///
/// Vanilla currently only accepts `"none"` and `"frozen"`; use
/// [`Biome::raw_temperature_modifier`] if a future Minecraft version adds
/// more accepted values before Sand's typed enum is updated.
///
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::TemperatureModifier",
    module = "sand::component",
    summary = "Typed `temperature_modifier` value for a [`Biome`].",
    context = "Typed `temperature_modifier` value for a [`Biome`]. Vanilla currently only accepts `\"none\"` and `\"frozen\"`; use [`Biome::raw_temperature_modifier`] if a future Minecraft version adds more accepted values before Sand's typed enum is updated.",
    minecraft = "Vanilla currently only accepts `\"none\"` and `\"frozen\"`; use [`Biome::raw_temperature_modifier`] if a future Minecraft version adds more accepted values before Sand's typed enum is updated.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::TemperatureModifier;",
    variants(Frozen = "`\"frozen\"` — used by biomes like frozen ocean.", None = "`\"none\"` — no modification (the default)."),
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureModifier {
    /// `"none"` — no modification (the default).
    None,
    /// `"frozen"` — used by biomes like frozen ocean.
    Frozen,
}

impl TemperatureModifier {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Frozen => "frozen",
        }
    }
}

/// How [`Biome::temperature_modifier`] was supplied.
#[derive(Debug, Clone)]
enum TemperatureModifierValue {
    Typed(TemperatureModifier),
    Raw(String),
}

impl TemperatureModifierValue {
    fn as_string(&self) -> String {
        match self {
            Self::Typed(m) => m.as_str().to_string(),
            Self::Raw(s) => s.clone(),
        }
    }
}

// ── BiomeEffects ──────────────────────────────────────────────────────────────

/// Visual and audio effects for a biome.
///
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::BiomeEffects",
    module = "sand::component",
    summary = "Visual and audio effects for a biome.",
    context = "Visual and audio effects for a biome. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::BiomeEffects;",
)]
#[derive(Clone)]
pub struct BiomeEffects {
    /// Fog color (RGB integer, e.g. `0xC0D8FF`).
    fog_color: u32,
    /// Water color (RGB integer).
    water_color: u32,
    /// Water fog color (RGB integer).
    water_fog_color: u32,
    /// Sky color (RGB integer).
    sky_color: u32,
    /// Optional grass color override (RGB integer).
    grass_color: Option<u32>,
    /// Optional foliage color override (RGB integer).
    foliage_color: Option<u32>,
    /// Ambient particle effect (raw JSON, optional).
    particle: Option<Value>,
    /// Ambient sound event reference (optional).
    ambient_sound: Option<AmbientSoundReference>,
    /// Mood sound (raw JSON, optional).
    mood_sound: Option<Value>,
    /// Additions sound (raw JSON, optional).
    additions_sound: Option<Value>,
    /// Background music (raw JSON, optional).
    music: Option<Value>,
}

impl BiomeEffects {
    /// Creates effects with the minimum required colors.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BiomeEffects::new",
        module = "sand::component",
        kind = "method",
        summary = "Creates effects with the minimum required colors.",
        context = "Creates effects with the minimum required colors. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(fog_color = "`fog_color` is used when creating effects with the minimum required colors.", water_color = "`water_color` is used when creating effects with the minimum required colors.", water_fog_color = "`water_fog_color` is used when creating effects with the minimum required colors.", sky_color = "`sky_color` is used when creating effects with the minimum required colors."),
        returns = "A `BiomeEffects` representing effects with the minimum required colors.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fog_color: u32, water_color: u32, water_fog_color: u32, sky_color: u32)  {\n    let biome_effects = sand::component::BiomeEffects::new(fog_color, water_color, water_fog_color, sky_color);\n}",
    )]
    pub fn new(fog_color: u32, water_color: u32, water_fog_color: u32, sky_color: u32) -> Self {
        Self {
            fog_color,
            water_color,
            water_fog_color,
            sky_color,
            grass_color: None,
            foliage_color: None,
            particle: None,
            ambient_sound: None,
            mood_sound: None,
            additions_sound: None,
            music: None,
        }
    }

    /// Overrides the grass color.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BiomeEffects::grass_color",
        module = "sand::component",
        kind = "method",
        summary = "Overrides the grass color.",
        context = "Overrides the grass color. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(color = "`color` is used to override the grass color."),
        returns = "The `BiomeEffects` value with the documented change applied to override the grass color.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_effects_value: sand::component::BiomeEffects, color: u32)  {\n    let updated_biome_effects = biome_effects_value.grass_color(color);\n}",
    )]
    pub fn grass_color(mut self, color: u32) -> Self {
        self.grass_color = Some(color);
        self
    }

    /// Overrides the foliage color.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BiomeEffects::foliage_color",
        module = "sand::component",
        kind = "method",
        summary = "Overrides the foliage color.",
        context = "Overrides the foliage color. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(color = "`color` is used to override the foliage color."),
        returns = "The `BiomeEffects` value with the documented change applied to override the foliage color.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_effects_value: sand::component::BiomeEffects, color: u32)  {\n    let updated_biome_effects = biome_effects_value.foliage_color(color);\n}",
    )]
    pub fn foliage_color(mut self, color: u32) -> Self {
        self.foliage_color = Some(color);
        self
    }

    /// Sets the ambient particle effect through the explicit raw JSON escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BiomeEffects::particle",
        module = "sand::component",
        kind = "method",
        summary = "Sets the ambient particle effect through the explicit raw JSON escape hatch.",
        context = "Sets the ambient particle effect through the explicit raw JSON escape hatch. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(particle = "`particle` provides the particle applied when setting the ambient particle effect through the explicit raw JSON escape hatch."),
        returns = "The `BiomeEffects` value with the documented change applied to set the ambient particle effect through the explicit raw JSON escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_effects_value: sand::component::BiomeEffects, particle: sand::component::RawJson)  {\n    let updated_biome_effects = biome_effects_value.particle(particle);\n}",
    )]
    pub fn particle(mut self, particle: RawJson) -> Self {
        self.particle = Some(particle.into_value());
        self
    }

    /// Sets the ambient loop sound to a typed [`SoundEventId`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BiomeEffects::ambient_sound",
        module = "sand::component",
        kind = "method",
        summary = "Sets the ambient loop sound to a typed [`SoundEventId`].",
        context = "Sets the ambient loop sound to a typed [`SoundEventId`]. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(sound = "`sound` provides the typed Minecraft resource identifier used to set the ambient loop sound to a typed [`SoundEventId`]."),
        returns = "The `BiomeEffects` value with the documented change applied to set the ambient loop sound to a typed [`SoundEventId`].",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_effects_value: sand::component::BiomeEffects, sound: sand::registry::SoundEventId)  {\n    let updated_biome_effects = biome_effects_value.ambient_sound(sound);\n}",
    )]
    pub fn ambient_sound(mut self, sound: SoundEventId) -> Self {
        self.ambient_sound = Some(AmbientSoundReference::Typed(sound));
        self
    }

    /// Sets the ambient loop sound through the explicit raw compatibility
    /// path.
    ///
    /// Prefer [`BiomeEffects::ambient_sound`] with a [`SoundEventId`]. This
    /// escape hatch exists for modded or version-specific sound references.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BiomeEffects::raw_ambient_sound",
        module = "sand::component",
        kind = "method",
        summary = "Sets the ambient loop sound through the explicit raw compatibility path.",
        context = "Sets the ambient loop sound through the explicit raw compatibility path. Prefer [`BiomeEffects::ambient_sound`] with a [`SoundEventId`]. This escape hatch exists for modded or version-specific sound references.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Prefer [`BiomeEffects::ambient_sound`] with a [`SoundEventId`]. This escape hatch exists for modded or version-specific sound references."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(sound = "`sound` provides the sound applied when setting the ambient loop sound through the explicit raw compatibility path."),
        returns = "The `BiomeEffects` value with the documented change applied to set the ambient loop sound through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_effects_value: sand::component::BiomeEffects, sound: impl Into < String >)  {\n    let updated_biome_effects = biome_effects_value.raw_ambient_sound(sound);\n}",
    )]
    pub fn raw_ambient_sound(mut self, sound: impl Into<String>) -> Self {
        self.ambient_sound = Some(AmbientSoundReference::Raw(sound.into()));
        self
    }

    /// Sets the mood sound through the explicit raw JSON escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BiomeEffects::mood_sound",
        module = "sand::component",
        kind = "method",
        summary = "Sets the mood sound through the explicit raw JSON escape hatch.",
        context = "Sets the mood sound through the explicit raw JSON escape hatch. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(sound = "`sound` provides the sound applied when setting the mood sound through the explicit raw JSON escape hatch."),
        returns = "The `BiomeEffects` value with the documented change applied to set the mood sound through the explicit raw JSON escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_effects_value: sand::component::BiomeEffects, sound: sand::component::RawJson)  {\n    let updated_biome_effects = biome_effects_value.mood_sound(sound);\n}",
    )]
    pub fn mood_sound(mut self, sound: RawJson) -> Self {
        self.mood_sound = Some(sound.into_value());
        self
    }

    /// Sets the additions sound through the explicit raw JSON escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BiomeEffects::additions_sound",
        module = "sand::component",
        kind = "method",
        summary = "Sets the additions sound through the explicit raw JSON escape hatch.",
        context = "Sets the additions sound through the explicit raw JSON escape hatch. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(sound = "`sound` provides the sound applied when setting the additions sound through the explicit raw JSON escape hatch."),
        returns = "The `BiomeEffects` value with the documented change applied to set the additions sound through the explicit raw JSON escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_effects_value: sand::component::BiomeEffects, sound: sand::component::RawJson)  {\n    let updated_biome_effects = biome_effects_value.additions_sound(sound);\n}",
    )]
    pub fn additions_sound(mut self, sound: RawJson) -> Self {
        self.additions_sound = Some(sound.into_value());
        self
    }

    /// Sets the background music through the explicit raw JSON escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BiomeEffects::music",
        module = "sand::component",
        kind = "method",
        summary = "Sets the background music through the explicit raw JSON escape hatch.",
        context = "Sets the background music through the explicit raw JSON escape hatch. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(music = "`music` provides the music applied when setting the background music through the explicit raw JSON escape hatch."),
        returns = "The `BiomeEffects` value with the documented change applied to set the background music through the explicit raw JSON escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_effects_value: sand::component::BiomeEffects, music: sand::component::RawJson)  {\n    let updated_biome_effects = biome_effects_value.music(music);\n}",
    )]
    pub fn music(mut self, music: RawJson) -> Self {
        self.music = Some(music.into_value());
        self
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "fog_color".to_string(),
            Value::Number(self.fog_color.into()),
        );
        map.insert(
            "water_color".to_string(),
            Value::Number(self.water_color.into()),
        );
        map.insert(
            "water_fog_color".to_string(),
            Value::Number(self.water_fog_color.into()),
        );
        map.insert(
            "sky_color".to_string(),
            Value::Number(self.sky_color.into()),
        );
        if let Some(gc) = self.grass_color {
            map.insert("grass_color".to_string(), Value::Number(gc.into()));
        }
        if let Some(fc) = self.foliage_color {
            map.insert("foliage_color".to_string(), Value::Number(fc.into()));
        }
        if let Some(ref p) = self.particle {
            map.insert("particle".to_string(), p.clone());
        }
        if let Some(ref s) = self.ambient_sound {
            map.insert("ambient_sound".to_string(), Value::String(s.as_string()));
        }
        if let Some(ref ms) = self.mood_sound {
            map.insert("mood_sound".to_string(), ms.clone());
        }
        if let Some(ref ads) = self.additions_sound {
            map.insert("additions_sound".to_string(), ads.clone());
        }
        if let Some(ref music) = self.music {
            map.insert("music".to_string(), music.clone());
        }
        Value::Object(map)
    }

    fn validate(&self, location: &ResourceLocation, path: &str) -> SandResult<()> {
        validation::require_u32_in_range(
            location,
            KIND,
            &format!("{path}.fog_color"),
            self.fog_color,
            0,
            MAX_RGB_COLOR,
        )?;
        validation::require_u32_in_range(
            location,
            KIND,
            &format!("{path}.water_color"),
            self.water_color,
            0,
            MAX_RGB_COLOR,
        )?;
        validation::require_u32_in_range(
            location,
            KIND,
            &format!("{path}.water_fog_color"),
            self.water_fog_color,
            0,
            MAX_RGB_COLOR,
        )?;
        validation::require_u32_in_range(
            location,
            KIND,
            &format!("{path}.sky_color"),
            self.sky_color,
            0,
            MAX_RGB_COLOR,
        )?;
        if let Some(gc) = self.grass_color {
            validation::require_u32_in_range(
                location,
                KIND,
                &format!("{path}.grass_color"),
                gc,
                0,
                MAX_RGB_COLOR,
            )?;
        }
        if let Some(fc) = self.foliage_color {
            validation::require_u32_in_range(
                location,
                KIND,
                &format!("{path}.foliage_color"),
                fc,
                0,
                MAX_RGB_COLOR,
            )?;
        }
        if let Some(AmbientSoundReference::Raw(ref sound)) = self.ambient_sound {
            validation::validate_resource_location_str(
                location,
                KIND,
                &format!("{path}.ambient_sound"),
                sound,
            )?;
        }
        Ok(())
    }
}

// ── CarvingStep ──────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::CarvingStep",
    aliases = ["sand::prelude::CarvingStep"],
    module = "sand::component",
    summary = "A vanilla carving step. Biomes group configured carvers by the step in which they run.",
    context = "A vanilla carving step. Biomes group configured carvers by the step in which they run. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::CarvingStep;",
    variants(Air = "Carvers that run before surface decoration (most caves and ravines).", Liquid = "Carvers that run after surface decoration and only affect liquids (underwater caves)."),
)]
/// A vanilla carving step. Biomes group configured carvers by the step in
/// which they run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CarvingStep {
    /// Carvers that run before surface decoration (most caves and ravines).
    Air,
    /// Carvers that run after surface decoration and only affect liquids
    /// (underwater caves).
    Liquid,
}

impl CarvingStep {
    /// The vanilla lowercase key written into biome JSON (`"air"`/`"liquid"`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::CarvingStep::as_str",
        aliases = ["sand::prelude::CarvingStep::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "The vanilla lowercase key written into biome JSON (`\"air\"`/`\"liquid\"`).",
        context = "The vanilla lowercase key written into biome JSON (`\"air\"`/`\"liquid\"`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the vanilla lowercase key written into biome JSON (`\"air\"`/`\"liquid\"`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(carving_step_value: &sand::component::CarvingStep)  {\n    let as_str = carving_step_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Air => "air",
            Self::Liquid => "liquid",
        }
    }
}

// ── Biome ─────────────────────────────────────────────────────────────────────

/// A biome definition (`data/<namespace>/worldgen/biome/<id>.json`).
///
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Biome",
    module = "sand::component",
    summary = "A biome definition (`data/<namespace>/worldgen/biome/<id>.json`).",
    context = "A biome definition (`data/<namespace>/worldgen/biome/<id>.json`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Biome;",
)]
pub struct Biome {
    location: ResourceLocation,
    /// Whether it rains (false = snows if cold enough).
    has_precipitation: bool,
    /// Temperature used for mob spawning and weather (typical range -0.5–2.0).
    temperature: f32,
    /// Temperature modifier.
    temperature_modifier: TemperatureModifierValue,
    /// Downfall (0.0–1.0) — affects rain and snow frequency.
    downfall: f32,
    /// Visual and audio effects for this biome.
    effects: BiomeEffects,
    /// Carvers as raw JSON (explicit escape hatch; see [`Biome::raw_carvers`]).
    carvers: Option<Value>,
    /// Carvers referenced by typed ID, grouped by carving step (see
    /// [`Biome::carver_step`]). Mutually exclusive with `carvers`.
    typed_carvers: BTreeMap<CarvingStep, Vec<ConfiguredCarverId>>,
    /// Features (raw JSON array of arrays, or a typed per-step map, optional).
    features: Option<FeaturesValue>,
    /// Typed or explicitly raw per-category spawn tables.
    spawners: Option<SpawnValue<BTreeMap<MobCategory, Vec<SpawnEntry>>>>,
    /// Typed or explicitly raw per-entity population costs.
    spawn_costs: Option<SpawnValue<BTreeMap<String, SpawnCost>>>,
}

impl Biome {
    /// Creates a new biome with required base fields.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::new",
        module = "sand::component",
        kind = "method",
        summary = "Creates a new biome with required base fields.",
        context = "Creates a new biome with required base fields. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new biome with required base fields.", effects = "`effects` is used when creating a new biome with required base fields."),
        returns = "A `Biome` representing a new biome with required base fields.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, effects: sand::component::BiomeEffects)  {\n    let biome = sand::component::Biome::new(location, effects);\n}",
    )]
    pub fn new(location: ResourceLocation, effects: BiomeEffects) -> Self {
        Self {
            location,
            has_precipitation: true,
            temperature: 0.5,
            temperature_modifier: TemperatureModifierValue::Typed(TemperatureModifier::None),
            downfall: 0.5,
            effects,
            carvers: None,
            typed_carvers: BTreeMap::new(),
            features: None,
            spawners: None,
            spawn_costs: None,
        }
    }

    /// Sets whether the biome has precipitation (rain/snow).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::has_precipitation",
        module = "sand::component",
        kind = "method",
        summary = "Sets whether the biome has precipitation (rain/snow).",
        context = "Sets whether the biome has precipitation (rain/snow). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set whether the biome has precipitation (rain/snow)."),
        returns = "The `Biome` value with the documented change applied to set whether the biome has precipitation (rain/snow).",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, v: bool)  {\n    let updated_biome = biome_value.has_precipitation(v);\n}",
    )]
    pub fn has_precipitation(mut self, v: bool) -> Self {
        self.has_precipitation = v;
        self
    }

    /// Sets the biome temperature.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::temperature",
        module = "sand::component",
        kind = "method",
        summary = "Sets the biome temperature.",
        context = "Sets the biome temperature. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(temp = "`temp` provides the temp applied when setting the biome temperature."),
        returns = "The `Biome` value with the documented change applied to set the biome temperature.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, temp: f32)  {\n    let updated_biome = biome_value.temperature(temp);\n}",
    )]
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Sets the temperature modifier to a typed [`TemperatureModifier`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::temperature_modifier",
        module = "sand::component",
        kind = "method",
        summary = "Sets the temperature modifier to a typed [`TemperatureModifier`].",
        context = "Sets the temperature modifier to a typed [`TemperatureModifier`]. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(modifier = "`modifier` provides the modifier applied when setting the temperature modifier to a typed [`TemperatureModifier`]."),
        returns = "The `Biome` value with the documented change applied to set the temperature modifier to a typed [`TemperatureModifier`].",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, modifier: sand::component::TemperatureModifier)  {\n    let updated_biome = biome_value.temperature_modifier(modifier);\n}",
    )]
    pub fn temperature_modifier(mut self, modifier: TemperatureModifier) -> Self {
        self.temperature_modifier = TemperatureModifierValue::Typed(modifier);
        self
    }

    /// Sets the temperature modifier through the explicit raw compatibility
    /// path.
    ///
    /// Prefer [`Biome::temperature_modifier`] with a [`TemperatureModifier`].
    /// Vanilla currently only accepts `"none"` and `"frozen"`; this escape
    /// hatch is retained in case a future Minecraft version adds more
    /// accepted values before Sand's typed enum is updated, but export-time
    /// validation still rejects anything else today.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::raw_temperature_modifier",
        module = "sand::component",
        kind = "method",
        summary = "Sets the temperature modifier through the explicit raw compatibility path.",
        context = "Sets the temperature modifier through the explicit raw compatibility path. Prefer [`Biome::temperature_modifier`] with a [`TemperatureModifier`]. Vanilla currently only accepts `\"none\"` and `\"frozen\"`; this escape hatch is retained in case a future Minecraft version adds more accepted values before Sand's typed enum is updated, but export-time validation still rejects anything else today.",
        minecraft = "Prefer [`Biome::temperature_modifier`] with a [`TemperatureModifier`]. Vanilla currently only accepts `\"none\"` and `\"frozen\"`; this escape hatch is retained in case a future Minecraft version adds more accepted values before Sand's typed enum is updated, but export-time validation still rejects anything else today.",
        use_when = ["Prefer [`Biome::temperature_modifier`] with a [`TemperatureModifier`]. Vanilla currently only accepts `\"none\"` and `\"frozen\"`; this escape hatch is retained in case a future Minecraft version adds more accepted values before Sand's typed enum is updated, but export-time validation still rejects anything else today."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(modifier = "`modifier` provides the modifier applied when setting the temperature modifier through the explicit raw compatibility path."),
        returns = "The `Biome` value with the documented change applied to set the temperature modifier through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, modifier: impl Into < String >)  {\n    let updated_biome = biome_value.raw_temperature_modifier(modifier);\n}",
    )]
    pub fn raw_temperature_modifier(mut self, modifier: impl Into<String>) -> Self {
        self.temperature_modifier = TemperatureModifierValue::Raw(modifier.into());
        self
    }

    /// Sets the downfall value (0.0–1.0).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::downfall",
        module = "sand::component",
        kind = "method",
        summary = "Sets the downfall value (0.0–1.0).",
        context = "Sets the downfall value (0.0–1.0). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(downfall = "`downfall` provides the downfall applied when setting the downfall value (0.0–1.0)."),
        returns = "The `Biome` value with the documented change applied to set the downfall value (0.0–1.0).",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, downfall: f32)  {\n    let updated_biome = biome_value.downfall(downfall);\n}",
    )]
    pub fn downfall(mut self, downfall: f32) -> Self {
        self.downfall = downfall;
        self
    }

    /// Sets the carvers list through the explicit raw JSON escape hatch.
    ///
    /// Prefer [`Biome::carver_step`] with a typed
    /// [`ConfiguredCarverId`] (obtained
    /// from [`crate::worldgen::ConfiguredCarver::id`]) on the normal path.
    /// This escape hatch exists for modded carver references or shapes
    /// outside the typed carving-step map. Mutually exclusive with
    /// [`Biome::carver_step`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::raw_carvers",
        module = "sand::component",
        kind = "method",
        summary = "Sets the carvers list through the explicit raw JSON escape hatch.",
        context = "Sets the carvers list through the explicit raw JSON escape hatch. Prefer [`Biome::carver_step`] with a typed [`ConfiguredCarverId`] (obtained from [`sand::component::ConfiguredCarver::id`]) on the normal path. This escape hatch exists for modded carver references or shapes outside the typed carving-step map. Mutually exclusive with [`Biome::carver_step`].",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Prefer [`Biome::carver_step`] with a typed [`ConfiguredCarverId`] (obtained from [`sand::component::ConfiguredCarver::id`]) on the normal path. This escape hatch exists for modded carver references or shapes outside the typed carving-step map. Mutually exclusive with [`Biome::carver_step`]."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(carvers = "`carvers` provides the carvers applied when setting the carvers list through the explicit raw JSON escape hatch."),
        returns = "The `Biome` value with the documented change applied to set the carvers list through the explicit raw JSON escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, carvers: sand::component::RawJson)  {\n    let updated_biome = biome_value.raw_carvers(carvers);\n}",
    )]
    pub fn raw_carvers(mut self, carvers: RawJson) -> Self {
        self.carvers = Some(carvers.into_value());
        self
    }

    /// References a configured carver by typed ID under the given carving
    /// step (`air` or `liquid`), preserving vanilla's step-grouped map/array
    /// shape (`{"air": [...], "liquid": [...]}`).
    ///
    /// Author the referenced carver with
    /// [`ConfiguredCarver`](crate::worldgen::ConfiguredCarver) and pass
    /// [`ConfiguredCarver::id`](crate::worldgen::ConfiguredCarver::id) here.
    /// Mutually exclusive with [`Biome::raw_carvers`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::carver_step",
        module = "sand::component",
        kind = "method",
        summary = "References a configured carver by typed ID under the given carving step (`air` or `liquid`), preserving vanilla's step-grouped map/array shape (`{\"air\": [...], \"liquid\": [...]}`).",
        context = "References a configured carver by typed ID under the given carving step (`air` or `liquid`), preserving vanilla's step-grouped map/array shape (`{\"air\": [...], \"liquid\": [...]}`). Author the referenced carver with [`ConfiguredCarver`](sand::component::ConfiguredCarver) and pass [`ConfiguredCarver::id`](sand::component::ConfiguredCarver::id) here. Mutually exclusive with [`Biome::raw_carvers`].",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(step = "`step` is used to reference a configured carver by typed ID under the given carving step (`air` or `liquid`), preserving vanilla's step-grouped map/array shape (`{\"air\": [...], \"liquid\": [...]}`).", carver = "`carver` provides the typed Minecraft resource identifier used to reference a configured carver by typed ID under the given carving step (`air` or `liquid`), preserving vanilla's step-grouped map/array shape (`{\"air\": [...], \"liquid\": [...]}`)."),
        returns = "The `Biome` value with the documented change applied to reference a configured carver by typed ID under the given carving step (`air` or `liquid`), preserving vanilla's step-grouped map/array shape (`{\"air\": [...], \"liquid\": [...]}`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, step: sand::component::CarvingStep, carver: sand::registry::ConfiguredCarverId)  {\n    let updated_biome = biome_value.carver_step(step, carver);\n}",
    )]
    pub fn carver_step(mut self, step: CarvingStep, carver: ConfiguredCarverId) -> Self {
        self.typed_carvers.entry(step).or_default().push(carver);
        self
    }

    /// Adds a typed configured-feature reference to the given
    /// [`GenerationStep`]'s bucket of the `features` list-of-lists.
    ///
    /// Repeated calls append to the same step and accumulate across steps.
    /// If [`Biome::raw_features`] was used previously, this replaces it with
    /// a fresh typed feature map (typed and raw features are not merged).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::feature",
        module = "sand::component",
        kind = "method",
        summary = "Adds a typed configured-feature reference to the given [`GenerationStep`]'s bucket of the `features` list-of-lists.",
        context = "Adds a typed configured-feature reference to the given [`GenerationStep`]'s bucket of the `features` list-of-lists. Repeated calls append to the same step and accumulate across steps. If [`Biome::raw_features`] was used previously, this replaces it with a fresh typed feature map (typed and raw features are not merged).",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(step = "`step` provides the step added when building a typed configured-feature reference to the given [`GenerationStep`]'s bucket of the `features` list-of-lists.", feature = "`feature` provides the typed Minecraft resource identifier used to add a typed configured-feature reference to the given [`GenerationStep`]'s bucket of the `features` list-of-lists."),
        returns = "The `Biome` value with the documented change applied to add a typed configured-feature reference to the given [`GenerationStep`]'s bucket of the `features` list-of-lists.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, step: sand::component::GenerationStep, feature: sand::registry::ConfiguredFeatureId)  {\n    let updated_biome = biome_value.feature(step, feature);\n}",
    )]
    pub fn feature(mut self, step: GenerationStep, feature: ConfiguredFeatureId) -> Self {
        let mut steps = match self.features {
            Some(FeaturesValue::Typed(steps)) => steps,
            _ => vec![Vec::new(); GENERATION_STEP_COUNT],
        };
        steps[step.index()].push(feature);
        self.features = Some(FeaturesValue::Typed(steps));
        self
    }

    /// Sets the features list-of-lists through the explicit raw
    /// compatibility path.
    ///
    /// Prefer [`Biome::feature`] with a [`GenerationStep`] and
    /// [`ConfiguredFeatureId`]. This escape hatch exists for modded or
    /// version-specific feature shapes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::raw_features",
        module = "sand::component",
        kind = "method",
        summary = "Sets the features list-of-lists through the explicit raw compatibility path.",
        context = "Sets the features list-of-lists through the explicit raw compatibility path. Prefer [`Biome::feature`] with a [`GenerationStep`] and [`ConfiguredFeatureId`]. This escape hatch exists for modded or version-specific feature shapes.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Prefer [`Biome::feature`] with a [`GenerationStep`] and [`ConfiguredFeatureId`]. This escape hatch exists for modded or version-specific feature shapes."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(features = "`features` provides the features applied when setting the features list-of-lists through the explicit raw compatibility path."),
        returns = "The `Biome` value with the documented change applied to set the features list-of-lists through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, features: sand::component::RawJson)  {\n    let updated_biome = biome_value.raw_features(features);\n}",
    )]
    pub fn raw_features(mut self, features: RawJson) -> Self {
        self.features = Some(FeaturesValue::Raw(features.into_value()));
        self
    }

    /// Replaces all per-category spawn tables using canonical typed values.
    ///
    /// Replaces the previous typed or raw table. Nested values are validated on export.
    /// Table keys are emitted deterministically; spawn-entry order is preserved.
    /// Each category's total spawn weight must fit a signed 32-bit integer.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Biome::spawners",
        module = "sand::component",
        summary = "Replaces all per-category spawn tables using canonical typed values.",
        context = "Replaces the previous typed or raw table; typed values validate before export and raw values receive object-shape validation.",
        minecraft = "Writes the biome's spawners object.",
        use_when = ["Configuring biome mob spawning without JSON"],
        avoid_when = ["Authoring a structure spawn override; use SpawnOverride"],
        params(spawners = "The replacement table."),
        returns = "This biome with the replacement table.",
        example = "fn demonstrate(biome: sand::component::Biome, table: std::collections::BTreeMap<sand::component::MobCategory, Vec<sand::component::SpawnEntry>>) { let _ = biome.spawners(table); }",
    )]
    pub fn spawners(mut self, spawners: BTreeMap<MobCategory, Vec<SpawnEntry>>) -> Self {
        self.spawners = Some(SpawnValue::Typed(spawners));
        self
    }

    /// Replaces all per-category spawn tables through an explicit raw JSON escape hatch.
    ///
    /// Replaces the previous typed or raw table. Only the outer object shape is validated.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Biome::raw_spawners",
        module = "sand::component",
        summary = "Replaces all per-category spawn tables through an explicit raw JSON escape hatch.",
        context = "Replaces the previous typed or raw table; typed values validate before export and raw values receive object-shape validation.",
        minecraft = "Writes the biome's spawners object.",
        use_when = ["Using an unsupported spawn schema"],
        avoid_when = ["The typed setter can express the desired table"],
        params(spawners = "The replacement table."),
        returns = "This biome with the replacement table.",
        example = "fn demonstrate(biome: sand::component::Biome, table: sand::component::RawJson) { let _ = biome.raw_spawners(table); }",
    )]
    pub fn raw_spawners(mut self, spawners: RawJson) -> Self {
        self.spawners = Some(SpawnValue::Raw(spawners.into_value()));
        self
    }

    /// Replaces all per-entity population-density costs using canonical typed values.
    /// Duplicate entity IDs keep the last supplied cost; keys export in sorted order.
    ///
    /// Replaces the previous typed or raw table. Nested values are validated on export.
    /// Table keys are emitted deterministically; spawn-entry order is preserved.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Biome::spawn_costs",
        module = "sand::component",
        summary = "Replaces all per-entity population-density costs using canonical typed values.",
        context = "Replaces the previous typed or raw table; typed values validate before export and raw values receive object-shape validation.",
        minecraft = "Writes the biome's spawn_costs object.",
        use_when = ["Configuring biome mob spawning without JSON"],
        avoid_when = ["Authoring a structure spawn override; use SpawnOverride"],
        params(costs = "The replacement table."),
        returns = "This biome with the replacement table.",
        example = "fn demonstrate(biome: sand::component::Biome, table: impl IntoIterator<Item = (sand::registry::EntityTypeId, sand::component::SpawnCost)>) { let _ = biome.spawn_costs(table); }",
    )]
    pub fn spawn_costs(
        mut self,
        costs: impl IntoIterator<Item = (EntityTypeId, SpawnCost)>,
    ) -> Self {
        self.spawn_costs = Some(SpawnValue::Typed(
            costs
                .into_iter()
                .map(|(id, cost)| (id.to_string(), cost))
                .collect(),
        ));
        self
    }

    /// Replaces all per-entity population-density costs through an explicit raw JSON escape hatch.
    ///
    /// Replaces the previous typed or raw table. Only the outer object shape is validated.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::Biome::raw_spawn_costs",
        module = "sand::component",
        summary = "Replaces all per-entity population-density costs through an explicit raw JSON escape hatch.",
        context = "Replaces the previous typed or raw table; typed values validate before export and raw values receive object-shape validation.",
        minecraft = "Writes the biome's spawn_costs object.",
        use_when = ["Using an unsupported spawn schema"],
        avoid_when = ["The typed setter can express the desired table"],
        params(costs = "The replacement table."),
        returns = "This biome with the replacement table.",
        example = "fn demonstrate(biome: sand::component::Biome, table: sand::component::RawJson) { let _ = biome.raw_spawn_costs(table); }",
    )]
    pub fn raw_spawn_costs(mut self, costs: RawJson) -> Self {
        self.spawn_costs = Some(SpawnValue::Raw(costs.into_value()));
        self
    }
}

impl DatapackComponent for Biome {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "has_precipitation".to_string(),
            Value::Bool(self.has_precipitation),
        );
        map.insert(
            "temperature".to_string(),
            serde_json::to_value(self.temperature).unwrap(),
        );
        map.insert(
            "temperature_modifier".to_string(),
            Value::String(self.temperature_modifier.as_string()),
        );
        map.insert(
            "downfall".to_string(),
            serde_json::to_value(self.downfall).unwrap(),
        );
        map.insert("effects".to_string(), self.effects.to_json());

        if let Some(ref v) = self.carvers {
            map.insert("carvers".to_string(), v.clone());
        } else if !self.typed_carvers.is_empty() {
            let mut carvers = serde_json::Map::new();
            for (step, ids) in &self.typed_carvers {
                let ids: Vec<Value> = ids.iter().map(|id| Value::String(id.to_string())).collect();
                carvers.insert(step.as_str().to_string(), Value::Array(ids));
            }
            map.insert("carvers".to_string(), Value::Object(carvers));
        } else {
            // Biome generation settings require this field even without carvers.
            map.insert("carvers".to_string(), serde_json::json!([]));
        }
        if let Some(ref v) = self.features {
            let json = match v {
                FeaturesValue::Typed(steps) => Value::Array(
                    steps
                        .iter()
                        .map(|step| {
                            Value::Array(
                                step.iter()
                                    .map(|id| Value::String(id.to_string()))
                                    .collect(),
                            )
                        })
                        .collect(),
                ),
                FeaturesValue::Raw(raw) => raw.clone(),
            };
            map.insert("features".to_string(), json);
        } else {
            map.insert(
                "features".to_string(),
                serde_json::json!(vec![Vec::<String>::new(); GENERATION_STEP_COUNT]),
            );
        }
        if let Some(ref v) = self.spawners {
            let value = match v {
                SpawnValue::Raw(value) => value.clone(),
                SpawnValue::Typed(table) => Value::Object(
                    table
                        .iter()
                        .map(|(key, value)| {
                            (
                                key.as_str().to_string(),
                                Value::Array(value.iter().map(SpawnEntry::to_json).collect()),
                            )
                        })
                        .collect(),
                ),
            };
            map.insert("spawners".to_string(), value);
        } else {
            map.insert("spawners".to_string(), serde_json::json!({}));
        }
        if let Some(ref v) = self.spawn_costs {
            let value = match v {
                SpawnValue::Raw(value) => value.clone(),
                SpawnValue::Typed(table) => Value::Object(
                    table
                        .iter()
                        .map(|(key, value)| (key.to_string(), value.to_json()))
                        .collect(),
                ),
            };
            map.insert("spawn_costs".to_string(), value);
        } else {
            map.insert("spawn_costs".to_string(), serde_json::json!({}));
        }

        Value::Object(map)
    }

    fn validate(&self) -> SandResult<()> {
        validation::require_finite_f32(&self.location, KIND, "temperature", self.temperature)?;
        validation::require_finite_f32(&self.location, KIND, "downfall", self.downfall)?;
        if let TemperatureModifierValue::Raw(ref modifier) = self.temperature_modifier
            && modifier != "none"
            && modifier != "frozen"
        {
            return Err(validation::error(
                &self.location,
                KIND,
                "temperature_modifier",
                &format!(
                    "invalid temperature_modifier \"{modifier}\"; expected \"none\" or \"frozen\""
                ),
            ));
        }
        self.effects.validate(&self.location, "effects")?;
        if let Some(ref v) = self.carvers {
            if !self.typed_carvers.is_empty() {
                return Err(validation::error(
                    &self.location,
                    KIND,
                    "carvers",
                    "cannot combine raw_carvers with typed carver_step entries; choose one",
                ));
            }
            validation::require_json_array(&self.location, KIND, "carvers", v)?;
        }
        if let Some(FeaturesValue::Raw(ref v)) = self.features {
            validation::require_json_array(&self.location, KIND, "features", v)?;
        }
        if let Some(ref v) = self.spawners {
            match v {
                SpawnValue::Raw(value) => {
                    validation::require_json_object(&self.location, KIND, "spawners", value)?
                }
                SpawnValue::Typed(table) => {
                    for (key, value) in table {
                        SpawnEntry::validate_table(
                            value,
                            &self.location,
                            KIND,
                            &format!("spawners.{}", key.as_str()),
                        )?;
                    }
                }
            }
        }
        if let Some(ref v) = self.spawn_costs {
            match v {
                SpawnValue::Raw(value) => {
                    validation::require_json_object(&self.location, KIND, "spawn_costs", value)?
                }
                SpawnValue::Typed(table) => {
                    for (key, value) in table {
                        value.validate(&self.location, &format!("spawn_costs.{key}"))?;
                    }
                }
            }
        }
        Ok(())
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/biome"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("my_pack", "frosted").unwrap()
    }

    fn effects() -> BiomeEffects {
        BiomeEffects::new(0xC0D8FF, 0x3F76E4, 0x050533, 0x78A7FF)
    }

    #[test]
    fn default_biome_includes_required_empty_generation_settings() {
        let biome = Biome::new(location(), effects());
        biome.validate().unwrap();
        let json = biome.to_json();
        assert_eq!(json["carvers"], serde_json::json!([]));
        assert_eq!(json["spawners"], serde_json::json!({}));
        assert_eq!(json["spawn_costs"], serde_json::json!({}));
        assert_eq!(
            json["features"],
            serde_json::json!([[], [], [], [], [], [], [], [], [], [], []])
        );
    }

    #[test]
    fn valid_biome_exports_unchanged() {
        let biome = Biome::new(location(), effects())
            .temperature(0.5)
            .downfall(0.5)
            .temperature_modifier(TemperatureModifier::Frozen)
            .raw_carvers(RawJson::new(serde_json::json!(["minecraft:cave"])))
            .raw_features(RawJson::new(serde_json::json!([["minecraft:ore_iron"]])))
            .raw_spawners(RawJson::new(serde_json::json!({"monster": []})))
            .raw_spawn_costs(RawJson::new(serde_json::json!({})));
        assert!(biome.validate().is_ok());
        let json = biome.to_json();
        assert_eq!(json["temperature_modifier"], "frozen");
    }

    #[test]
    fn non_finite_temperature_rejected_without_panic() {
        let biome = Biome::new(location(), effects()).temperature(f32::NAN);
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("temperature"), "{err}");
        // try_content must not panic even though to_json would.
        assert!(biome.try_content().is_err());
    }

    #[test]
    fn non_finite_downfall_rejected_without_panic() {
        let biome = Biome::new(location(), effects()).downfall(f32::INFINITY);
        assert!(biome.validate().is_err());
        assert!(biome.try_content().is_err());
    }

    #[test]
    fn invalid_temperature_modifier_rejected() {
        let biome = Biome::new(location(), effects()).raw_temperature_modifier("cold");
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("cold"), "{err}");
        assert!(err.contains("temperature_modifier"), "{err}");
    }

    #[test]
    fn typed_temperature_modifier_variants_round_trip() {
        let none_biome =
            Biome::new(location(), effects()).temperature_modifier(TemperatureModifier::None);
        assert!(none_biome.validate().is_ok());
        assert_eq!(none_biome.to_json()["temperature_modifier"], "none");

        let frozen_biome =
            Biome::new(location(), effects()).temperature_modifier(TemperatureModifier::Frozen);
        assert!(frozen_biome.validate().is_ok());
        assert_eq!(frozen_biome.to_json()["temperature_modifier"], "frozen");
    }

    #[test]
    fn raw_temperature_modifier_escape_hatch_still_works_for_known_values() {
        let biome = Biome::new(location(), effects()).raw_temperature_modifier("frozen");
        assert!(biome.validate().is_ok());
        assert_eq!(biome.to_json()["temperature_modifier"], "frozen");
    }

    #[test]
    fn rgb_color_above_max_rejected() {
        let biome = Biome::new(location(), BiomeEffects::new(0x0100_0000, 0, 0, 0));
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("fog_color"), "{err}");
    }

    #[test]
    fn optional_color_override_above_max_rejected() {
        let biome = Biome::new(location(), effects().grass_color(0xFFFF_FFFF));
        assert!(biome.validate().is_err());
    }

    #[test]
    fn malformed_raw_ambient_sound_rejected() {
        let biome = Biome::new(location(), effects().raw_ambient_sound(""));
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("ambient_sound"), "{err}");
    }

    #[test]
    fn typed_ambient_sound_accepted() {
        let biome = Biome::new(
            location(),
            effects().ambient_sound(SoundEventId::minecraft("ambient.cave").unwrap()),
        );
        assert!(biome.validate().is_ok());
        assert_eq!(
            biome.to_json()["effects"]["ambient_sound"],
            "minecraft:ambient.cave"
        );
    }

    #[test]
    fn raw_ambient_sound_escape_hatch_still_works() {
        let biome = Biome::new(
            location(),
            effects().raw_ambient_sound("modded:custom.ambient"),
        );
        assert!(biome.validate().is_ok());
        assert_eq!(
            biome.to_json()["effects"]["ambient_sound"],
            "modded:custom.ambient"
        );
    }

    #[test]
    fn carvers_wrong_shape_rejected() {
        let biome = Biome::new(location(), effects())
            .raw_carvers(RawJson::new(serde_json::json!({"a": 1})));
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("carvers"), "{err}");
    }

    #[test]
    fn features_wrong_shape_rejected() {
        let biome = Biome::new(location(), effects())
            .raw_features(RawJson::new(serde_json::json!({"a": 1})));
        assert!(biome.validate().is_err());
    }

    #[test]
    fn typed_features_are_placed_in_correct_generation_step_bucket() {
        let biome = Biome::new(location(), effects())
            .feature(
                GenerationStep::VegetalDecoration,
                ConfiguredFeatureId::minecraft("oak").unwrap(),
            )
            .feature(
                GenerationStep::UndergroundOres,
                ConfiguredFeatureId::minecraft("ore_iron").unwrap(),
            )
            .feature(
                GenerationStep::UndergroundOres,
                ConfiguredFeatureId::minecraft("ore_gold").unwrap(),
            );
        assert!(biome.validate().is_ok());
        let features = biome.to_json()["features"].clone();
        let steps = features.as_array().unwrap();
        assert_eq!(steps.len(), GENERATION_STEP_COUNT);
        assert_eq!(
            steps[GenerationStep::UndergroundOres.index()],
            serde_json::json!(["minecraft:ore_iron", "minecraft:ore_gold"])
        );
        assert_eq!(
            steps[GenerationStep::VegetalDecoration.index()],
            serde_json::json!(["minecraft:oak"])
        );
        assert_eq!(steps[GenerationStep::Lakes.index()], serde_json::json!([]));
    }

    #[test]
    fn raw_features_escape_hatch_still_works() {
        let biome = Biome::new(location(), effects())
            .raw_features(RawJson::new(serde_json::json!([["modded:custom_feature"]])));
        assert!(biome.validate().is_ok());
        assert_eq!(
            biome.to_json()["features"],
            serde_json::json!([["modded:custom_feature"]])
        );
    }

    #[test]
    fn spawners_wrong_shape_rejected() {
        let biome =
            Biome::new(location(), effects()).raw_spawners(RawJson::new(serde_json::json!(["a"])));
        assert!(biome.validate().is_err());
    }

    #[test]
    fn spawn_costs_wrong_shape_rejected() {
        let biome = Biome::new(location(), effects())
            .raw_spawn_costs(RawJson::new(serde_json::json!(["a"])));
        assert!(biome.validate().is_err());
    }

    #[test]
    fn raw_carvers_array_escape_hatch_still_works() {
        let biome = Biome::new(location(), effects())
            .raw_carvers(RawJson::new(serde_json::json!(["modded:custom_carver"])));
        assert!(biome.validate().is_ok());
        assert_eq!(
            biome.to_json()["carvers"],
            serde_json::json!(["modded:custom_carver"])
        );
    }

    #[test]
    fn typed_carver_step_reference_produces_step_grouped_map() {
        let carver = ConfiguredCarverId::minecraft("cave").unwrap();
        let underwater = ConfiguredCarverId::minecraft("underwater_cave").unwrap();
        let biome = Biome::new(location(), effects())
            .carver_step(CarvingStep::Air, carver)
            .carver_step(CarvingStep::Liquid, underwater);
        assert!(biome.validate().is_ok());
        assert_eq!(
            biome.to_json()["carvers"],
            serde_json::json!({
                "air": ["minecraft:cave"],
                "liquid": ["minecraft:underwater_cave"],
            })
        );
    }

    #[test]
    fn mixing_raw_carvers_and_typed_carver_step_is_rejected() {
        let biome = Biome::new(location(), effects())
            .raw_carvers(RawJson::new(serde_json::json!(["minecraft:cave"])))
            .carver_step(
                CarvingStep::Air,
                ConfiguredCarverId::minecraft("cave").unwrap(),
            );
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("carvers"), "{err}");
    }
    #[test]
    fn typed_spawn_tables_preserve_weights_groups_and_costs() {
        let zombie = EntityTypeId::minecraft("zombie").unwrap();
        let biome = Biome::new(location(), effects())
            .spawners(BTreeMap::from([
                (
                    MobCategory::Monster,
                    vec![SpawnEntry::new(zombie.clone(), 20, 1, 4)],
                ),
                (MobCategory::Creature, vec![]),
            ]))
            .spawn_costs([(zombie, SpawnCost::new(0.7, 0.15))]);
        biome.validate().unwrap();
        let json = biome.to_json();
        assert_eq!(
            json["spawners"],
            serde_json::json!({
                "creature": [],
                "monster": [{"type": "minecraft:zombie", "weight": 20, "minCount": 1, "maxCount": 4}],
            })
        );
        assert_eq!(
            json["spawn_costs"],
            serde_json::json!({
                "minecraft:zombie": {"energy_budget": 0.7, "charge": 0.15},
            })
        );
    }

    #[test]
    fn spawn_weight_totals_are_bounded_per_category() {
        let entry =
            |weight| SpawnEntry::new(EntityTypeId::minecraft("zombie").unwrap(), weight, 1, 1);
        for (first, valid) in [(i32::MAX as u32 - 1, true), (i32::MAX as u32, false)] {
            let biome = Biome::new(location(), effects()).spawners(BTreeMap::from([
                (MobCategory::Monster, vec![entry(first), entry(1)]),
                (MobCategory::Creature, vec![entry(i32::MAX as u32)]),
            ]));
            if valid {
                biome.validate().unwrap();
            } else {
                let error = biome.try_content().unwrap_err().to_string();
                assert!(error.contains("my_pack:frosted"), "{error}");
                assert!(error.contains("spawners.monster"), "{error}");
                assert!(error.contains("sum of spawn weights"), "{error}");
            }
        }
    }

    #[test]
    fn invalid_spawn_entries_name_biome_category_and_entry() {
        for (weight, min, max, field) in [
            (0, 1, 2, "weight"),
            (1, 0, 2, "minCount"),
            (1, 3, 2, "maxCount"),
            (u32::MAX, 1, 2, "weight"),
            (1, u32::MAX, u32::MAX, "minCount"),
            (1, 1, u32::MAX, "maxCount"),
        ] {
            let biome = Biome::new(location(), effects()).spawners(BTreeMap::from([(
                MobCategory::Monster,
                vec![SpawnEntry::new(
                    EntityTypeId::minecraft("zombie").unwrap(),
                    weight,
                    min,
                    max,
                )],
            )]));
            let error = biome.validate().unwrap_err().to_string();
            for context in [
                "worldgen/biome",
                "my_pack:frosted",
                &format!("spawners.monster[0].{field}"),
            ] {
                assert!(error.contains(context), "{error}");
            }
        }
    }

    #[test]
    fn spawn_costs_reject_nonfinite_numbers_before_json_serialization() {
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for (budget, charge, field) in
                [(invalid, 1.0, "energy_budget"), (1.0, invalid, "charge")]
            {
                let biome = Biome::new(location(), effects()).spawn_costs([(
                    EntityTypeId::minecraft("zombie").unwrap(),
                    SpawnCost::new(budget, charge),
                )]);
                let error = biome.validate().unwrap_err().to_string();
                assert!(
                    error.contains(&format!("spawn_costs.minecraft:zombie.{field}")),
                    "{error}"
                );
                assert!(error.contains("finite"), "{error}");
            }
        }
        // The vanilla codec accepts signed finite doubles, including zero.
        Biome::new(location(), effects())
            .spawn_costs([(
                EntityTypeId::minecraft("zombie").unwrap(),
                SpawnCost::new(-1.0, 0.0),
            )])
            .validate()
            .unwrap();
    }

    #[test]
    fn typed_and_raw_spawn_tables_replace_each_other() {
        let typed = Biome::new(location(), effects())
            .raw_spawners(RawJson::new(serde_json::json!({"custom": []})))
            .raw_spawn_costs(RawJson::new(serde_json::json!({"custom": {}})))
            .spawners(BTreeMap::new())
            .spawn_costs([]);
        typed.validate().unwrap();
        assert_eq!(typed.to_json()["spawners"], serde_json::json!({}));
        assert_eq!(typed.to_json()["spawn_costs"], serde_json::json!({}));
        let raw = typed
            .raw_spawners(RawJson::new(serde_json::json!({"custom": []})))
            .raw_spawn_costs(RawJson::new(serde_json::json!({"custom": {}})));
        raw.validate().unwrap();
        assert_eq!(raw.to_json()["spawners"], serde_json::json!({"custom": []}));
        assert_eq!(
            raw.to_json()["spawn_costs"],
            serde_json::json!({"custom": {}})
        );
    }
}
