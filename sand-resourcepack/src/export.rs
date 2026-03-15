use std::collections::BTreeMap;

use serde_json::Value;

use crate::component::{AssetContent, ResourcePackRecord};
use crate::descriptor::ResourcePackDescriptor;

/// Collect all inventory-registered resource pack components, merge font
/// providers that share the same output file, and return the result as a
/// JSON string for consumption by `sand build --resourcepack`.
///
/// Called by the generated `sand_resource_export` binary inside the user's
/// project.
///
/// # Font merging
///
/// Multiple [`HudBar`](crate::HudBar) and [`HudElement`](crate::HudElement)
/// registrations may target the same font file (e.g. both writing to
/// `assets/ns/font/default.json`). This function detects such cases by
/// comparing output paths and, when two JSON outputs share a path, merges
/// their `"providers"` arrays into a single file.
///
/// A warning is printed to `stderr` when duplicate unicode codepoints are
/// detected across providers in the same font file — Minecraft silently uses
/// the last definition, which is almost always a bug.
pub fn export_resourcepack_json(namespace: &str) -> String {
    // Collect all AssetOutputs from every registered component.
    // Use a BTreeMap keyed by output path so merging is deterministic.
    let mut json_map: BTreeMap<String, Vec<Value>> = BTreeMap::new(); // path → providers
    let mut copy_records: Vec<ResourcePackRecord> = Vec::new();

    for desc in inventory::iter::<ResourcePackDescriptor>() {
        let component = (desc.make)();
        for output in component.assets(namespace) {
            match output.content {
                AssetContent::Json(v) => {
                    // If this is a font file (has "providers" array), collect
                    // for merging. Otherwise write it directly.
                    if let Some(providers) = v.get("providers").and_then(Value::as_array) {
                        json_map
                            .entry(output.path)
                            .or_default()
                            .extend(providers.iter().cloned());
                    } else {
                        let serialized = serde_json::to_string_pretty(&v)
                            .expect("failed to serialize resource pack JSON");
                        copy_records.push(ResourcePackRecord {
                            path: output.path,
                            content_type: "json".to_string(),
                            content: serialized,
                        });
                    }
                }
                AssetContent::CopyFrom(src) => {
                    copy_records.push(ResourcePackRecord {
                        path: output.path,
                        content_type: "copy".to_string(),
                        content: src,
                    });
                }
                AssetContent::Bytes(bytes) => {
                    use base64::Engine as _;
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    copy_records.push(ResourcePackRecord {
                        path: output.path,
                        content_type: "bytes".to_string(),
                        content: encoded,
                    });
                }
            }
        }
    }

    // Emit merged font files, checking for duplicate codepoints.
    let mut records: Vec<ResourcePackRecord> = Vec::new();

    for (path, mut providers) in json_map {
        // Inject the space-advance provider so advance_x() characters work
        // in every font file without manual setup.
        providers.push(space_advance_provider());
        warn_duplicate_codepoints(&path, &providers);
        let merged = serde_json::json!({ "providers": providers });
        records.push(ResourcePackRecord {
            path,
            content_type: "json".to_string(),
            content: serde_json::to_string_pretty(&merged)
                .expect("failed to serialize merged font JSON"),
        });
    }

    // Append non-font records after fonts (order is cosmetic).
    records.extend(copy_records);

    serde_json::to_string_pretty(&records).expect("failed to serialize resource pack records")
}

/// Build the `space` font provider that maps Private-Use-Area characters
/// (U+F801..U+F81B) to power-of-two advance widths.
///
/// This provider is injected into every exported font file automatically so
/// that [`advance_x`](crate::advance_x) characters resolve without any
/// manual font configuration.
fn space_advance_provider() -> serde_json::Value {
    use serde_json::{Map, Value, json};

    let mut advances: Map<String, Value> = Map::new();

    // Positive advances: U+F801 = +1, U+F802 = +2, …, U+F80B = +1024.
    for bit in 0..11u32 {
        let cp = 0xF801u32 + bit;
        let advance = 1i32 << bit;
        // Key must be the actual unicode character so serde_json serializes it
        // as `\uF801`, not a double-escaped `\\uF801` which Minecraft rejects.
        if let Some(c) = char::from_u32(cp) {
            advances.insert(c.to_string(), json!(advance));
        }
    }

    // Negative advances: U+F811 = -1, U+F812 = -2, …, U+F81B = -1024.
    for bit in 0..11u32 {
        let cp = 0xF811u32 + bit;
        let advance = -(1i32 << bit);
        if let Some(c) = char::from_u32(cp) {
            advances.insert(c.to_string(), json!(advance));
        }
    }

    json!({ "type": "space", "advances": advances })
}

/// Scan all `"chars"` arrays across providers in a font file and warn when
/// the same codepoint appears more than once.
fn warn_duplicate_codepoints(font_path: &str, providers: &[Value]) {
    let mut seen: BTreeMap<char, usize> = BTreeMap::new();
    for (i, provider) in providers.iter().enumerate() {
        if let Some(rows) = provider.get("chars").and_then(Value::as_array) {
            for row in rows {
                if let Some(s) = row.as_str() {
                    for ch in s.chars() {
                        if let Some(prev) = seen.insert(ch, i) {
                            eprintln!(
                                "sand-resourcepack: warning: duplicate codepoint U+{:04X} in \
                                 '{}' (providers[{}] and providers[{}]). Minecraft will use \
                                 the last definition.",
                                ch as u32, font_path, prev, i
                            );
                        }
                    }
                }
            }
        }
    }
}
