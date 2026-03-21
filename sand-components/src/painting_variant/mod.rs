//! Builder for `data/<namespace>/painting_variant/` JSON files (Minecraft 1.21+).

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

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
