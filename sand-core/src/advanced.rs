//! Supported low-level export hook for custom framework integrations.
//!
//! Most datapack authors should start with [`crate::prelude`]. Use this module
//! when you are integrating Sand's component exporter into a custom build
//! workflow.
//!
//! This API is public and supported, but intentionally does not expose Sand's
//! inventory descriptors, component-record protocol, or capability plumbing.
//! Prefer typed builders from the prelude when they cover the use case.

use sand_macros::api;

use crate::error::Result;

/// Collect registered components as JSON for a configured Minecraft version.
///
/// The raw version text belongs at the project-configuration boundary. Sand
/// resolves it once, applies the matching Minecraft capability gates, and
/// uses a conservative fallback for a parseable release it has not verified.
#[api(
    registry = sand_api_contract,
    path = "sand::advanced::try_export_components_json",
    module = "sand::advanced",
    summary = "Exports registered Sand components as version-validated JSON.",
    context = "Custom build integrations need one fallible boundary from a project namespace and configured Minecraft target to the JSON stream consumed by Sand tooling.",
    minecraft = "Resolves the target release's pack and feature capabilities before serializing datapack resources.",
    use_when = ["Writing a custom sand_export hook", "Integrating Sand's component stream into another build tool"],
    avoid_when = ["Building an ordinary project with sand build", "Manually selecting capability flags or component records"],
    params(
        namespace = "The datapack namespace whose local resources are being exported.",
        mc_version = "The project configuration's Minecraft version text, such as 1.21.4 or 26.2."
    ),
    returns = "Pretty JSON component records, or a validation and version-resolution error.",
    example = "let json = sand::advanced::try_export_components_json(\"example\", \"26.2\")?;"
)]
pub fn try_export_components_json(namespace: &str, mc_version: &str) -> Result<String> {
    let resolved = crate::version::resolve_export_caps(mc_version)?;
    Ok(crate::component::try_export_components_json_for_version(
        namespace,
        &resolved.caps,
        &resolved.version,
        resolved.is_fallback,
    )?)
}
