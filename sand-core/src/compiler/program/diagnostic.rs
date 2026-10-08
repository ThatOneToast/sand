//! Stable machine diagnostics for every portable compilation phase.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A located error from decoding, resolution, validation, or lowering.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Diagnostic", module = "sand::advanced::compiler",
 summary = "A located error from decoding, resolution, validation, or lowering.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(code = "Code for this portable definition.", severity = "Severity for this portable definition.", message = "Message for this portable definition.", module = "Module for this portable definition.", pointer = "Pointer for this portable definition.", origin = "Optional host source provenance, excluded from generated identities."),
)]
pub struct Diagnostic {
    /// Stable category code, suitable for machine branching.
    pub code: String,
    /// Currently `error`; warnings may be introduced independently.
    pub severity: String,
    /// Human-readable explanation without requiring implementation knowledge.
    pub message: String,
    /// Explicit module identity or host-supplied document identity.
    pub module: String,
    /// RFC 6901 JSON pointer relative to the identified document/module.
    pub pointer: String,
    /// Optional source span or editor node/port metadata supplied by the host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<serde_json::Value>,
}

impl Diagnostic {
    pub(super) fn error(code: &str, module: &str, pointer: &str, message: impl ToString) -> Self {
        Self {
            code: code.into(),
            severity: "error".into(),
            message: message.to_string(),
            module: module.into(),
            pointer: pointer.into(),
            origin: None,
        }
    }
}
