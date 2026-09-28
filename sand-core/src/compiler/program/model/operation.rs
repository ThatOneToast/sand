//! Owned portable operation definitions.
use super::{FieldReference, FunctionReference, ScoreComparison};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A gameplay operation with optional host-provided source provenance.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Operation", module = "sand::advanced::compiler",
 summary = "A gameplay operation with optional host-provided source provenance.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(action = "Action for this portable definition.", origin = "Optional host source provenance, excluded from generated identities."),
)]
pub struct Operation {
    /// Typed operation performed in the current executor context.
    pub action: Action,
    /// DSL span or editor node/port metadata; never part of resource identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<serde_json::Value>,
}

/// Supported typed gameplay operations. Raw commands require explicit opt-in.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Action", module = "sand::advanced::compiler",
 summary = "Supported typed gameplay operations. Raw commands require explicit opt-in.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(ScoreSet = "ScoreSet semantic alternative; see Rustdoc for its execution behavior.", ScoreAdd = "ScoreAdd semantic alternative; see Rustdoc for its execution behavior.", Branch = "Branch semantic alternative; see Rustdoc for its execution behavior.", Players = "Players semantic alternative; see Rustdoc for its execution behavior.", Call = "Call semantic alternative; see Rustdoc for its execution behavior.", Raw = "Raw semantic alternative; see Rustdoc for its execution behavior."),
 variant_fields(ScoreSet(score = "Score operand for this operation.", value = "Value operand for this operation."), ScoreAdd(score = "Score operand for this operation.", value = "Value operand for this operation."), Branch(condition = "Condition operand for this operation.", then = "Then operand for this operation.", otherwise = "Otherwise operand for this operation."), Players(body = "Body operand for this operation."), Call(function = "Function operand for this operation."), Raw(command = "Command operand for this operation.")),
)]
pub enum Action {
    /// Assign an integer player-self State field.
    ScoreSet { score: FieldReference, value: i32 },
    /// Add an integer to a player-self State field.
    ScoreAdd { score: FieldReference, value: i32 },
    /// Evaluate the comparison once and execute exactly one ordered body.
    Branch {
        condition: ScoreComparison,
        then: Vec<Operation>,
        #[serde(default)]
        otherwise: Vec<Operation>,
    },
    /// Execute the body as each player without changing positional context.
    Players { body: Vec<Operation> },
    /// Invoke a named function with a checked execution-context requirement.
    Call { function: FunctionReference },
    /// Explicit escape hatch; only command-line safety and syntax are checked.
    Raw { command: String },
}
