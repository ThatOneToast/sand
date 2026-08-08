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
    assert_eq!(manifest.pending_scope_ceiling, 39);
    assert_eq!(manifest.scopes.len(), 39);
    assert!(
        manifest
            .scopes
            .iter()
            .all(|scope| scope.state == ScopeState::Pending)
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
                5_713,
                5_713,
                "api-surface-baseline-placeholder.txt".to_owned(),
            ),
            (
                "1.21.4",
                10_925,
                10_925,
                "api-surface-baseline-1.21.4.txt".to_owned(),
            ),
            (
                "26.2",
                11_835,
                11_835,
                "api-surface-baseline.txt".to_owned(),
            ),
        ]
    );
}
