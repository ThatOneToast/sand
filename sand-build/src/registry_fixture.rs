use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Result, VersionCacheLock, download, report, resolve_version};

const PROVENANCE: &str = "Mojang server data generator generated/reports/datapack.json";

#[derive(Debug, Deserialize)]
struct DatapackReport {
    registries: BTreeMap<String, DatapackEntry>,
    others: BTreeMap<String, DatapackEntry>,
}

#[derive(Debug, Deserialize)]
struct DatapackEntry {
    elements: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct Fixture<'a> {
    minecraft_version: &'a str,
    provenance: &'static str,
    registries: Vec<FixtureRegistry>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct FixtureRegistry {
    registry_id: String,
    datapack_dir: String,
}

fn normalize(version: &str, report: &str) -> Result<String> {
    let report: DatapackReport = serde_json::from_str(report)?;
    let mut registries: Vec<_> = report
        .registries
        .into_iter()
        .filter(|(_, entry)| entry.elements)
        .map(|(registry_id, _)| FixtureRegistry {
            datapack_dir: registry_id
                .strip_prefix("minecraft:")
                .unwrap_or(&registry_id)
                .to_string(),
            registry_id,
        })
        .collect();

    if report
        .others
        .get("function")
        .is_some_and(|entry| entry.elements)
    {
        registries.push(FixtureRegistry {
            registry_id: "minecraft:function".to_string(),
            datapack_dir: "function".to_string(),
        });
    }
    registries.sort_by(|a, b| a.registry_id.cmp(&b.registry_id));

    let fixture = Fixture {
        minecraft_version: version,
        provenance: PROVENANCE,
        registries,
    };
    Ok(format!("{}\n", serde_json::to_string_pretty(&fixture)?))
}

/// Regenerate a minimal, deterministic registry-coverage fixture.
///
/// This maintenance operation may download a server jar and invokes the
/// selected Minecraft version's data generator. It is never called by normal
/// tests or CI.
pub fn refresh_registry_coverage_fixture(version: &str, output: &Path) -> Result<()> {
    let (version_id, version_json_url) = resolve_version(version)?;
    let _lock = VersionCacheLock::acquire(&version_id)?;
    let jar = download::ensure_server_jar(&version_id, &version_json_url)?;
    let reports = report::ensure_reports(&version_id, &jar)?;
    let datapack = std::fs::read_to_string(reports.join("datapack.json"))?;
    std::fs::write(output, normalize(&version_id, &datapack)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_selects_elements_and_orders_ids() {
        let report = r#"{
            "registries": {
                "minecraft:zeta": {"elements": true},
                "minecraft:ignored": {"elements": false},
                "minecraft:alpha": {"elements": true}
            },
            "others": {"function": {"elements": true}}
        }"#;
        let normalized = normalize("test", report).unwrap();
        let alpha = normalized.find("minecraft:alpha").unwrap();
        let function = normalized.find("minecraft:function").unwrap();
        let zeta = normalized.find("minecraft:zeta").unwrap();
        assert!(alpha < function && function < zeta);
        assert!(!normalized.contains("minecraft:ignored"));
        assert!(normalized.ends_with('\n'));
    }
}
