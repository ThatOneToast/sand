//! Builder for `data/<namespace>/banner_pattern/` JSON files (Minecraft 1.21+).

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

/// A banner pattern definition (`data/<namespace>/banner_pattern/<id>.json`).
///
/// Banner patterns define custom designs that can be applied to banners and shields
/// using a loom. Each pattern requires a translation key for its display name.
pub struct BannerPattern {
    location: ResourceLocation,
    /// The asset ID of the texture for this banner pattern
    /// (e.g. `"minecraft:diagonal_left"`).
    asset_id: String,
    /// Translation key used for the pattern's display name in the UI
    /// (e.g. `"block.minecraft.banner.diagonal_left"`).
    translation_key: String,
}

impl BannerPattern {
    /// Creates a new banner pattern with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            asset_id: String::new(),
            translation_key: String::new(),
        }
    }

    /// Sets the asset ID (texture reference) for this banner pattern.
    pub fn asset_id(mut self, id: impl Into<String>) -> Self {
        self.asset_id = id.into();
        self
    }

    /// Sets the translation key for the banner pattern's display name.
    pub fn translation_key(mut self, key: impl Into<String>) -> Self {
        self.translation_key = key.into();
        self
    }
}

impl DatapackComponent for BannerPattern {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "asset_id": self.asset_id,
            "translation_key": self.translation_key,
        })
    }

    fn component_dir(&self) -> &'static str {
        "banner_pattern"
    }
}
