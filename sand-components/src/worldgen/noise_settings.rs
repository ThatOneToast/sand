//! Noise settings builder for `data/<namespace>/worldgen/noise_settings/<id>.json`.
//!
//! Noise settings are complex; this builder stores most sub-fields as raw JSON
//! so users can supply the exact structure required, while providing typed
//! helpers for the most commonly adjusted top-level fields.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

/// A noise settings definition (`data/<namespace>/worldgen/noise_settings/<id>.json`).
///
/// Controls chunk generation noise: sea level, bedrock placement, ore veins, etc.
/// The underlying format is very complex so most fields are accepted as raw JSON.
pub struct NoiseSettings {
    location: ResourceLocation,
    /// Sea level in blocks (default: 63).
    sea_level: i32,
    /// Whether to disable mob generation during chunk generation.
    disable_mob_generation: bool,
    /// Whether to place aquifers.
    aquifers_enabled: bool,
    /// Whether to generate ore veins.
    ore_veins_enabled: bool,
    /// Whether to use legacy random source.
    legacy_random_source: bool,
    /// Default block (raw JSON, e.g. `{"Name":"minecraft:stone"}`).
    default_block: Value,
    /// Default fluid (raw JSON, e.g. `{"Name":"minecraft:water","Properties":{"level":"0"}}`).
    default_fluid: Value,
    /// Noise router (raw JSON — highly complex).
    noise_router: Option<Value>,
    /// Spawn target list (raw JSON array).
    spawn_target: Option<Value>,
    /// Surface rules (raw JSON).
    surface_rule: Option<Value>,
}

impl NoiseSettings {
    /// Creates a new noise settings builder with sensible overworld-like defaults.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            sea_level: 63,
            disable_mob_generation: false,
            aquifers_enabled: true,
            ore_veins_enabled: true,
            legacy_random_source: false,
            default_block: serde_json::json!({"Name": "minecraft:stone"}),
            default_fluid: serde_json::json!({"Name": "minecraft:water", "Properties": {"level": "0"}}),
            noise_router: None,
            spawn_target: None,
            surface_rule: None,
        }
    }

    /// Sets the sea level in blocks.
    pub fn sea_level(mut self, level: i32) -> Self {
        self.sea_level = level;
        self
    }

    /// Sets whether mob generation is disabled during world gen.
    pub fn disable_mob_generation(mut self, v: bool) -> Self {
        self.disable_mob_generation = v;
        self
    }

    /// Sets whether aquifers are enabled.
    pub fn aquifers_enabled(mut self, v: bool) -> Self {
        self.aquifers_enabled = v;
        self
    }

    /// Sets whether ore veins are enabled.
    pub fn ore_veins_enabled(mut self, v: bool) -> Self {
        self.ore_veins_enabled = v;
        self
    }

    /// Sets whether to use the legacy random source.
    pub fn legacy_random_source(mut self, v: bool) -> Self {
        self.legacy_random_source = v;
        self
    }

    /// Sets the default block as raw JSON.
    pub fn default_block(mut self, block: Value) -> Self {
        self.default_block = block;
        self
    }

    /// Sets the default fluid as raw JSON.
    pub fn default_fluid(mut self, fluid: Value) -> Self {
        self.default_fluid = fluid;
        self
    }

    /// Sets the noise router as raw JSON.
    pub fn noise_router(mut self, router: Value) -> Self {
        self.noise_router = Some(router);
        self
    }

    /// Sets the spawn target list as raw JSON.
    pub fn spawn_target(mut self, target: Value) -> Self {
        self.spawn_target = Some(target);
        self
    }

    /// Sets the surface rules as raw JSON.
    pub fn surface_rule(mut self, rule: Value) -> Self {
        self.surface_rule = Some(rule);
        self
    }
}

impl DatapackComponent for NoiseSettings {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "sea_level".to_string(),
            Value::Number(self.sea_level.into()),
        );
        map.insert(
            "disable_mob_generation".to_string(),
            Value::Bool(self.disable_mob_generation),
        );
        map.insert(
            "aquifers_enabled".to_string(),
            Value::Bool(self.aquifers_enabled),
        );
        map.insert(
            "ore_veins_enabled".to_string(),
            Value::Bool(self.ore_veins_enabled),
        );
        map.insert(
            "legacy_random_source".to_string(),
            Value::Bool(self.legacy_random_source),
        );
        map.insert("default_block".to_string(), self.default_block.clone());
        map.insert("default_fluid".to_string(), self.default_fluid.clone());
        if let Some(ref v) = self.noise_router {
            map.insert("noise_router".to_string(), v.clone());
        }
        if let Some(ref v) = self.spawn_target {
            map.insert("spawn_target".to_string(), v.clone());
        }
        if let Some(ref v) = self.surface_rule {
            map.insert("surface_rule".to_string(), v.clone());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/noise_settings"
    }
}
