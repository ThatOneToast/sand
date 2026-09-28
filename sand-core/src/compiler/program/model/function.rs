//! Owned portable function definitions.
use super::Operation;
use sand_components::ResourceLocation;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Minimum executor context required by a function or external call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ExecutionContext", module = "sand::advanced::compiler",
 summary = "Minimum executor context required by a function or external call.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Server = "Server semantic alternative; see Rustdoc for its execution behavior.", Player = "Player semantic alternative; see Rustdoc for its execution behavior."),
)]
pub enum ExecutionContext {
    /// Does not access player self; callable in either context.
    Server,
    /// Requires a player executor; entry initializes player State.
    Player,
}

/// An explicitly named function with an ordered typed body.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Function", module = "sand::advanced::compiler",
 summary = "An explicitly named function with an ordered typed body.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(id = "Id for this portable definition.", context = "Context for this portable definition.", body = "Body for this portable definition."),
)]
pub struct Function {
    /// Explicit namespaced function identity.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Executor context required by every caller.
    pub context: ExecutionContext,
    /// Operations execute in authored order.
    pub body: Vec<Operation>,
}

/// A function reference distinguishes locally checked and external behavior.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::FunctionReference", module = "sand::advanced::compiler",
 summary = "A function reference distinguishes locally checked and external behavior.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Internal = "Internal semantic alternative; see Rustdoc for its execution behavior.", External = "External semantic alternative; see Rustdoc for its execution behavior."),
 variant_fields(Internal(id = "Id operand for this operation."), External(id = "Id operand for this operation.", context = "Context operand for this operation.")),
)]
pub enum FunctionReference {
    /// Resolves against this complete program before emission.
    Internal {
        /// Function identity declared in any module.
        #[schemars(with = "String")]
        id: ResourceLocation,
    },
    /// Explicit interoperability; existence and behavior cannot be verified.
    External {
        /// External function identity.
        #[schemars(with = "String")]
        id: ResourceLocation,
        /// Context required by the external implementation.
        context: ExecutionContext,
    },
}

/// A typed membership edge; load/tick tags require server-callable functions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::TagMembership", module = "sand::advanced::compiler",
 summary = "A typed membership edge; load/tick tags require server-callable functions.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(tag = "Tag for this portable definition.", function = "Function for this portable definition."),
)]
pub struct TagMembership {
    /// Function tag identity.
    #[schemars(with = "String")]
    pub tag: ResourceLocation,
    /// Internal or external member function.
    pub function: FunctionReference,
}
