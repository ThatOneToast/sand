//! Portable compiler boundary. This module performs no host I/O or collection.
mod decode;
mod diagnostic;
mod limits;
mod lower;
pub(crate) mod model;
mod validate;

pub use crate::compiler::runtime::{RuntimeAdvice, RuntimeReport};
pub use diagnostic::Diagnostic;
pub use model::{
    Action, Amount, Audience, BiomeMatches, BlockMatches, BlockPosition, BossBar, BossBarColor,
    BossBarStyle, Condition, CookingMethod, CustomItem, DataField, DataOperand, DataProjection,
    DataReference, DimensionMatches, EntitiesExist, EntityData, EntityFlag, EntityQuery,
    EntitySort, EventHook, ExecutionContext, FieldReference, FlagSet, Flow, FlowHook,
    FlowHookPhase, FlowTransition, Function, FunctionReference, ItemHeld, LootItem, LootTable,
    MessageChannel, Module, ModuleSource, Operation, Pack, PlayerEvent, PlayerGameMode, Program,
    ProgramTarget, Rarity, Recipe, RecipeOutput, Relation, ResourceTag, ScaledField,
    ScoreComparison, ScoreField, ScoreOperand, ScoreOperator, ScoreRelation, State, StateAttached,
    TagKind, TagMembership, TeleportDestination, TextColor, TextPart, TextStyle, Weather,
};
pub use sand_commands::DataModifyOperation;
use std::collections::BTreeMap;

// Validation and host discovery consume one catalog as portable coverage grows.
const CAPABILITIES: &[&str] = &[
    "player_state",
    "state_defaults",
    "state_bounds",
    "state_flows",
    "global_state",
    "functions",
    "score",
    "branch",
    "players",
    "calls",
    "lifecycle",
    "function_tags",
    "raw_commands",
    "score_operations",
    "random",
    "field_relations",
    "messages",
    "sounds",
    "particles",
    "effects",
    "items",
    "schedule",
    "owned_waits",
    "player_events",
    "control_flow",
    "attributes",
    "entities",
    "custom_items",
    "recipes",
    "resource_tags",
    "entity_state",
    "teleport",
    "timers",
    "blocks",
    "world_conditions",
    "biome_conditions",
    "experience",
    "world_state",
    "boss_bars",
    "loot_tables",
    "smithing_recipes",
    "recursion",
    "entity_data",
];

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
 fields(target = "Target for this portable definition.", resources = "Resources for this portable definition.", diagnostics = "Diagnostics for this portable definition.", runtime_report = "Runtime estimates and compiler-owned provenance."),
)]
pub struct CompiledPack {
    /// Exact verified Minecraft target used for compilation.
    pub target: String,
    /// Sorted pack-relative paths and final bytes, including pack.mcmeta.
    pub resources: BTreeMap<String, Vec<u8>>,
    /// Nonfatal diagnostics; v1 currently emits none on success.
    pub diagnostics: Vec<Diagnostic>,
    /// Runtime estimates and provenance, separate from published datapack files.
    pub runtime_report: RuntimeReport,
}

/// An inclusive line range in an emitted function and the operation it came from.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::CommandSource", module = "sand::advanced::compiler",
 summary = "Trace an emitted command range to its portable operation.",
 context = "A location in the optional command source map of a portable compilation.",
 minecraft = "Counts lines in the final mcfunction resource after export.",
 use_when = ["Mapping generated command diagnostics back to a portable producer"],
 avoid_when = ["Only deterministic datapack files are needed"],
 example = "use sand::advanced::compiler::CommandSource;",
 fields(lines = "Inclusive one-based generated command lines.", module = "Portable module identity.", pointer = "JSON pointer within the module."),
)]
pub struct CommandSource {
    /// Inclusive one-based lines in the emitted resource.
    pub lines: std::ops::RangeInclusive<usize>,
    /// Portable module identity.
    pub module: String,
    /// JSON pointer within the portable module document.
    pub pointer: String,
}

/// Generated function paths and their command sources in line order.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::CommandSources", module = "sand::advanced::compiler",
 summary = "Index emitted function line ranges by final pack-relative path.",
 context = "Optional provenance index produced by portable compilation.",
 minecraft = "Refers to the same mcfunction files emitted in CompiledPack.resources.",
 use_when = ["Looking up source locations for generated commands"],
 avoid_when = ["The host only needs published pack bytes"],
 example = "use sand::advanced::compiler::CommandSources;",
 fields(functions = "Final resource paths and their command source ranges."),
)]
pub struct CommandSources {
    /// Final `.mcfunction` resource paths and their command source ranges.
    pub functions: BTreeMap<String, Vec<CommandSource>>,
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
        Self::compile_inner(program, None)
    }

    /// Compile the same pack bytes as [`Self::compile`] and trace emitted command
    /// lines to portable operations. Framework initialization without an
    /// operation origin is omitted from the index.
    #[sand_macros::api(
     registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::compile_with_command_sources", module = "sand::advanced::compiler",
     summary = "Compile a portable pack and locate emitted command ranges.",
     context = "Opt-in provenance for hosts that carry portable JSON pointers back to authored source.",
     minecraft = "Emits the same bytes as ordinary portable compilation.",
     use_when = ["Displaying source locations for emitted commands"],
     avoid_when = ["The host only needs published pack bytes"],
     example = "use sand::advanced::compiler::{Compiler, Program}; fn demo(program: &Program) { let _ = Compiler::compile_with_command_sources(program); }",
     kind = "method", returns = "Compiled pack and command source index, or validation diagnostics.", params(program = "Resolved owned program with an explicit target."),
    )]
    pub fn compile_with_command_sources(
        program: &Program,
    ) -> Result<(CompiledPack, CommandSources), Vec<Diagnostic>> {
        let mut sources = CommandSources::default();
        let pack = Self::compile_inner(program, Some(&mut sources.functions))?;
        Ok((pack, sources))
    }

    fn compile_inner(
        program: &Program,
        sources: Option<&mut BTreeMap<String, Vec<CommandSource>>>,
    ) -> Result<CompiledPack, Vec<Diagnostic>> {
        let target = validate::validate(program)?;
        let lowered = lower::lower(program, &target, sources)?;
        Ok(CompiledPack {
            target: target.resolved_name().into(),
            resources: lowered.resources,
            diagnostics: Vec::new(),
            runtime_report: lowered.runtime_report,
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
        let schema = Self::schema();
        let vocabulary = |definition: &str, field: &str| -> Vec<String> {
            schema["$defs"][definition]["oneOf"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|entry| {
                    entry["properties"][field]["const"]
                        .as_str()
                        .map(str::to_string)
                })
                .collect()
        };
        serde_json::json!({
            "format": "sand.program", "format_version": 1,
            "compiler": { "name": "sand", "version": env!("CARGO_PKG_VERSION") },
            "targets": sand_version::registry::targets().collect::<Vec<_>>(),
            "operations": vocabulary("Action", "op"),
            "events": vocabulary("PlayerEvent", "kind"),
            "capabilities": CAPABILITIES,
            "raw_validation": "explicit opt-in; command-line safety and supported syntax only",
            "limits": { "document_bytes": limits::DOCUMENT_BYTES, "input_bytes": limits::INPUT_BYTES,
                "modules": limits::MODULES, "json_depth": limits::JSON_DEPTH, "operation_depth": limits::OP_DEPTH,
                "declarations": limits::DECLARATIONS, "operations": limits::OPERATIONS,
                "resources": limits::RESOURCES, "resource_path_bytes": limits::RESOURCE_PATH_BYTES,
                "resource_segment_bytes": limits::RESOURCE_SEGMENT_BYTES, "resource_bytes": limits::DOCUMENT_BYTES, "output_bytes": limits::OUTPUT_BYTES }
        })
    }
}
