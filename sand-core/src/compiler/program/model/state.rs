//! Owned portable state definitions.
use sand_components::ResourceLocation;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Revision-one, zero-default player score State using canonical ownership.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::State", module = "sand::advanced::compiler",
 summary = "Revision-one, zero-default player score State using canonical ownership.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(id = "Id for this portable definition.", scope = "Scope for this portable definition.", revision = "Revision for this portable definition.", fields = "Fields for this portable definition."),
)]
pub struct State {
    /// Namespaced State identity.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Must be `player` in protocol v1.
    pub scope: String,
    /// State schema revision, currently 1; distinct from protocol revision.
    pub revision: u32,
    /// Named integer score fields.
    pub fields: Vec<ScoreField>,
}

/// An integer State field. Defaults other than zero are not yet portable.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ScoreField", module = "sand::advanced::compiler",
 summary = "An integer State field. Defaults other than zero are not yet portable.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(name = "Name for this portable definition.", default = "Default for this portable definition."),
)]
pub struct ScoreField {
    /// Field identity within its owning State.
    pub name: String,
    /// Initial value, applied only when missing.
    #[serde(default)]
    pub default: i32,
}

/// A field reference binds a field identity to its declaring State.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::FieldReference", module = "sand::advanced::compiler",
 summary = "A field reference binds a field identity to its declaring State.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(state = "State for this portable definition.", field = "Field for this portable definition."),
)]
pub struct FieldReference {
    /// Declaring State identity.
    #[schemars(with = "String")]
    pub state: ResourceLocation,
    /// Field name within that State.
    pub field: String,
}

/// Inclusive integer comparison bounds for a player State score.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ScoreComparison", module = "sand::advanced::compiler",
 summary = "Inclusive integer comparison bounds for a player State score.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(score = "Score for this portable definition.", min = "Min for this portable definition.", max = "Max for this portable definition."),
)]
pub struct ScoreComparison {
    /// Player-self score to compare.
    pub score: FieldReference,
    /// Inclusive lower bound; omission means unbounded below.
    pub min: Option<i32>,
    /// Inclusive upper bound; omission means unbounded above.
    pub max: Option<i32>,
}
