//! Builder for `data/<namespace>/painting_variant/` JSON files (Minecraft 1.21+).
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `asset_id` must be non-empty and a valid plain resource location
//!   (e.g. `"minecraft:kebab"`).
//! - `width` and `height` must be in `1..=16` (a real vanilla-documented
//!   constraint for custom painting variants).
//! - `author` and `title`, when set, must not contain control characters.
//!   This is a Sand-side defensive check, not a documented vanilla
//!   requirement.

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;
use crate::validation;

/// A painting variant definition (`data/<namespace>/painting_variant/<id>.json`).
///
/// Painting variants define the textures and dimensions used when a painting entity
/// spawns or is placed. The `asset_id` points to a resource in the resource pack
/// under `textures/painting/`.
pub struct PaintingVariant {
    location: ResourceLocation,
    /// Asset ID for the painting texture (e.g. `"minecraft:kebab"`).
    asset_id: String,
    /// Width of the painting in blocks (1–16).
    width: u32,
    /// Height of the painting in blocks (1–16).
    height: u32,
    /// Optional author display name.
    author: Option<String>,
    /// Optional painting title display name.
    title: Option<String>,
}

impl PaintingVariant {
    /// Creates a new painting variant with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            asset_id: String::new(),
            width: 1,
            height: 1,
            author: None,
            title: None,
        }
    }

    /// Sets the asset ID (texture reference) for this painting.
    pub fn asset_id(mut self, id: impl Into<String>) -> Self {
        self.asset_id = id.into();
        self
    }

    /// Sets the width of the painting in blocks (1–16).
    pub fn width(mut self, w: u32) -> Self {
        self.width = w;
        self
    }

    /// Sets the height of the painting in blocks (1–16).
    pub fn height(mut self, h: u32) -> Self {
        self.height = h;
        self
    }

    /// Convenience method to set both width and height at once.
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the author display string shown in the painting tooltip.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Sets the title display string shown in the painting tooltip.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

impl DatapackComponent for PaintingVariant {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "painting_variant";
        validation::require_non_empty(&self.location, kind, "asset_id", &self.asset_id)?;
        validation::validate_resource_location_str(
            &self.location,
            kind,
            "asset_id",
            &self.asset_id,
        )?;
        validation::require_u32_in_range(&self.location, kind, "width", self.width, 1, 16)?;
        validation::require_u32_in_range(&self.location, kind, "height", self.height, 1, 16)?;
        if let Some(ref author) = self.author {
            validation::reject_control_chars(&self.location, kind, "author", author)?;
        }
        if let Some(ref title) = self.title {
            validation::reject_control_chars(&self.location, kind, "title", title)?;
        }
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("asset_id".to_string(), Value::String(self.asset_id.clone()));
        map.insert("width".to_string(), Value::Number(self.width.into()));
        map.insert("height".to_string(), Value::Number(self.height.into()));
        if let Some(ref author) = self.author {
            map.insert("author".to_string(), Value::String(author.clone()));
        }
        if let Some(ref title) = self.title {
            map.insert("title".to_string(), Value::String(title.clone()));
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "painting_variant"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "kebab").unwrap()
    }

    fn valid() -> PaintingVariant {
        PaintingVariant::new(rl())
            .asset_id("minecraft:kebab")
            .dimensions(1, 1)
    }

    #[test]
    fn valid_painting_variant_passes_validation() {
        assert!(valid().validate().is_ok());
    }

    #[test]
    fn empty_asset_id_is_rejected() {
        let pv = PaintingVariant::new(rl()).dimensions(1, 1);
        let err = pv.validate().unwrap_err();
        assert!(err.to_string().contains("asset_id"), "{err}");
    }

    #[test]
    fn malformed_asset_id_is_rejected() {
        let pv = valid().asset_id("Not Valid!");
        assert!(pv.validate().is_err());
    }

    #[test]
    fn width_zero_is_rejected() {
        let pv = valid().width(0);
        assert!(pv.validate().is_err());
    }

    #[test]
    fn width_above_sixteen_is_rejected() {
        let pv = valid().width(17);
        assert!(pv.validate().is_err());
    }

    #[test]
    fn height_zero_is_rejected() {
        let pv = valid().height(0);
        assert!(pv.validate().is_err());
    }

    #[test]
    fn height_above_sixteen_is_rejected() {
        let pv = valid().height(17);
        assert!(pv.validate().is_err());
    }

    #[test]
    fn width_and_height_bounds_are_accepted() {
        assert!(valid().dimensions(1, 16).validate().is_ok());
        assert!(valid().dimensions(16, 1).validate().is_ok());
    }

    #[test]
    fn invalid_painting_variant_fails_export() {
        let pv = PaintingVariant::new(rl());
        assert!(pv.try_content().is_err());
    }

    #[test]
    fn valid_painting_variant_json_is_stable() {
        let pv = valid().author("Bob Ross").title("Kebab");
        let json = pv.to_json();
        assert_eq!(json["asset_id"], "minecraft:kebab");
        assert_eq!(json["width"], 1);
        assert_eq!(json["height"], 1);
        assert_eq!(json["author"], "Bob Ross");
        assert_eq!(json["title"], "Kebab");
        let a = serde_json::to_string_pretty(&pv.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&pv.to_json()).unwrap();
        assert_eq!(a, b);
    }
}
