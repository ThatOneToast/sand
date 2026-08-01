//! Biome builder for `data/<namespace>/worldgen/biome/<id>.json`.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;
use crate::validation;

/// Maximum value for an RGB integer color field (`0xFFFFFF`).
const MAX_RGB_COLOR: u32 = 0xFF_FFFF;

const KIND: &str = "worldgen/biome";

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
    /// Ambient sound event ID (optional).
    pub ambient_sound: Option<String>,
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

    /// Sets the ambient loop sound.
    pub fn ambient_sound(mut self, sound: impl Into<String>) -> Self {
        self.ambient_sound = Some(sound.into());
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
            map.insert("ambient_sound".to_string(), Value::String(s.clone()));
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
        if let Some(ref sound) = self.ambient_sound {
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

// ── Biome ─────────────────────────────────────────────────────────────────────

/// A biome definition (`data/<namespace>/worldgen/biome/<id>.json`).
pub struct Biome {
    location: ResourceLocation,
    /// Whether it rains (false = snows if cold enough).
    has_precipitation: bool,
    /// Temperature used for mob spawning and weather (typical range -0.5–2.0).
    temperature: f32,
    /// Temperature modifier: `"none"` or `"frozen"`.
    temperature_modifier: String,
    /// Downfall (0.0–1.0) — affects rain and snow frequency.
    downfall: f32,
    /// Visual and audio effects for this biome.
    effects: BiomeEffects,
    /// Carvers (raw JSON array, optional).
    carvers: Option<Value>,
    /// Features (raw JSON array of arrays, optional).
    features: Option<Value>,
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
            temperature_modifier: "none".to_string(),
            downfall: 0.5,
            effects,
            carvers: None,
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

    /// Sets the temperature modifier (`"none"` or `"frozen"`).
    pub fn temperature_modifier(mut self, modifier: impl Into<String>) -> Self {
        self.temperature_modifier = modifier.into();
        self
    }

    /// Sets the downfall value (0.0–1.0).
    pub fn downfall(mut self, downfall: f32) -> Self {
        self.downfall = downfall;
        self
    }

    /// Sets the carvers list as raw JSON.
    pub fn carvers(mut self, carvers: Value) -> Self {
        self.carvers = Some(carvers);
        self
    }

    /// Sets the features list-of-lists as raw JSON.
    pub fn features(mut self, features: Value) -> Self {
        self.features = Some(features);
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
            Value::String(self.temperature_modifier.clone()),
        );
        map.insert(
            "downfall".to_string(),
            serde_json::to_value(self.downfall).unwrap(),
        );
        map.insert("effects".to_string(), self.effects.to_json());

        if let Some(ref v) = self.carvers {
            map.insert("carvers".to_string(), v.clone());
        }
        if let Some(ref v) = self.features {
            map.insert("features".to_string(), v.clone());
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
        if self.temperature_modifier != "none" && self.temperature_modifier != "frozen" {
            return Err(validation::error(
                &self.location,
                KIND,
                "temperature_modifier",
                &format!(
                    "invalid temperature_modifier \"{}\"; expected \"none\" or \"frozen\"",
                    self.temperature_modifier
                ),
            ));
        }
        self.effects.validate(&self.location, "effects")?;
        if let Some(ref v) = self.carvers {
            validation::require_json_array(&self.location, KIND, "carvers", v)?;
        }
        if let Some(ref v) = self.features {
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
            .temperature_modifier("frozen")
            .carvers(serde_json::json!(["minecraft:cave"]))
            .features(serde_json::json!([["minecraft:ore_iron"]]))
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
        let biome = Biome::new(location(), effects()).temperature_modifier("cold");
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("cold"), "{err}");
        assert!(err.contains("temperature_modifier"), "{err}");
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
    fn malformed_ambient_sound_rejected() {
        let biome = Biome::new(location(), effects().ambient_sound(""));
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("ambient_sound"), "{err}");
    }

    #[test]
    fn well_formed_ambient_sound_accepted() {
        let biome = Biome::new(
            location(),
            effects().ambient_sound("minecraft:ambient.cave"),
        );
        assert!(biome.validate().is_ok());
    }

    #[test]
    fn carvers_wrong_shape_rejected() {
        let biome = Biome::new(location(), effects()).carvers(serde_json::json!({"a": 1}));
        let err = biome.validate().unwrap_err().to_string();
        assert!(err.contains("carvers"), "{err}");
    }

    #[test]
    fn features_wrong_shape_rejected() {
        let biome = Biome::new(location(), effects()).features(serde_json::json!({"a": 1}));
        assert!(biome.validate().is_err());
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
        let biome =
            Biome::new(location(), effects()).carvers(serde_json::json!(["modded:custom_carver"]));
        assert!(biome.validate().is_ok());
        assert_eq!(
            biome.to_json()["carvers"],
            serde_json::json!(["modded:custom_carver"])
        );
    }
}
