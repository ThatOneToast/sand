use serde::Serialize;
use serde_json::Value;

use crate::resource_location::ResourceLocation;

pub enum ComponentContent {
    Json(serde_json::Value),
    Text(String),
}

/// A value that can be written as a file into a Minecraft datapack.
///
/// Implementors represent datapack elements such as functions, advancements,
/// recipes, and loot tables. Each component knows its own resource location
/// and can serialize itself to the JSON (or text) format that Minecraft expects.
pub trait DatapackComponent {
    /// The resource location that identifies this component within the datapack
    /// (e.g. `my_pack:function/tick`).
    fn resource_location(&self) -> &ResourceLocation;

    /// Serialize this component to the JSON value that will be written to disk.
    ///
    /// For `.mcfunction` files the commands are returned as a
    /// `Value::Array` of strings rather than an object.
    fn to_json(&self) -> Value;

    fn content(&self) -> ComponentContent {
        ComponentContent::Json(self.to_json())
    }

    /// The subdirectory under `data/<namespace>/` where this component lives.
    ///
    /// Examples: `"advancement"`, `"function"`, `"loot_table"`, `"recipe"`,
    /// `"predicate"`, `"item_modifier"`, `"tags"`.
    fn component_dir(&self) -> &'static str;

    /// The file extension for this component (without the leading dot).
    ///
    /// Defaults to `"json"`. Override for `.mcfunction` files.
    fn file_extension(&self) -> &'static str {
        "json"
    }
}

/// A type that can produce a collection of [`DatapackComponent`]s ready to be
/// written into a datapack output directory.
pub trait IntoDatapack {
    fn into_datapack(self) -> Vec<Box<dyn DatapackComponent>>;
}

#[derive(Serialize)]
pub struct ComponentRecord {
    pub namespace: String,
    pub dir: String,
    pub path: String,
    pub ext: String,
    pub content: String,
}

/// Collect all inventory-registered components and return them as a JSON string
/// for consumption by `sand build`. Called by the generated `sand_export` binary.
pub fn export_components_json(namespace: &str) -> String {
    use crate::function::{ComponentFactory, FunctionDescriptor, FunctionTagDescriptor};
    use crate::inventory;
    use std::collections::BTreeMap;

    let mut records: Vec<ComponentRecord> = Vec::new();

    // #[function] annotated functions
    for desc in inventory::iter::<FunctionDescriptor>() {
        let commands = (desc.make)();
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: desc.path.to_string(),
            ext: "mcfunction".to_string(),
            content: commands.join("\n"),
        });
    }

    // #[component] annotated functions
    for factory in inventory::iter::<ComponentFactory>() {
        let comp = (factory.make)();
        let rl = comp.resource_location();
        let content = match comp.content() {
            ComponentContent::Json(v) => serde_json::to_string_pretty(&v).unwrap(),
            ComponentContent::Text(t) => t,
        };
        records.push(ComponentRecord {
            namespace: rl.namespace().to_string(),
            dir: comp.component_dir().to_string(),
            path: rl.path().to_string(),
            ext: comp.file_extension().to_string(),
            content,
        });
    }

    // #[component(Tick)] / #[component(Load)] / #[component(Tag = "...")] —
    // collect all entries grouped by tag resource location, then emit one
    // `tags/functions/<name>.json` per tag, merging multiple registrations.
    //
    // Tag RL format: "minecraft:tick" → namespace="minecraft", path="tick"
    // Output file: data/minecraft/tags/functions/tick.json
    let mut tag_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for desc in inventory::iter::<FunctionTagDescriptor>() {
        let fn_ref = format!("{}:{}", namespace, desc.function_path);
        tag_map
            .entry(desc.tag.to_string())
            .or_default()
            .push(fn_ref);
    }
    for (tag_rl, values) in tag_map {
        // Split "ns:path" into namespace and path components.
        let (tag_ns, tag_path) = match tag_rl.split_once(':') {
            Some((ns, path)) => (ns.to_string(), path.to_string()),
            None => (namespace.to_string(), tag_rl.clone()),
        };
        let json = serde_json::json!({ "values": values });
        records.push(ComponentRecord {
            namespace: tag_ns,
            dir: "tags/function".to_string(),
            path: tag_path,
            ext: "json".to_string(),
            content: serde_json::to_string_pretty(&json).unwrap(),
        });
    }

    serde_json::to_string_pretty(&records).unwrap()
}
