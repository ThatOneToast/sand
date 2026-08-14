use std::collections::BTreeMap;
use std::path::Path;

use sand_api_enforce::SurfaceProfileManifest;

#[test]
fn checked_repository_surface_baseline_is_complete_and_partitioned() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let baseline =
        std::fs::read_to_string(workspace.join("sand/api-surface-baseline.txt")).unwrap();
    let lines = baseline.lines().collect::<Vec<_>>();
    assert_eq!(lines[0], "schema_version=1");
    assert_eq!(
        lines[1],
        "configuration=all-supported-features,current-target"
    );
    assert_eq!(lines[2], "minecraft_version=26.2");
    assert_eq!(lines[3], "total=10936");

    let kinds = prefixed_counts(&lines, "kind ");
    assert_eq!(kinds.values().sum::<usize>(), 10_936);
    assert_eq!(kinds["field"], 717);
    assert_eq!(kinds["variant"], 5_740);
    assert_eq!(kinds["enum"], 149);
    assert_eq!(kinds["trait"], 33);
    assert_eq!(kinds["trait_method"], 50);
    assert_eq!(kinds["attribute_macro"], 8);
    assert_eq!(kinds["derive_macro"], 3);

    let origins = prefixed_counts(&lines, "origin ");
    assert_eq!(origins.values().sum::<usize>(), 10_936);
    assert_eq!(origins["source"], 4_580);
    assert_eq!(origins["generator:generated_commands"], 1_233);
    assert_eq!(origins["generator:generated_registries"], 4_867);
    assert_eq!(origins["generator:generated_registry_ids"], 136);
    assert_eq!(origins["generator:generated_effect_registry_enums"], 95);
    assert_eq!(origins["generator:generated_event_markers"], 25);
    assert!(!origins.contains_key("generator:generated_resource_refs"));

    let scope_lines = lines
        .iter()
        .filter(|line| line.contains(" module=sand") && line.contains(" items="))
        .collect::<Vec<_>>();
    assert_eq!(scope_lines.len(), 39);
    let scoped_items = scope_lines
        .iter()
        .map(|line| numeric_field(line, "items="))
        .sum::<usize>();
    assert_eq!(scoped_items, 10_936);
    assert_eq!(
        lines.last().copied(),
        Some(
            "totals pending_scopes=2 pending_items=559 enforced_items=10377 pending_scope_ceiling=2 pending_item_ceiling=559"
        )
    );
}

#[test]
fn checked_repository_profiles_bind_exact_versioned_baselines() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let sand = workspace.join("sand");
    let profiles = SurfaceProfileManifest::from_path(&sand.join("api-surface-profiles.toml"))
        .expect("repository surface profiles must be valid");
    assert_eq!(profiles.profiles.len(), 3);

    let expected = [
        ("placeholder-codegen", 4_836, 0, 0, 559),
        ("1.21.4", 10_026, 902, 4_288, 559),
        ("26.2", 10_936, 1_233, 4_867, 559),
    ];
    for (version, total, commands, registries, pending) in expected {
        let profile = profiles
            .profiles
            .iter()
            .find(|profile| profile.minecraft_version == version)
            .unwrap();
        assert_eq!(profile.static_surface_items, total);
        assert_eq!(profile.pending_item_ceiling, pending);
        let baseline = std::fs::read_to_string(sand.join(&profile.baseline)).unwrap();
        let lines = baseline.lines().collect::<Vec<_>>();
        assert_eq!(lines[2], format!("minecraft_version={version}"));
        assert_eq!(lines[3], format!("total={total}"));
        let origins = prefixed_counts(&lines, "origin ");
        assert_eq!(origins.values().sum::<usize>(), total);
        assert_eq!(origins["source"], 4_580);
        assert_eq!(
            origins
                .get("generator:generated_commands")
                .copied()
                .unwrap_or_default(),
            commands
        );
        assert_eq!(
            origins
                .get("generator:generated_registries")
                .copied()
                .unwrap_or_default(),
            registries
        );
        assert_eq!(
            numeric_field(lines.last().unwrap(), "pending_items="),
            pending
        );
    }
}

fn prefixed_counts(lines: &[&str], prefix: &str) -> BTreeMap<String, usize> {
    lines
        .iter()
        .filter_map(|line| line.strip_prefix(prefix))
        .map(|line| {
            let (name, count) = line.rsplit_once('=').unwrap();
            (name.to_owned(), count.parse().unwrap())
        })
        .collect()
}

fn numeric_field(line: &str, field: &str) -> usize {
    line.split_whitespace()
        .find_map(|part| part.strip_prefix(field))
        .unwrap()
        .parse()
        .unwrap()
}
