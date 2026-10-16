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

/// Low-level state primitives for framework integrations.
///
/// Ordinary datapacks should declare gameplay data with `#[derive(State)]`.
#[api(
    registry = sand_api_contract,
    path = "sand::advanced::state",
    module = "sand::advanced",
    summary = "Low-level state primitives for framework integrations.",
    context = "This explicitly advanced namespace contains the scoreboard-backed primitives used beneath derived State schemas.",
    minecraft = "These primitives lower directly to scoreboards and generated lifecycle commands.",
    use_when = ["Building framework integrations below the derived State authoring layer"],
    avoid_when = ["Declaring ordinary gameplay state; derive State instead"],
    example = "use sand::advanced::state::ScoreVar;",
)]
pub mod state {
    pub use crate::state::{
        Cooldown, Flag, FlagRef, FlowTransitionBuilder, GameState, GameStateRef, ScoreConst,
        ScoreConstants, ScoreExpr, ScoreOperand, ScoreOperation, ScoreRef, ScoreVar, StateFlow,
        StateTransitionBuilder, Timer, TypedGameState,
    };
}

/// Collect a versioned pack export with resources and runtime analysis.
///
/// The object contains `schema_version`, `components`, and `runtime_report`.
/// The report describes explicit command evaluations and records unknown costs;
/// it does not predict MSPT or Minecraft command-budget consumption.
///
/// The raw version text belongs at the project-configuration boundary. Sand
/// resolves it once, applies the matching Minecraft capability gates, and
/// uses a conservative fallback for a parseable release it has not verified.
#[api(
    registry = sand_api_contract,
    path = "sand::advanced::try_export_pack_json",
    module = "sand::advanced",
    summary = "Exports version-validated pack resources with runtime estimates and provenance.",
    context = "Custom build integrations need one fallible boundary from a project namespace and configured Minecraft target to the JSON stream consumed by Sand tooling.",
    minecraft = "Resolves the target release's pack and feature capabilities before serializing datapack resources.",
    use_when = ["Writing a custom sand_export hook", "Integrating Sand's component stream into another build tool"],
    avoid_when = ["Building an ordinary project with sand build", "Manually selecting capability flags or component records"],
    params(
        namespace = "The datapack namespace whose local resources are being exported.",
        mc_version = "The project configuration's Minecraft version text, such as 26.1 or 26.2."
    ),
    returns = "A JSON object with schema_version, components, and runtime_report, or a validation and version-resolution error.",
    example = "let json = sand::advanced::try_export_pack_json(\"example\", \"26.2\")?;"
)]
pub fn try_export_pack_json(namespace: &str, mc_version: &str) -> Result<String> {
    let resolved = crate::version::resolve_export_caps(mc_version)?;
    Ok(crate::compiler::export::try_export_pack_json(
        namespace,
        &resolved.caps,
        &resolved.version,
        resolved.is_fallback,
    )?)
}

/// Owned program model and stateless compiler for external authoring frontends.
///
/// Decode strict JSON or construct a Program, then check or compile with an
/// explicit target. Host adapters own module resolution and pack publication.
#[api(
    registry = sand_api_contract, path = "sand::advanced::compiler", module = "sand::advanced",
    summary = "Compiles owned gameplay programs without Rust collection or host I/O.",
    context = "External languages and editors submit typed programs with explicit targets and module contents.",
    minecraft = "Returns validated deterministic datapack resources using canonical State lifecycle behavior.",
    use_when = ["Embedding Sand in a DSL, node editor, or schema-guided tool"],
    avoid_when = ["Ordinary Rust datapacks can use the prelude and sand build"],
    example = "let schema = sand::advanced::compiler::Compiler::schema();"
)]
pub mod compiler {
    pub use crate::compiler::program::*;
}
