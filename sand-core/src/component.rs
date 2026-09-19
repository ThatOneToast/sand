//! Public component/export surface of `sand-core`.
//!
//! The export implementation lives in the compiler pipeline
//! (`crate::compiler::export`, split by phase in Phase 2 of ADR 001); this
//! module keeps the long-standing public paths stable.

// ── Unified registration and resource traits ─────────────────────────────────
// DatapackComponent remains the one-resource definition owned by
// sand-components. IntoDatapack and DatapackRegistration live here because
// they aggregate compiler behavior in addition to component files.

pub use crate::registration::{
    DatapackRegistration, FunctionTagContribution, IntoDatapack, LifecycleContribution,
};
pub use sand_components::component::{ComponentContent, DatapackComponent};
pub use sand_components::error::SandError as ComponentExportError;
pub use sand_version::{ComponentFeature, VersionCaps};

pub use crate::compiler::export::{
    ComponentRecord, ExportResult, export_components_json, try_export_components,
    try_export_components_for_version, try_export_components_json,
    try_export_components_json_for_version,
};
