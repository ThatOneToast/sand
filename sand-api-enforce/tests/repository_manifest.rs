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
    assert_eq!(profiles.profiles.len(), 2);
    assert_eq!(profiles.profiles[0].minecraft_version, "1.21.4");
    assert_eq!(profiles.profiles[0].static_surface_items, 10_925);
    assert_eq!(profiles.profiles[1].minecraft_version, "26.2");
    assert_eq!(profiles.profiles[1].static_surface_items, 11_835);
}
