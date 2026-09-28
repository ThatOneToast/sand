//! Owned portable envelope definitions.
use super::{Function, Operation, State, TagMembership};
use sand_components::ResourceLocation;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A complete compilation request with explicit protocol and target metadata.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Program", module = "sand::advanced::compiler",
 summary = "A complete compilation request with explicit protocol and target metadata.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(format = "Protocol format discriminator, sand.program.", format_version = "Protocol revision, currently 1.", target = "ProgramTarget for this portable definition.", pack = "Pack for this portable definition.", requires = "Requires for this portable definition.", modules = "Modules for this portable definition."),
)]
pub struct Program<M = Module> {
    /// Must be `sand.program`.
    pub format: String,
    /// Portable protocol revision; currently 1.
    pub format_version: u32,
    /// Exact verified Minecraft release.
    pub target: ProgramTarget,
    /// Pack identity and description.
    pub pack: Pack,
    /// Capabilities required by this input, checked independently of actual use.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    /// Flat module declarations. Resolved programs contain no paths.
    pub modules: Vec<M>,
}

/// A target supplied explicitly by the host, never inferred from environment.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ProgramTarget", module = "sand::advanced::compiler",
 summary = "A target supplied explicitly by the host, never inferred from environment.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(minecraft = "Minecraft for this portable definition."),
)]
pub struct ProgramTarget {
    /// Exact release such as `26.2`; `latest` is not portable.
    pub minecraft: String,
}

/// Metadata shared by all resources in one pack.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Pack", module = "sand::advanced::compiler",
 summary = "Metadata shared by all resources in one pack.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(namespace = "Namespace for this portable definition.", description = "Description for this portable definition."),
)]
pub struct Pack {
    /// Namespace for compiler lifecycle resources.
    pub namespace: String,
    /// Human-readable description emitted into pack.mcmeta.
    pub description: String,
}

/// An embedded module or a manifest-relative local module document path.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::ModuleSource", module = "sand::advanced::compiler",
 summary = "An embedded module or a manifest-relative local module document path.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Embedded = "Embedded semantic alternative; see Rustdoc for its execution behavior.", Path = "Path semantic alternative; see Rustdoc for its execution behavior."),
 variant_fields(Embedded = ["An owned embedded module definition."], Path = ["A manifest-relative local module document path."]),
)]
pub enum ModuleSource {
    /// A module embedded in the envelope.
    Embedded(Module),
    /// A path resolved by an explicit host resolver.
    Path(String),
}

/// One named owner of declarations and ordered lifecycle contributions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Module", module = "sand::advanced::compiler",
 summary = "One named owner of declarations and ordered lifecycle contributions.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(id = "Id for this portable definition.", states = "States for this portable definition.", functions = "Functions for this portable definition.", load = "Load for this portable definition.", tick = "Tick for this portable definition.", tags = "Tags for this portable definition."),
)]
pub struct Module {
    /// Explicit semantic identity, independent of source filename.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// Player State definitions owned by this module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<State>,
    /// Named functions, including declarations unreachable from tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<Function>,
    /// Server-context bodies run at load in authored order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub load: Vec<Operation>,
    /// Server-context bodies run each tick in authored order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tick: Vec<Operation>,
    /// Typed function-tag membership.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<TagMembership>,
}
