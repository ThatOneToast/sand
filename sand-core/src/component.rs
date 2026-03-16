use serde::Serialize;
use serde_json::Value;

use crate::resource_location::ResourceLocation;

/// Content of a datapack component, either structured JSON or raw text.
pub enum ComponentContent {
    /// Structured JSON value (for most datapack files like advancements, loot tables).
    Json(serde_json::Value),
    /// Raw text content (for `.mcfunction` files).
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

    /// Get the serialized content of this component, defaulting to JSON form.
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
    /// Convert this value into a vector of boxed datapack components.
    fn into_datapack(self) -> Vec<Box<dyn DatapackComponent>>;
}

/// A serializable record of a datapack component for output during the build process.
#[derive(Serialize)]
pub struct ComponentRecord {
    /// The namespace (e.g. `"my_pack"`).
    pub namespace: String,
    /// The component type directory (e.g. `"function"`, `"advancement"`).
    pub dir: String,
    /// The resource location path (e.g. `"my_tick"`, `"utils/helper"`).
    pub path: String,
    /// The file extension without the dot (e.g. `"mcfunction"`, `"json"`).
    pub ext: String,
    /// The serialized content of the component.
    pub content: String,
}

/// Collect all inventory-registered components and return them as a JSON string
/// for consumption by `sand build`. Called by the generated `sand_export` binary.
///
/// This function iterates through all registered:
/// - `FunctionDescriptor`s (creating `.mcfunction` files)
/// - `ComponentFactory`s (creating component JSON files)
/// - `FunctionTagDescriptor`s (grouping functions into tags)
/// - `ArmorEventDescriptor`s (creating armor event handlers)
/// - `EventDescriptor`s (creating advancement-backed events)
///
/// Returns a JSON string containing an array of `ComponentRecord` objects,
/// one per component to be written to the datapack.
pub fn export_components_json(namespace: &str) -> String {
    use crate::function::{
        ArmorEventDescriptor, ArmorEventKind, ArmorSlot, ComponentFactory, EventDescriptor,
        FunctionDescriptor, FunctionTagDescriptor,
    };
    use crate::inventory;
    use std::collections::BTreeMap;

    let mut records: Vec<ComponentRecord> = Vec::new();
    // tag_map is declared early so armor events can inject into minecraft:tick.
    let mut tag_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // ── FunctionDescriptors ───────────────────────────────────────────────────
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

    // ── ComponentFactories ────────────────────────────────────────────────────
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

    // ── FunctionTagDescriptors → fill tag_map ─────────────────────────────────
    for desc in inventory::iter::<FunctionTagDescriptor>() {
        let fn_ref = format!("{}:{}", namespace, desc.function_path);
        tag_map
            .entry(desc.tag.to_string())
            .or_default()
            .push(fn_ref);
    }

    // ── ArmorEventDescriptors ─────────────────────────────────────────────────
    let armor_events: Vec<&ArmorEventDescriptor> =
        inventory::iter::<ArmorEventDescriptor>().collect();
    if !armor_events.is_empty() {
        // 1. Generate each callback mcfunction.
        for desc in &armor_events {
            let commands = (desc.make)();
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: desc.path.to_string(),
                ext: "mcfunction".to_string(),
                content: commands.join("\n"),
            });
        }

        // 2. Group watches by (slot + item_id + custom_data_snbt) so each
        //    unique combo shares one tracking tag.
        //    Map key → (slot, item_id, custom_data_snbt, handlers)
        let mut watches: BTreeMap<
            String,
            (
                ArmorSlot,
                Option<&'static str>,
                Option<&'static str>,
                Vec<(ArmorEventKind, &'static str)>,
            ),
        > = BTreeMap::new();

        for desc in &armor_events {
            let key = {
                let mut parts = vec![desc.slot.tag_name_segment().to_string()];
                if let Some(id) = desc.item_id {
                    parts.push(sanitize_armor_tag(id));
                }
                if let Some(cd) = desc.custom_data_snbt {
                    parts.push(sanitize_armor_tag(cd));
                }
                parts.join("_")
            };
            let entry = watches.entry(key).or_insert((
                desc.slot,
                desc.item_id,
                desc.custom_data_snbt,
                Vec::new(),
            ));
            entry.3.push((desc.kind, desc.path));
        }

        // 3. Build __sand_armor_check commands.
        let mut armor_cmds: Vec<String> = Vec::new();

        for (key, (slot, item_id, custom_data, handlers)) in &watches {
            let tag = format!("__armor_{key}");
            let item_cond = build_item_cond(*slot, *item_id, *custom_data);

            // Equip/Unequip dispatches.
            for (kind, path) in handlers {
                match kind {
                    ArmorEventKind::Equip => {
                        armor_cmds.push(format!(
                            "execute as @a[tag=!{tag}] if {item_cond} run function {namespace}:{path}"
                        ));
                    }
                    ArmorEventKind::Unequip => {
                        armor_cmds.push(format!(
                            "execute as @a[tag={tag}] unless {item_cond} run function {namespace}:{path}"
                        ));
                    }
                }
            }

            // Tag update (remove then re-add if condition is met).
            armor_cmds.push(format!("tag @a remove {tag}"));
            armor_cmds.push(format!("execute as @a if {item_cond} run tag @s add {tag}"));
        }

        // 4. Register __sand_armor_check as a function.
        let armor_path = "__sand_armor_check";
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: armor_path.to_string(),
            ext: "mcfunction".to_string(),
            content: armor_cmds.join("\n"),
        });

        // 5. Inject into minecraft:tick so it runs every tick.
        tag_map
            .entry("minecraft:tick".to_string())
            .or_default()
            .push(format!("{namespace}:{armor_path}"));
    }

    // ── Finalize tag_map → records ────────────────────────────────────────────
    // (Moved after armor events so armor check can be injected into tick.)
    for (tag_rl, values) in tag_map {
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

    // ── EventDescriptors ──────────────────────────────────────────────────────
    for desc in inventory::iter::<EventDescriptor>() {
        let advancement_id = desc
            .id_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}:{}", namespace, desc.path));

        let mut commands = (desc.make)();
        if desc.revoke {
            commands.insert(0, format!("advancement revoke @s only {}", advancement_id));
        }
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: desc.path.to_string(),
            ext: "mcfunction".to_string(),
            content: commands.join("\n"),
        });

        let trigger = (desc.make_trigger)();
        let fn_ref = format!("{}:{}", namespace, desc.path);
        let advancement = crate::components::advancement::Advancement::new(
            advancement_id
                .parse()
                .expect("invalid advancement ID in #[event]"),
        )
        .criterion(
            "event",
            crate::components::advancement::Criterion::new(trigger),
        )
        .rewards(crate::components::advancement::AdvancementRewards::new().function(fn_ref));
        let content = serde_json::to_string_pretty(&advancement.to_json()).unwrap();
        let rl = advancement.resource_location();
        records.push(ComponentRecord {
            namespace: rl.namespace().to_string(),
            dir: advancement.component_dir().to_string(),
            path: rl.path().to_string(),
            ext: advancement.file_extension().to_string(),
            content,
        });
    }

    serde_json::to_string_pretty(&records).unwrap()
}

/// Sanitize a string for use inside an entity tag name.
///
/// Keeps only `[a-zA-Z0-9_]`, replaces everything else with `_`, and strips
/// leading/trailing underscores so the result is always a clean segment.
fn sanitize_armor_tag(s: &str) -> String {
    let raw: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    raw.trim_matches('_').to_string()
}

/// Build the `execute if items` condition for an armor slot check (Minecraft 1.20.5+).
///
/// Returns e.g. `items entity @s armor.feet minecraft:leather_boots[minecraft:custom_data={mana_boots:true}]`
fn build_item_cond(
    slot: crate::function::ArmorSlot,
    item_id: Option<&str>,
    custom_data: Option<&str>,
) -> String {
    let predicate = match (item_id, custom_data) {
        (None, _) => "*".to_string(),
        (Some(id), None) => id.to_string(),
        (Some(id), Some(cd)) => format!("{}[minecraft:custom_data={}]", id, cd),
    };
    format!("items entity @s {} {}", slot.slot_name(), predicate)
}
