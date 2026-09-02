//! Biome builder for `data/<namespace>/worldgen/biome/<id>.json`.
//!
//! This module types the biome-effects ambient sound reference, the
//! `temperature_modifier` field, the per-generation-step `features`
//! list-of-lists, and the step-grouped `carvers` map — see
//! [`BiomeEffects::ambient_sound`], [`Biome::temperature_modifier`],
//! [`Biome::feature`], and [`Biome::carver_step`]. `spawners` and
//! `spawn_costs` remain raw JSON fields; their typing is deferred future
//! scope (#182).

use std::collections::BTreeMap;

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::{ConfiguredCarverId, ConfiguredFeatureId, SoundEventId};
use crate::resource_location::ResourceLocation;
use crate::validation;
use crate::worldgen::structure::GenerationStep;

/// Maximum value for an RGB integer color field (`0xFFFFFF`).
const MAX_RGB_COLOR: u32 = 0xFF_FFFF;

const KIND: &str = "worldgen/biome";

/// Number of vanilla feature-generation decoration steps — the fixed length
/// of the `features` list-of-lists array. See [`GenerationStep`].
const GENERATION_STEP_COUNT: usize = 11;

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
        params(fog_color = "`fog_color` supplies the fog color value used to create effects with the minimum required colors.", water_color = "`water_color` supplies the water color value used to create effects with the minimum required colors.", water_fog_color = "`water_fog_color` supplies the water fog color value used to create effects with the minimum required colors.", sky_color = "`sky_color` supplies the sky color value used to create effects with the minimum required colors."),
        returns = "A newly constructed `BiomeEffects` configured to create effects with the minimum required colors.",
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
        params(color = "`color` supplies the color value used to override the grass color."),
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
        params(color = "`color` supplies the color value used to override the foliage color."),
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
        params(particle = "`particle` supplies the particle value used to set the ambient particle effect through the explicit raw JSON escape hatch."),
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
        params(sound = "`sound` supplies the sound value used to set the ambient loop sound through the explicit raw compatibility path."),
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
        params(sound = "`sound` supplies the sound value used to set the mood sound through the explicit raw JSON escape hatch."),
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
        params(sound = "`sound` supplies the sound value used to set the additions sound through the explicit raw JSON escape hatch."),
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
        params(music = "`music` supplies the music value used to set the background music through the explicit raw JSON escape hatch."),
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
    /// Creature, monster, ambient spawn lists (raw JSON, optional).
    spawners: Option<Value>,
    /// Spawn costs (raw JSON, optional).
    spawn_costs: Option<Value>,
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
        params(location = "`location` provides the typed resource identifier or location used to create a new biome with required base fields.", effects = "`effects` supplies the effects value used to create a new biome with required base fields."),
        returns = "A newly constructed `Biome` configured to create a new biome with required base fields.",
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
        params(temp = "`temp` supplies the temp value used to set the biome temperature."),
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
        params(modifier = "`modifier` supplies the modifier value used to set the temperature modifier to a typed [`TemperatureModifier`]."),
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
        params(modifier = "`modifier` supplies the modifier value used to set the temperature modifier through the explicit raw compatibility path."),
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
        params(downfall = "`downfall` supplies the downfall value used to set the downfall value (0.0–1.0)."),
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
        params(carvers = "`carvers` supplies the carvers value used to set the carvers list through the explicit raw JSON escape hatch."),
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
        params(step = "`step` supplies the step value used to reference a configured carver by typed ID under the given carving step (`air` or `liquid`), preserving vanilla's step-grouped map/array shape (`{\"air\": [...], \"liquid\": [...]}`).", carver = "`carver` provides the typed Minecraft resource identifier used to reference a configured carver by typed ID under the given carving step (`air` or `liquid`), preserving vanilla's step-grouped map/array shape (`{\"air\": [...], \"liquid\": [...]}`)."),
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
        params(step = "`step` supplies the step value used to add a typed configured-feature reference to the given [`GenerationStep`]'s bucket of the `features` list-of-lists.", feature = "`feature` provides the typed Minecraft resource identifier used to add a typed configured-feature reference to the given [`GenerationStep`]'s bucket of the `features` list-of-lists."),
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
        params(features = "`features` supplies the features value used to set the features list-of-lists through the explicit raw compatibility path."),
        returns = "The `Biome` value with the documented change applied to set the features list-of-lists through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, features: sand::component::RawJson)  {\n    let updated_biome = biome_value.raw_features(features);\n}",
    )]
    pub fn raw_features(mut self, features: RawJson) -> Self {
        self.features = Some(FeaturesValue::Raw(features.into_value()));
        self
    }

    /// Sets the spawners object through the explicit raw JSON escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::spawners",
        module = "sand::component",
        kind = "method",
        summary = "Sets the spawners object through the explicit raw JSON escape hatch.",
        context = "Sets the spawners object through the explicit raw JSON escape hatch. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(spawners = "`spawners` supplies the spawners value used to set the spawners object through the explicit raw JSON escape hatch."),
        returns = "The `Biome` value with the documented change applied to set the spawners object through the explicit raw JSON escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, spawners: sand::component::RawJson)  {\n    let updated_biome = biome_value.spawners(spawners);\n}",
    )]
    pub fn spawners(mut self, spawners: RawJson) -> Self {
        self.spawners = Some(spawners.into_value());
        self
    }

    /// Sets the spawn costs object through the explicit raw JSON escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Biome::spawn_costs",
        module = "sand::component",
        kind = "method",
        summary = "Sets the spawn costs object through the explicit raw JSON escape hatch.",
        context = "Sets the spawn costs object through the explicit raw JSON escape hatch. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(costs = "`costs` supplies the costs value used to set the spawn costs object through the explicit raw JSON escape hatch."),
        returns = "The `Biome` value with the documented change applied to set the spawn costs object through the explicit raw JSON escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_value: sand::component::Biome, costs: sand::component::RawJson)  {\n    let updated_biome = biome_value.spawn_costs(costs);\n}",
    )]
    pub fn spawn_costs(mut self, costs: RawJson) -> Self {
        self.spawn_costs = Some(costs.into_value());
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
        }
        if let Some(ref v) = self.spawners {
            map.insert("spawners".to_string(), v.clone());
        }
        if let Some(ref v) = self.spawn_costs {
            map.insert("spawn_costs".to_string(), v.clone());
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
            validation::require_json_object(&self.location, KIND, "spawners", v)?;
        }
        if let Some(ref v) = self.spawn_costs {
            validation::require_json_object(&self.location, KIND, "spawn_costs", v)?;
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
    fn valid_biome_exports_unchanged() {
        let biome = Biome::new(location(), effects())
            .temperature(0.5)
            .downfall(0.5)
            .temperature_modifier(TemperatureModifier::Frozen)
            .raw_carvers(RawJson::new(serde_json::json!(["minecraft:cave"])))
            .raw_features(RawJson::new(serde_json::json!([["minecraft:ore_iron"]])))
            .spawners(RawJson::new(serde_json::json!({"monster": []})))
            .spawn_costs(RawJson::new(serde_json::json!({})));
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
            Biome::new(location(), effects()).spawners(RawJson::new(serde_json::json!(["a"])));
        assert!(biome.validate().is_err());
    }

    #[test]
    fn spawn_costs_wrong_shape_rejected() {
        let biome =
            Biome::new(location(), effects()).spawn_costs(RawJson::new(serde_json::json!(["a"])));
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
}
