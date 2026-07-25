//! Builders for `data/<namespace>/wolf_variant/` JSON files (Minecraft 1.21+).
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `wild_texture`, `tame_texture`, and `angry_texture` must be non-empty
//!   and valid plain resource locations (e.g.
//!   `"minecraft:entity/wolf/wolf"`).
//! - `biomes` must be one of the JSON shapes Minecraft actually accepts for
//!   this field: a single non-empty biome ID or tag reference (string), or a
//!   non-empty array of biome ID / tag reference strings. Empty arrays,
//!   empty strings, and any other JSON shape (object, number, bool, null,
//!   non-string array elements) are rejected.

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;
use crate::validation;

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

    /// Validates that `biomes` is a supported shape: a single biome ID/tag
    /// string, or a non-empty array of biome ID/tag strings.
    fn validate_biomes(&self) -> SandResult<()> {
        let kind = "wolf_variant";
        match &self.biomes {
            Value::String(s) => {
                validation::require_non_empty(&self.location, kind, "biomes", s)?;
                validation::validate_resource_or_tag_location_str(
                    &self.location,
                    kind,
                    "biomes",
                    s,
                )?;
            }
            Value::Array(items) => {
                validation::require_non_empty_collection(
                    &self.location,
                    kind,
                    "biomes",
                    items.len(),
                )?;
                for item in items {
                    match item {
                        Value::String(s) => {
                            validation::require_non_empty(&self.location, kind, "biomes", s)?;
                            validation::validate_resource_or_tag_location_str(
                                &self.location,
                                kind,
                                "biomes",
                                s,
                            )?;
                        }
                        other => {
                            return Err(validation::error(
                                &self.location,
                                kind,
                                "biomes",
                                &format!(
                                    "biomes array entries must be strings; received `{other}`"
                                ),
                            ));
                        }
                    }
                }
            }
            other => {
                return Err(validation::error(
                    &self.location,
                    kind,
                    "biomes",
                    &format!("biomes must be a string or an array of strings; received `{other}`"),
                ));
            }
        }
        Ok(())
    }
}

impl DatapackComponent for WolfVariant {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "wolf_variant";
        for (field, value) in [
            ("wild_texture", &self.wild_texture),
            ("tame_texture", &self.tame_texture),
            ("angry_texture", &self.angry_texture),
        ] {
            validation::require_non_empty(&self.location, kind, field, value)?;
            validation::validate_resource_location_str(&self.location, kind, field, value)?;
        }
        self.validate_biomes()?;
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "ashen").unwrap()
    }

    fn valid() -> WolfVariant {
        WolfVariant::new(rl())
            .wild_texture("minecraft:entity/wolf/wolf_ashen")
            .tame_texture("minecraft:entity/wolf/wolf_ashen_tame")
            .angry_texture("minecraft:entity/wolf/wolf_ashen_angry")
            .biome("minecraft:snowy_taiga")
    }

    #[test]
    fn valid_wolf_variant_passes_validation() {
        assert!(valid().validate().is_ok());
    }

    #[test]
    fn empty_wild_texture_is_rejected() {
        let wv = valid().wild_texture("");
        let err = wv.validate().unwrap_err();
        assert!(err.to_string().contains("wild_texture"), "{err}");
    }

    #[test]
    fn empty_tame_texture_is_rejected() {
        let wv = valid().tame_texture("");
        let err = wv.validate().unwrap_err();
        assert!(err.to_string().contains("tame_texture"), "{err}");
    }

    #[test]
    fn empty_angry_texture_is_rejected() {
        let wv = valid().angry_texture("");
        let err = wv.validate().unwrap_err();
        assert!(err.to_string().contains("angry_texture"), "{err}");
    }

    #[test]
    fn malformed_texture_is_rejected() {
        let wv = valid().wild_texture("Not Valid!");
        assert!(wv.validate().is_err());
    }

    #[test]
    fn empty_single_biome_is_rejected() {
        let wv = valid().biome("");
        assert!(wv.validate().is_err());
    }

    #[test]
    fn malformed_single_biome_is_rejected() {
        let wv = valid().biome("Not Valid!");
        assert!(wv.validate().is_err());
    }

    #[test]
    fn tag_biome_is_accepted() {
        let wv = valid().biome("#minecraft:is_forest");
        assert!(wv.validate().is_ok());
    }

    #[test]
    fn empty_biomes_array_is_rejected() {
        let wv = valid().biomes(Vec::<String>::new());
        let err = wv.validate().unwrap_err();
        assert!(err.to_string().contains("biomes"), "{err}");
    }

    #[test]
    fn biomes_array_with_malformed_entry_is_rejected() {
        let wv = valid().biomes(["minecraft:plains", "Not Valid!"]);
        assert!(wv.validate().is_err());
    }

    #[test]
    fn biomes_array_of_valid_entries_is_accepted() {
        let wv = valid().biomes(["minecraft:plains", "#minecraft:is_forest"]);
        assert!(wv.validate().is_ok());
    }

    #[test]
    fn biomes_raw_non_string_array_entry_is_rejected() {
        let wv = valid().biomes_raw(serde_json::json!(["minecraft:plains", 5]));
        assert!(wv.validate().is_err());
    }

    #[test]
    fn biomes_raw_object_shape_is_rejected() {
        let wv = valid().biomes_raw(serde_json::json!({"not": "supported"}));
        assert!(wv.validate().is_err());
    }

    #[test]
    fn biomes_raw_number_shape_is_rejected() {
        let wv = valid().biomes_raw(serde_json::json!(5));
        assert!(wv.validate().is_err());
    }

    #[test]
    fn biomes_raw_null_shape_is_rejected() {
        let wv = valid().biomes_raw(Value::Null);
        assert!(wv.validate().is_err());
    }

    #[test]
    fn invalid_wolf_variant_fails_export() {
        let wv = WolfVariant::new(rl());
        assert!(wv.try_content().is_err());
    }

    #[test]
    fn valid_wolf_variant_json_is_stable() {
        let wv = valid();
        let json = wv.to_json();
        assert_eq!(json["wild_texture"], "minecraft:entity/wolf/wolf_ashen");
        assert_eq!(
            json["tame_texture"],
            "minecraft:entity/wolf/wolf_ashen_tame"
        );
        assert_eq!(
            json["angry_texture"],
            "minecraft:entity/wolf/wolf_ashen_angry"
        );
        assert_eq!(json["biomes"], "minecraft:snowy_taiga");
        let a = serde_json::to_string_pretty(&wv.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&wv.to_json()).unwrap();
        assert_eq!(a, b);
    }
}
