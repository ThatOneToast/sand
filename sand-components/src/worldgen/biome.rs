//! Biome builder for `data/<namespace>/worldgen/biome/<id>.json`.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

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

    fn component_dir(&self) -> &'static str {
        "worldgen/biome"
    }
}
