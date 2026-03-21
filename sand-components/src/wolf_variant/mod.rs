//! Builders for `data/<namespace>/wolf_variant/` JSON files (Minecraft 1.21+).

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

/// A wolf variant definition (`data/<namespace>/wolf_variant/<id>.json`).
///
/// Wolf variants control the skin textures shown for wolves spawned in specific biomes.
pub struct WolfVariant {
    location: ResourceLocation,
    /// Texture path for wild (untamed) wolves.
    wild_texture: String,
    /// Texture path for tame wolves.
    tame_texture: String,
    /// Texture path for angry wolves.
    angry_texture: String,
    /// Biome(s) where this wolf variant spawns. Can be a single biome ID string
    /// or a JSON array of biome IDs / biome tags.
    biomes: Value,
}

impl WolfVariant {
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            wild_texture: String::new(),
            tame_texture: String::new(),
            angry_texture: String::new(),
            biomes: Value::Array(vec![]),
        }
    }

    /// Set the texture path for wild (untamed) wolves.
    pub fn wild_texture(mut self, path: impl Into<String>) -> Self {
        self.wild_texture = path.into();
        self
    }

    /// Set the texture path for tame wolves.
    pub fn tame_texture(mut self, path: impl Into<String>) -> Self {
        self.tame_texture = path.into();
        self
    }

    /// Set the texture path for angry wolves.
    pub fn angry_texture(mut self, path: impl Into<String>) -> Self {
        self.angry_texture = path.into();
        self
    }

    /// Set the biome this variant spawns in (single biome string).
    pub fn biome(mut self, biome_id: impl Into<String>) -> Self {
        self.biomes = Value::String(biome_id.into());
        self
    }

    /// Set multiple biomes this variant spawns in.
    pub fn biomes(mut self, biome_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.biomes = Value::Array(
            biome_ids
                .into_iter()
                .map(|s| Value::String(s.into()))
                .collect(),
        );
        self
    }

    /// Set biomes from a raw JSON value (for tags like `"#minecraft:is_forest"`).
    pub fn biomes_raw(mut self, biomes: Value) -> Self {
        self.biomes = biomes;
        self
    }
}

impl DatapackComponent for WolfVariant {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "wild_texture": self.wild_texture,
            "tame_texture": self.tame_texture,
            "angry_texture": self.angry_texture,
            "biomes": self.biomes,
        })
    }

    fn component_dir(&self) -> &'static str {
        "wolf_variant"
    }
}
