//! Entry points of the export pipeline.
//!
//! This module owns the public export functions re-exported through
//! [`crate::component`] and the version-aware [`ExportCtx`] threaded through
//! every phase. The phases themselves live in the sibling modules:
//! [`pipeline`] (collection → aggregation → assembly driver), [`records`]
//! (component → record boundary), [`events`] (event graph lowering),
//! [`predicates`], [`armor`], [`schedules`], [`dialogs`], [`functions`],
//! [`lifecycle`], [`diagnostics`], and [`tags`].
#![allow(clippy::result_large_err)]

pub(crate) mod armor;
pub(crate) mod diagnostics;
pub(crate) mod dialogs;
pub(crate) mod events;
pub(crate) mod functions;
pub(crate) mod identities;
pub(crate) mod lifecycle;
pub(crate) mod participant_transport;
pub(crate) mod pipeline;
pub(crate) mod predicates;
pub(crate) mod records;
pub(crate) mod schedules;
pub(crate) mod tags;

pub use self::records::{ComponentRecord, ExportResult};

use sand_version::VersionCaps;

/// One collection result; reporting metadata is separate from datapack records.
pub(crate) struct CollectedPack {
    pub(crate) records: Vec<ComponentRecord>,
    pub(crate) runtime: crate::compiler::runtime::RuntimePlan,
}

/// Version-aware export context — carries the resolved capability set and the
/// requested version string for diagnostics.
pub(crate) struct ExportCtx<'a> {
    pub(crate) caps: &'a VersionCaps,
    pub(crate) requested_version: &'a str,
    pub(crate) is_fallback: bool,
}

/// Collect all inventory-registered components into records, routing every
/// `ComponentFactory` through the fallible `component_to_record` helper
/// which validates all components (JSON, text, and copy-backed) before
/// accepting their content.
///
/// This is the **unprofiled** compatibility path — no version-gating is
/// performed. Use [`try_export_components_for_version`] when the target
/// `VersionProfile` is known so that version-gated components are rejected
/// before any pack output is written.
pub fn try_export_components(namespace: &str) -> ExportResult<Vec<ComponentRecord>> {
    pipeline::try_export_components_impl(namespace, None).map(|pack| pack.records)
}

/// Version-aware fallible export: collect all inventory-registered components,
/// rejecting components and advancement-backed events that require features not
/// available in the target [`VersionCaps`].
///
/// The `requested_version` string and `is_fallback` flag are used for
/// diagnostics only. Invalid components and unsupported version-gated
/// components are rejected **before** any pack output is written.
pub fn try_export_components_for_version(
    namespace: &str,
    caps: &VersionCaps,
    requested_version: &str,
    is_fallback: bool,
) -> ExportResult<Vec<ComponentRecord>> {
    let ctx = ExportCtx {
        caps,
        requested_version,
        is_fallback,
    };
    pipeline::try_export_components_impl(namespace, Some(&ctx)).map(|pack| pack.records)
}

/// Fallibly collect all inventory-registered components and return them as a
/// resource-only JSON string for inspection and compiler tests.
///
/// Build integrations use [`crate::advanced::try_export_pack_json`] so runtime
/// provenance travels with the resources. This resource-only API returns the
/// JSON array of [`ComponentRecord`] objects as a
/// `String`. On failure it returns a [`sand_components::error::SandError`] carrying the
/// resource location, component kind, and validation field — **no panic, no
/// backtrace**. The caller is responsible for printing a diagnostic to stderr
/// and exiting non-zero.
///
/// See [`try_export_components`] for the record-level fallible API.
pub fn try_export_components_json(namespace: &str) -> ExportResult<String> {
    let records = try_export_components(namespace)?;
    serde_json::to_string_pretty(&records).map_err(sand_components::error::SandError::Serialization)
}

/// Version-aware fallible JSON export: collect all components and advancement-backed
/// events, rejecting any that require features not available in the target version.
///
/// Use this resource-only form for version-aware inspection. Build exporters
/// use [`crate::advanced::try_export_pack_json`] to include runtime analysis.
/// On failure it returns a [`sand_components::error::SandError`]
/// carrying the resource location, component kind, and version-gating context
/// — no panic, no backtrace.
pub fn try_export_components_json_for_version(
    namespace: &str,
    caps: &VersionCaps,
    requested_version: &str,
    is_fallback: bool,
) -> ExportResult<String> {
    let records =
        try_export_components_for_version(namespace, caps, requested_version, is_fallback)?;
    serde_json::to_string_pretty(&records).map_err(sand_components::error::SandError::Serialization)
}

/// Collect all inventory-registered components and return them as a JSON string
/// for consumption by `sand build`.
///
/// **Compatibility wrapper** — this function **panics** on component validation
/// or serialization failure. It is retained for backward compatibility with
/// direct callers that expect an infallible `String` return. The generated
/// `sand_export` binary and all scaffold templates use
/// [`try_export_components_json`] or [`try_export_components_json_for_version`]
/// instead.
pub fn export_components_json(namespace: &str) -> String {
    match try_export_components_json(namespace) {
        Ok(s) => s,
        Err(e) => panic!("sand component export failed: {e}"),
    }
}

/// Serialize compiler resources together with their compilation-local report.
pub(crate) fn try_export_pack_json(
    namespace: &str,
    caps: &VersionCaps,
    requested_version: &str,
    is_fallback: bool,
) -> ExportResult<String> {
    #[derive(serde::Serialize)]
    struct PackExport {
        schema_version: u32,
        components: Vec<ComponentRecord>,
        runtime_report: crate::compiler::runtime::RuntimeReport,
    }
    let ctx = ExportCtx {
        caps,
        requested_version,
        is_fallback,
    };
    let pack = pipeline::try_export_components_impl(namespace, Some(&ctx))?;
    let runtime_report =
        crate::compiler::runtime::RuntimeReport::collect(&pack.records, &pack.runtime);
    serde_json::to_string_pretty(&PackExport {
        schema_version: 1,
        components: pack.records,
        runtime_report,
    })
    .map_err(sand_components::error::SandError::Serialization)
}
