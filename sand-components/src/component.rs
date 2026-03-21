use serde_json::Value;

use crate::resource_location::ResourceLocation;

/// Content of a datapack component — structured JSON or raw text.
pub enum ComponentContent {
    /// Structured JSON value (advancements, loot tables, recipes, etc.).
    Json(Value),
    /// Raw text content (for `.mcfunction` files).
    Text(String),
}

/// A value that can be written as a file into a Minecraft datapack.
///
/// Implementors represent datapack elements such as advancements, recipes,
/// loot tables, predicates, and item modifiers. Each component knows its
/// resource location and can serialize itself to the format Minecraft expects.
pub trait DatapackComponent {
    /// The resource location that identifies this component within the datapack.
    fn resource_location(&self) -> &ResourceLocation;

    /// Serialize this component to the JSON value written to disk.
    fn to_json(&self) -> Value;

    /// Get the serialized content of this component (defaults to JSON).
    fn content(&self) -> ComponentContent {
        ComponentContent::Json(self.to_json())
    }

    /// The subdirectory under `data/<namespace>/` where this component lives.
    ///
    /// Examples: `"advancement"`, `"loot_table"`, `"recipe"`, `"predicate"`,
    /// `"item_modifier"`, `"tags"`.
    fn component_dir(&self) -> &'static str;

    /// The file extension for this component (without the dot). Defaults to `"json"`.
    fn file_extension(&self) -> &'static str {
        "json"
    }
}

/// A type that can produce a collection of [`DatapackComponent`]s.
pub trait IntoDatapack {
    fn into_datapack(self) -> Vec<Box<dyn DatapackComponent>>;
}
