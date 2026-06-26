use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

/// A datapack structure template copied from an existing `.nbt` file.
///
/// Structure templates are binary NBT assets. Sand treats them as copy-backed
/// datapack components and writes them under
/// `data/<namespace>/structure/<path>.nbt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureTemplate {
    location: ResourceLocation,
    source_path: String,
}

impl StructureTemplate {
    /// Create a copy-backed structure template.
    ///
    /// `source_path` is relative to the project root containing `sand.toml`.
    /// The build pipeline validates that it is a safe relative `.nbt` path.
    pub fn new(location: ResourceLocation, source_path: impl Into<String>) -> Self {
        Self {
            location,
            source_path: source_path.into(),
        }
    }

    /// Return the source path that will be copied into the datapack.
    pub fn source_path(&self) -> &str {
        &self.source_path
    }
}

impl DatapackComponent for StructureTemplate {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        Value::Null
    }

    fn component_dir(&self) -> &'static str {
        "structure"
    }

    fn file_extension(&self) -> &'static str {
        "nbt"
    }

    fn copy_source_path(&self) -> Option<&str> {
        Some(&self.source_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structure_template_exports_as_copy_backed_nbt_component() {
        let template = StructureTemplate::new(
            ResourceLocation::new("example", "rooms/start").unwrap(),
            "structures/start.nbt",
        );

        assert_eq!(template.resource_location().namespace(), "example");
        assert_eq!(template.resource_location().path(), "rooms/start");
        assert_eq!(template.component_dir(), "structure");
        assert_eq!(template.file_extension(), "nbt");

        assert_eq!(template.copy_source_path(), Some("structures/start.nbt"));
    }
}
