//! Maintenance generator for the offline, target-specific vanilla catalogs.
use std::collections::BTreeMap;
use std::path::Path;

use crate::{Result, cache, download, fingerprint::hash_bytes, report, resolve_version};
use serde_json::{Value, json};

/// Regenerate offline registry catalogs from verified Mojang server reports.
///
/// This is an explicit maintenance operation: it may download pinned server
/// JARs and run their data generators. Normal compilation only reads the
/// committed result and performs no network or cache I/O. Each entry includes
/// the exact report fingerprint; names use the same conversion as Rust codegen.
pub fn refresh_vanilla_catalog(versions: &[String], output: &Path) -> Result<()> {
    let mut targets = BTreeMap::new();
    for version in versions {
        let root = cache::cache_dir()?;
        let entry = resolve_version(&root, version, &download::Http)?;
        let cache = cache::VersionCache::acquire(&root, &entry.id)?;
        let jar = download::ensure_server_jar(&cache, &entry, &download::Http)?;
        let reports = report::ensure_reports(&cache, &jar)?;
        let bytes = std::fs::read(reports.join("registries.json"))?;
        let report: Value = serde_json::from_slice(&bytes)?;
        let mut registries = BTreeMap::new();
        for kind in [
            "item",
            "block",
            "entity_type",
            "sound_event",
            "particle_type",
            "attribute",
            "mob_effect",
            "potion",
        ] {
            let name = format!("minecraft:{kind}");
            let entries = report[&name]["entries"]
                .as_object()
                .ok_or_else(|| cache::invalid(format!("{version}: missing registry {name}")))?;
            let mut variants = BTreeMap::new();
            for id in entries.keys() {
                let variant = crate::codegen::registries::resource_key_to_variant(id);
                if variants.insert(variant.clone(), id.clone()).is_some() {
                    return Err(cache::invalid(format!(
                        "{version}: colliding variant {variant} in {name}"
                    )));
                }
            }
            registries.insert(name, variants);
        }
        targets.insert(
            entry.id.clone(),
            json!({"registries": registries, "report_sha1": hash_bytes(&bytes)}),
        );
    }
    let result = json!({"format": "sand.vanilla_catalog", "format_version": 1,
        "provenance": "Verified Mojang server data generator reports/registries.json", "targets": targets});
    std::fs::write(
        output,
        format!("{}\n", serde_json::to_string_pretty(&result)?),
    )?;
    Ok(())
}
