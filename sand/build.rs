use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use sand_api_enforce::{ScopeManifest, ScopeState};

fn main() {
    println!("cargo:rerun-if-changed=api-scopes.toml");

    let manifest = ScopeManifest::from_path("api-scopes.toml")
        .unwrap_or_else(|error| panic!("invalid Sand API scope manifest: {error}"));
    let mut pending = manifest
        .scopes
        .iter()
        .filter(|scope| scope.state == ScopeState::Pending)
        .map(|scope| scope.id.as_str())
        .collect::<Vec<_>>();
    pending.sort_unstable();
    if pending.len() > manifest.pending_scope_ceiling {
        panic!(
            "Sand API pending scope count {} exceeds committed ceiling {}",
            pending.len(),
            manifest.pending_scope_ceiling
        );
    }

    let status = if pending.is_empty() {
        "Complete"
    } else {
        "Partial"
    };
    let mut generated = String::new();
    writeln!(
        generated,
        "pub fn installed_coverage() -> ApiCoverage {{ ApiCoverage {{"
    )
    .unwrap();
    writeln!(generated, "status: CoverageStatus::{status},").unwrap();
    writeln!(
        generated,
        "static_surface_items: {},",
        manifest.static_surface_items
    )
    .unwrap();
    writeln!(
        generated,
        "pending_item_ceiling: {},",
        manifest.pending_item_ceiling
    )
    .unwrap();
    writeln!(
        generated,
        "pending_scope_ceiling: {},",
        manifest.pending_scope_ceiling
    )
    .unwrap();
    generated.push_str("pending_scopes: vec![\n");
    for id in pending {
        writeln!(generated, "String::from({id:?}),").unwrap();
    }
    generated.push_str("] } }\n");

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"))
        .join("api_coverage.rs");
    fs::write(&output, generated)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output.display()));
}
