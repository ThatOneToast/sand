//! Biome builder for `data/<namespace>/worldgen/biome/<id>.json`.
//!
//! This module types the biome-effects ambient sound reference, the
//! `temperature_modifier` field, the per-generation-step `features`
//! list-of-lists, and the step-grouped `carvers` map — see
//! [`BiomeEffects::ambient_sound`], [`Biome::temperature_modifier`],
//! [`Biome::feature`], and [`Biome::carver_step`]. `spawners` and
//! `spawn_costs` remain raw `Value` fields; their typing is deferred future
//! scope (#182).

use std::collections::BTreeMap;

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
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
#[derive(Clone)]
pub struct BiomeEffects {
    /// Fog color (RGB integer, e.g. `0xC0D8FF`).
    pub fog_color: u32,
    /// Water color (RGB integer).
    pub water_color: u32,
    /// Water fog color (RGB integer).
    pub water_fog_color: u32,
    /// Sky color (RGB integer).
    pub sky_color: u32,
    /// Optional grass color override (RGB integer).
    pub grass_color: Option<u32>,
    /// Optional foliage color override (RGB integer).
    pub foliage_color: Option<u32>,
    /// Ambient particle effect (raw JSON, optional).
    pub particle: Option<Value>,
    /// Ambient sound event reference (optional).
    ambient_sound: Option<AmbientSoundReference>,
    /// Mood sound (raw JSON, optional).
    pub mood_sound: Option<Value>,
    /// Additions sound (raw JSON, optional).
    pub additions_sound: Option<Value>,
    /// Background music (raw JSON, optional).
    pub music: Option<Value>,
}

impl BiomeEffects {
    /// Creates effects with the minimum required colors.
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
    pub fn grass_color(mut self, color: u32) -> Self {
        self.grass_color = Some(color);
        self
    }

    /// Overrides the foliage color.
    pub fn foliage_color(mut self, color: u32) -> Self {
        self.foliage_color = Some(color);
        self
    }

    /// Sets the ambient particle effect as raw JSON.
    pub fn particle(mut self, particle: Value) -> Self {
        self.particle = Some(particle);
        self
    }

    /// Sets the ambient loop sound to a typed [`SoundEventId`].
    pub fn ambient_sound(mut self, sound: SoundEventId) -> Self {
        self.ambient_sound = Some(AmbientSoundReference::Typed(sound));
        self
    }

    /// Sets the ambient loop sound through the explicit raw compatibility
    /// path.
    ///
    /// Prefer [`BiomeEffects::ambient_sound`] with a [`SoundEventId`]. This
    /// escape hatch exists for modded or version-specific sound references.
    pub fn raw_ambient_sound(mut self, sound: impl Into<String>) -> Self {
        self.ambient_sound = Some(AmbientSoundReference::Raw(sound.into()));
        self
    }

    /// Sets the mood sound as raw JSON.
    pub fn mood_sound(mut self, sound: Value) -> Self {
        self.mood_sound = Some(sound);
        self
    }

    /// Sets the additions sound as raw JSON.
    pub fn additions_sound(mut self, sound: Value) -> Self {
        self.additions_sound = Some(sound);
        self
    }

    /// Sets the background music as raw JSON.
    pub fn music(mut self, music: Value) -> Self {
        self.music = Some(music);
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
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Air => "air",
            Self::Liquid => "liquid",
        }
    }
}

// ── Biome ─────────────────────────────────────────────────────────────────────

/// A biome definition (`data/<namespace>/worldgen/biome/<id>.json`).
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
    pub fn has_precipitation(mut self, v: bool) -> Self {
        self.has_precipitation = v;
        self
    }

    /// Sets the biome temperature.
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Sets the temperature modifier to a typed [`TemperatureModifier`].
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
    pub fn raw_temperature_modifier(mut self, modifier: impl Into<String>) -> Self {
        self.temperature_modifier = TemperatureModifierValue::Raw(modifier.into());
        self
    }

    /// Sets the downfall value (0.0–1.0).
    pub fn downfall(mut self, downfall: f32) -> Self {
        self.downfall = downfall;
        self
    }

    /// Sets the carvers list as raw JSON.
    ///
    /// Prefer [`Biome::carver_step`] with a typed
    /// [`ConfiguredCarverId`] (obtained
    /// from [`crate::worldgen::ConfiguredCarver::id`]) on the normal path.
    /// This escape hatch exists for modded carver references or shapes
    /// outside the typed carving-step map. Mutually exclusive with
    /// [`Biome::carver_step`].
    pub fn raw_carvers(mut self, carvers: Value) -> Self {
        self.carvers = Some(carvers);
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
    pub fn raw_features(mut self, features: Value) -> Self {
        self.features = Some(FeaturesValue::Raw(features));
        self
    }

    /// Sets the spawners object as raw JSON.
    pub fn spawners(mut self, spawners: Value) -> Self {
        self.spawners = Some(spawners);
        self
    }

    /// Sets the spawn costs object as raw JSON.
    pub fn spawn_costs(mut self, costs: Value) -> Self {
        self.spawn_costs = Some(costs);
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
            .raw_carvers(serde_json::json!(["minecraft:cave"]))
            .raw_features(serde_json::json!([["minecraft:ore_iron"]]))
            .spawners(serde_json::json!({"monster": []}))
            .spawn_costs(serde_json::json!({}));
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
        let biome = Biome::new(location(), effects()).raw_carvers(serde_json::json!({"a": 1}));
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("carvers"), "{err}");
    }

    #[test]
    fn features_wrong_shape_rejected() {
        let biome = Biome::new(location(), effects()).raw_features(serde_json::json!({"a": 1}));
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
            .raw_features(serde_json::json!([["modded:custom_feature"]]));
        assert!(biome.validate().is_ok());
        assert_eq!(
            biome.to_json()["features"],
            serde_json::json!([["modded:custom_feature"]])
        );
    }

    #[test]
    fn spawners_wrong_shape_rejected() {
        let biome = Biome::new(location(), effects()).spawners(serde_json::json!(["a"]));
        assert!(biome.validate().is_err());
    }

    #[test]
    fn spawn_costs_wrong_shape_rejected() {
        let biome = Biome::new(location(), effects()).spawn_costs(serde_json::json!(["a"]));
        assert!(biome.validate().is_err());
    }

    #[test]
    fn raw_carvers_array_escape_hatch_still_works() {
        let biome = Biome::new(location(), effects())
            .raw_carvers(serde_json::json!(["modded:custom_carver"]));
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
            .raw_carvers(serde_json::json!(["minecraft:cave"]))
            .carver_step(
                CarvingStep::Air,
                ConfiguredCarverId::minecraft("cave").unwrap(),
            );
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("carvers"), "{err}");
    }
}
