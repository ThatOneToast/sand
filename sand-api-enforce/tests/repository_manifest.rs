use std::path::PathBuf;

use sand_api_enforce::{ScopeManifest, ScopeState, SurfaceProfileManifest};

#[test]
fn repository_surface_manifest_records_the_audited_pending_baseline() {
    let manifest = ScopeManifest::from_path(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sand/api-scopes.toml"),
    )
    .unwrap();

    // Version-dependent aggregate values live only in the exact generated
    // surface profiles and are applied by the facade build after provider
    // version selection.
    assert_eq!(manifest.static_surface_items, 0);
    assert_eq!(manifest.pending_item_ceiling, 0);
    assert_eq!(manifest.pending_scope_ceiling, 10);
    assert_eq!(manifest.scopes.len(), 39);
    assert!(
        manifest
            .scopes
            .iter()
            .filter(|scope| scope.state == ScopeState::Enforced)
            .map(|scope| scope.id.as_str())
            .eq([
                "root-source",
                "event-source",
                "events-source",
                "item-source",
                "inventory-source",
                "predicate-source",
                "generated-predicate-id-wrapper",
                "condition-source",
                "execute-when-source",
                "resource-ref-source",
                "version-source",
                "vfx-source",
                "advanced-source",
                "generated-commands",
                "generated-vanilla-registries",
                "generated-resource-id-wrappers",
                "generated-registry-id-wrappers",
                "generated-effect-registry-enums",
                "generated-event-markers",
                "generated-function-macro",
                "generated-component-macro",
                "generated-event-macro",
                "generated-item-macro",
                "generated-armor-event-macro",
                "generated-schedule-macro",
                "generated-state-derive",
                "generated-storage-derive",
                "generated-resourcepack-macros",
                "generated-entity-state-enum-derive",
            ])
    );
    assert!(
        manifest
            .scopes
            .iter()
            .any(|scope| scope.id == "generated-commands")
    );
    assert!(
        manifest
            .scopes
            .iter()
            .any(|scope| scope.id == "prelude-unassigned-source")
    );

    let profiles = SurfaceProfileManifest::from_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sand/api-surface-profiles.toml"),
    )
    .unwrap();
    let exact_profiles = profiles
        .profiles
        .iter()
        .map(|profile| {
            (
                profile.minecraft_version.as_str(),
                profile.static_surface_items,
                profile.pending_item_ceiling,
                profile.baseline.to_string_lossy().into_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        exact_profiles,
        [
            (
                "placeholder-codegen",
                5_051,
                4_085,
                "api-surface-baseline-placeholder.txt".to_owned(),
            ),
            (
                "1.21.4",
                10_263,
                4_085,
                "api-surface-baseline-1.21.4.txt".to_owned(),
            ),
            ("26.2", 11_173, 4_085, "api-surface-baseline.txt".to_owned(),),
        ]
    );
}
