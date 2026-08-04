use std::path::PathBuf;

use sand_api_enforce::{ScopeManifest, ScopeState};

#[test]
fn repository_surface_manifest_records_the_audited_pending_baseline() {
    let manifest = ScopeManifest::from_path(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sand/api-scopes.toml"),
    )
    .unwrap();

    assert_eq!(manifest.static_surface_items, 11_835);
    assert_eq!(manifest.pending_item_ceiling, 11_835);
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
}
