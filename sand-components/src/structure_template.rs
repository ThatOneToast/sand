use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::StructureTemplate",
    aliases = ["sand::prelude::StructureTemplate"],
    module = "sand::component",
    summary = "A datapack structure template copied from an existing `.nbt` file.",
    context = "A datapack structure template copied from an existing `.nbt` file. Structure templates are binary NBT assets. Sand treats them as copy-backed datapack components and writes them under `data/<namespace>/structure/<path>.nbt`.",
    minecraft = "Structure templates are binary NBT assets. Sand treats them as copy-backed datapack components and writes them under `data/<namespace>/structure/<path>.nbt`.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::StructureTemplate;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureTemplate::new",
        aliases = ["sand::prelude::StructureTemplate::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a copy-backed structure template. `source_path` is relative to the project root containing `sand.toml`. The build pipeline validates that it is a safe relative `.nbt` path.",
        context = "Create a copy-backed structure template. `source_path` is relative to the project root containing `sand.toml`. The build pipeline validates that it is a safe relative `.nbt` path. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "`source_path` is relative to the project root containing `sand.toml`. The build pipeline validates that it is a safe relative `.nbt` path.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a copy-backed structure template. `source_path` is relative to the project root containing `sand.toml`. The build pipeline validates that it is a safe relative `.nbt` path.", source_path = "`source_path` is relative to the project root containing `sand.toml`. The build pipeline validates that it is a safe relative `.nbt` path."),
        returns = "A newly constructed `StructureTemplate` configured to create a copy-backed structure template. `source_path` is relative to the project root containing `sand.toml`. The build pipeline validates that it is a safe relative `.nbt` path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, source_path: impl Into < String >)  {\n    let structure_template = sand::component::StructureTemplate::new(location, source_path);\n}",
    )]
    pub fn new(location: ResourceLocation, source_path: impl Into<String>) -> Self {
        Self {
            location,
            source_path: source_path.into(),
        }
    }

    /// Return the source path that will be copied into the datapack.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureTemplate::source_path",
        aliases = ["sand::prelude::StructureTemplate::source_path"],
        module = "sand::component",
        kind = "method",
        summary = "Return the source path that will be copied into the datapack.",
        context = "Return the source path that will be copied into the datapack. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "Return the source path that will be copied into the datapack.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_template_value: &sand::component::StructureTemplate)  {\n    let source_path = structure_template_value.source_path();\n}",
    )]
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
