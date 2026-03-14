use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

/// A Minecraft tag file that groups entities, items, blocks, or other objects together.
pub struct Tag {
    pub location: ResourceLocation,
    pub replace: bool,
    pub values: Vec<String>,
}

impl Tag {
    /// Creates a new Tag with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            replace: false,
            values: Vec::new(),
        }
    }

    /// Adds a single entry to this tag.
    pub fn entry(mut self, id: impl std::fmt::Display) -> Self {
        self.values.push(id.to_string());
        self
    }

    /// Adds a reference to another tag (prefixed with #).
    pub fn tag_ref(mut self, tag: impl std::fmt::Display) -> Self {
        self.values.push(format!("#{tag}"));
        self
    }

    /// Sets whether this tag should replace existing tag data.
    pub fn replace(mut self, v: bool) -> Self {
        self.replace = v;
        self
    }
}

impl DatapackComponent for Tag {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "replace": self.replace,
            "values": self.values,
        })
    }

    fn component_dir(&self) -> &'static str { "tags" }
}
