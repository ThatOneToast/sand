//! Portable compiler boundary. This module performs no host I/O or collection.
mod decode;
mod diagnostic;
mod limits;
mod lower;
pub(crate) mod model;
mod validate;

pub use diagnostic::Diagnostic;
pub use model::{
    Action, ExecutionContext, FieldReference, Function, FunctionReference, Module, ModuleSource,
    Operation, Pack, Program, ProgramTarget, ScoreComparison, ScoreField, State, TagMembership,
};
use std::collections::BTreeMap;

/// Complete deterministic pack output, ready for a host's publication adapter.
#[derive(Debug, Clone, serde::Serialize)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::CompiledPack", module = "sand::advanced::compiler",
 summary = "Complete deterministic pack output, ready for a host's publication adapter.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(target = "Target for this portable definition.", resources = "Resources for this portable definition.", diagnostics = "Diagnostics for this portable definition."),
)]
pub struct CompiledPack {
    /// Exact verified Minecraft target used for compilation.
    pub target: String,
    /// Sorted pack-relative paths and final bytes, including pack.mcmeta.
    pub resources: BTreeMap<String, Vec<u8>>,
    /// Nonfatal diagnostics; v1 currently emits none on success.
    pub diagnostics: Vec<Diagnostic>,
}

/// Stateless portable compiler; repeated and concurrent calls share no state.
///
/// Rust collection and JSON producers provide the same owned Program. Explicit
/// targets and in-memory module contents make this boundary suitable for DSLs,
/// editors, and subprocess adapters without Cargo or environment configuration.

#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Compiler", module = "sand::advanced::compiler",
 summary = "Stateless portable compiler; repeated and concurrent calls share no state.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",

)]
pub struct Compiler;
impl Compiler {
    /// Decode an envelope and explicitly supplied local module documents.
    ///
    /// All input is strict JSON. Unknown fields, duplicate keys, unsupported
    /// scalar types and input limits fail before semantic compilation. Paths
    /// only index `modules`; this method performs no filesystem discovery.

    #[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::decode", module = "sand::advanced::compiler",
 summary = "Decode a strict portable envelope and explicitly supplied modules.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 kind = "method", returns = "The validated result or located diagnostics; schema and capability methods return machine-readable JSON.", params(bytes = "UTF-8 envelope document bytes.", modules = "Explicit module-path to UTF-8 document contents map."),
)]
    pub fn decode(
        bytes: &[u8],
        modules: &std::collections::BTreeMap<String, Vec<u8>>,
    ) -> Result<Program, Vec<Diagnostic>> {
        decode::resolve(bytes, modules)
    }

    /// List manifest paths so a host can supply a constrained resolver.
    ///
    /// This does not open files. Decode the supplied documents with `decode`
    /// after resolving these paths under the host's declared project root.

    #[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::module_paths", module = "sand::advanced::compiler",
 summary = "List flat module inputs without opening files.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 kind = "method", returns = "The validated result or located diagnostics; schema and capability methods return machine-readable JSON.", params(bytes = "UTF-8 envelope document bytes."),
)]
    pub fn module_paths(bytes: &[u8]) -> Result<Vec<String>, Vec<Diagnostic>> {
        decode::module_paths(bytes)
    }

    /// Validate every declaration, context, command and generated-output limit.
    ///
    /// Uses the same compilation path as `compile`, then discards resource bytes.

    #[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::check", module = "sand::advanced::compiler",
 summary = "Validate all program semantics and generated output limits.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 kind = "method", returns = "The validated result or located diagnostics; schema and capability methods return machine-readable JSON.", params(program = "Resolved owned program with an explicit target."),
)]
    pub fn check(program: &Program) -> Result<(), Vec<Diagnostic>> {
        Self::compile(program).map(|_| ())
    }

    /// Compile a resolved owned program with its explicit target into a pack.
    ///
    /// No inventory, author callbacks, environment, filesystem, or network are
    /// consulted. Output ordering depends on semantic IDs and authored bodies,
    /// never module filenames or origin metadata. Publication belongs to hosts.

    #[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::compile", module = "sand::advanced::compiler",
 summary = "Compile a resolved program into deterministic complete pack resources.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 kind = "method", returns = "The validated result or located diagnostics; schema and capability methods return machine-readable JSON.", params(program = "Resolved owned program with an explicit target."),
)]
    pub fn compile(program: &Program) -> Result<CompiledPack, Vec<Diagnostic>> {
        let target = validate::validate(program)?;
        let resources = lower::lower(program, &target)?;
        Ok(CompiledPack {
            target: target.resolved_name().into(),
            resources,
            diagnostics: Vec::new(),
        })
    }

    /// Generate the bundled envelope schema from the authoritative Rust model.
    ///
    /// Both embedded definitions and flat module paths are accepted. Domain
    /// definitions live in `$defs`, whose references resolve entirely offline.

    #[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::schema", module = "sand::advanced::compiler",
 summary = "Generate the bundled portable program JSON Schema.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 kind = "method", returns = "The validated result or located diagnostics; schema and capability methods return machine-readable JSON.",
)]
    pub fn schema() -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(Program<ModuleSource>))
            .expect("schema serializes")
    }

    /// Generate a standalone module-document schema for manifest producers.

    #[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::module_schema", module = "sand::advanced::compiler",
 summary = "Generate a standalone module-document JSON Schema.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 kind = "method", returns = "The validated result or located diagnostics; schema and capability methods return machine-readable JSON.",
)]
    pub fn module_schema() -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(Module)).expect("schema serializes")
    }

    /// Discover protocol identity, operations, target policy and bounded-work limits.

    #[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::capabilities", module = "sand::advanced::compiler",
 summary = "Describe supported protocol operations, targets, and resource limits.",
 context = "Owned portable gameplay definitions and diagnostics shared by Rust construction and strict JSON decoding.",
 minecraft = "Validates explicit targets and emits deterministic datapack resources through canonical State and command lowering.",
 use_when = ["Integrating a language, DSL, or editor with Sand without Cargo per program"],
 avoid_when = ["Ordinary Rust gameplay authoring is better served by the prelude"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 kind = "method", returns = "The validated result or located diagnostics; schema and capability methods return machine-readable JSON.",
)]
    pub fn capabilities() -> serde_json::Value {
        serde_json::json!({
            "format": "sand.program", "format_version": 1,
            "compiler": { "name": "sand", "version": env!("CARGO_PKG_VERSION") },
            "targets": ["26.1", "26.1.1", "26.1.2", "26.2"],
            "operations": ["score_set", "score_add", "branch", "players", "call", "raw"],
            "capabilities": ["player_state", "functions", "score", "branch", "players", "calls", "lifecycle", "function_tags", "raw_commands"],
            "raw_validation": "explicit opt-in; command-line safety and supported syntax only",
            "limits": { "document_bytes": limits::DOCUMENT_BYTES, "input_bytes": limits::INPUT_BYTES,
                "modules": limits::MODULES, "json_depth": limits::JSON_DEPTH, "operation_depth": limits::OP_DEPTH,
                "declarations": limits::DECLARATIONS, "operations": limits::OPERATIONS,
                "resources": limits::RESOURCES, "resource_path_bytes": limits::RESOURCE_PATH_BYTES,
                "resource_segment_bytes": limits::RESOURCE_SEGMENT_BYTES, "resource_bytes": limits::DOCUMENT_BYTES, "output_bytes": limits::OUTPUT_BYTES }
        })
    }
}
