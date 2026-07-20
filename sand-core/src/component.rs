//! Public component/export surface of `sand-core`.
//!
//! The export implementation lives in the compiler pipeline
//! (`crate::compiler::export`, split by phase in Phase 2 of ADR 001); this
//! module keeps the long-standing public paths stable.

// ── Unified traits ────────────────────────────────────────────────────────────
// Re-export the canonical definitions from sand-components so the entire
// workspace shares ONE set of traits.  All builders in sand-components already
// implement these; McFunction (below) does too via crate::resource_location
// which now resolves to sand_components::ResourceLocation.

pub use sand_components::component::{ComponentContent, DatapackComponent, IntoDatapack};
pub use sand_components::error::SandError as ComponentExportError;
pub use sand_version::{ComponentFeature, VersionCaps};

pub use crate::compiler::export::{
    ComponentRecord, ExportResult, export_components_json, try_export_components,
    try_export_components_for_version, try_export_components_json,
    try_export_components_json_for_version,
};
