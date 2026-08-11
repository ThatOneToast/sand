//! Supported lower-level APIs for custom framework integrations.
//!
//! Most datapack authors should start with [`crate::prelude`]. Use this module
//! when you are building custom export tooling, dynamic function registries, or
//! raw interop around Minecraft features that Sand does not model yet.
//!
//! These APIs are public and supported, but they expose more of Sand's export
//! and generated-output machinery than ordinary packs need. Prefer typed
//! builders from the prelude when they cover the use case.

pub use crate::component::{
    ComponentContent, ComponentFeature, ComponentRecord, VersionCaps, export_components_json,
    try_export_components, try_export_components_for_version, try_export_components_json,
    try_export_components_json_for_version,
};
pub use crate::function::{
    ComponentFactory, EventDescriptor, EventDispatch, EventPathEntry, FunctionDescriptor,
    FunctionPointerEntry, FunctionPointerTypeEntry, FunctionTagDescriptor, ScheduleDescriptor,
    drain_dyn_fns, register_dyn_fn, register_dyn_fn_dedup,
};
pub use sand_components::{RawCommand, RawComponent, RawJson, RawSnbt};

/// Export-time capability information resolved from a project `mc_version`.
///
/// This is an advanced integration value. Ordinary datapack code should use
/// [`crate::version::VersionProfile`] and [`crate::version::VersionFeature`]
/// instead of driving the export capability bridge itself.
#[derive(Debug, Clone)]
pub struct ResolvedExportCaps {
    /// The verified target version or conservative fallback label.
    pub version: String,
    /// Whether Sand could not find an exact profile for the requested version.
    pub is_fallback: bool,
    /// The cycle-safe component capability set consumed by export hooks.
    pub caps: VersionCaps,
}

/// Resolve a raw `sand.toml` `mc_version` for a custom export hook.
///
/// This accepts the configuration boundary's raw string deliberately. It
/// rejects malformed values and returns conservative disabled capabilities for
/// syntactically valid but unverified releases.
pub fn resolve_export_caps(mc_version: &str) -> crate::error::Result<ResolvedExportCaps> {
    let resolved = crate::version::resolve_export_caps(mc_version)?;
    Ok(ResolvedExportCaps {
        version: resolved.version,
        is_fallback: resolved.is_fallback,
        caps: resolved.caps,
    })
}
