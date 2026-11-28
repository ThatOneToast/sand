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
    AbilityCommand, AbilityCost, AbilityDefinition, Action, Advancement, AdvancementCriterion,
    AdvancementDisplay, Amount, ArchetypeCommand, Audience, BiomeMatches, BlockDataAddress,
    BlockMatches, BlockPosition, BossBar, BossBarColor, BossBarStyle, CheckpointCommand,
    CheckpointDefinition, CheckpointFieldPair, CombatCritical, CombatDefinition, Condition,
    CookingMethod, CustomBiome, CustomItem, DataField, DataOperand, DataProjection, DataReference,
    DimensionDefinition, DimensionMatches, EncounterAttack, EncounterDefinition, EntitiesExist,
    EntityArchetypeDefinition, EntityData, EntityFlag, EntityQuery, EntitySort, EventHook,
    ExecutionContext, FieldReference, FlagSet, FlatDimension, Flow, FlowHook, FlowHookPhase,
    FlowTransition, Function, FunctionReference, HudBossBar, HudDefinition,
    InventoryTransferDefinition, ItemHeld, JigsawStructure, JobDefinition, LootItem, LootPool,
    LootTable, MenuChoice, MenuDefinition, MenuView, MessageChannel, Module, ModuleSource,
    NoiseDimension, NoticeDialog, Operation, OreFeatureDefinition, Pack, PlayerEvent,
    PlayerGameMode, Program, ProgramTarget, ProgressionBranch, ProgressionDefinition,
    ProgressionObjective, ProjectileDefinition, Rarity, Recipe, RecipeOutput, Relation,
    ResourceTag, RosterCommand, RosterDefinition, ScaledField, SceneContent, SceneDefinition,
    SceneKeyframe, ScenePart, ScoreComparison, ScoreField, ScoreOperand, ScoreOperator,
    ScoreRelation, SessionCommand, SessionDefinition, State, StateAttached, StateCooldown,
    StatusEffectDefinition, TagKind, TagMembership, TeleportDestination, TemplatePoolDefinition,
    TemplatePoolPiece, TextColor, TextPart, TextStyle, Weather, WorldPredicate, WrittenBookContent,
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
    "translated_text",
    "interactive_text",
    "written_books",
    "advancements",
    "advancement_completion_events",
    "notice_dialogs",
    "flat_dimensions",
    "fixed_biome_noise_dimensions",
    "dimension_types",
    "structure_sets",
    "jigsaw_structures",
    "placed_biome_features",
    "ore_features",
    "configured_features",
    "placed_features",
    "player_checkpoints",
    "owned_encounters",
    "owned_status_effects",
    "entity_archetypes",
    "online_rosters",
    "event_predicate_guards",
    "named_breakpoints",
    "player_transitions",
    "custom_biomes",
    "structure_templates",
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
    "damage_types",
    "predicates",
    "smithing_recipes",
    "recursion",
    "entity_data",
    "entity_data_queries",
    "inventory_counts",
    "runtime_item_removal",
    "inventory_transfers",
    "owner_huds",
    "native_menus",
    "owned_projectiles",
    "progressions",
    "owned_sessions",
    "owned_jobs",
    "owned_scenes",
    "owned_abilities",
    "state_combat",
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

/// A compiled wait site and the read-only command for its pending instances.
#[derive(Debug, Clone, serde::Serialize)]
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::advanced::compiler::WaitInspection", module = "sand::advanced::compiler",
    summary = "A read-only probe for pending instances of a compiled wait site.",
    context = "The portable operation's origin lets authoring tools show the corresponding wait.",
    minecraft = "Reads the native queue or owner ledger without ticking, cancelling, or resuming it.",
    use_when = ["Inspecting delayed work in a running Minecraft development world"],
    avoid_when = ["Cancelling a wait or asking a continuation to resume"],
    fields(site = "Stable identity of the compiled wait site.", group = "Optional authored cancellation group.", command = "Read-only Minecraft query.", origin = "Optional portable operation origin."),
    example = "use sand::advanced::compiler::WaitInspection;",
)]
pub struct WaitInspection {
    /// Generated identity of this wait site.
    pub site: String,
    /// Optional authored cancellation group.
    pub group: Option<String>,
    /// Read-only command addressing the same queue or ledger used by the runtime.
    pub command: String,
    /// Optional origin supplied by the portable program producer.
    pub origin: Option<serde_json::Value>,
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

    /// Build read-only native commands that inspect one State's current values.
    ///
    /// An owned State needs the UUID of one player or entity as Minecraft's
    /// four signed integers. Global State uses its canonical fake score holder
    /// and storage slot. The result includes the stored revision and, for owned
    /// State, its suppression flag. Calling these commands never attaches State.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::state_inspection", module = "sand::advanced::compiler",
        summary = "Build read-only commands for inspecting a portable State.",
        context = "Inspect global State or the exact State record of a known player or entity.",
        minecraft = "Reads generated scoreboard and native storage identities without changing them.",
        use_when = ["Inspecting stored State from a local Minecraft console or debugger"],
        avoid_when = ["Reading gameplay State inside a Sand function"],
        params(program = "Resolved owned program with an explicit target.", id = "State resource identity.", owner = "Minecraft UUID split into four signed integers for owned State."),
        returns = "Native read commands keyed by State field name, or validation diagnostics.",
        example = "use sand::advanced::compiler::{Compiler, Program}; fn probe(program: &Program) { let _ = Compiler::state_inspection(program, &\"example:round\".parse().unwrap(), None); }",
    )]
    pub fn state_inspection(
        program: &Program,
        id: &sand_components::ResourceLocation,
        owner: Option<[i32; 4]>,
    ) -> Result<BTreeMap<String, String>, Vec<Diagnostic>> {
        validate::validate(program)?;
        let error = |message| {
            vec![Diagnostic::error(
                "SAND_PROGRAM_INSPECT",
                &id.to_string(),
                "",
                message,
            )]
        };
        let state = program
            .modules
            .iter()
            .flat_map(|module| &module.states)
            .find(|state| &state.id == id)
            .ok_or_else(|| error(format!("unknown State `{id}`")))?;
        let global = state.scope == "global";
        let (holder, owner_path) = match (global, owner) {
            (true, None) => (None, None),
            (false, Some([first, second, third, fourth])) => {
                let hex = format!(
                    "{:08x}{:08x}{:08x}{:08x}",
                    first as u32, second as u32, third as u32, fourth as u32
                );
                let uuid = format!(
                    "{}-{}-{}-{}-{}",
                    &hex[..8],
                    &hex[8..12],
                    &hex[12..16],
                    &hex[16..20],
                    &hex[20..]
                );
                let snbt = format!("[I;{first},{second},{third},{fourth}]");
                (
                    Some(uuid),
                    Some(crate::entity::state::state_data_owner_path(&snbt)),
                )
            }
            (true, Some(_)) => {
                return Err(error(format!("global State `{id}` has no entity owner")));
            }
            (false, None) => return Err(error(format!("State `{id}` requires an owner UUID"))),
        };
        let read_score = |field: &str| {
            let objective = sand_commands::ObjectiveName::logical(format!("{id}.{field}"));
            match &holder {
                Some(uuid) => {
                    format!("execute as {uuid} run scoreboard players get @s {objective}")
                }
                None => format!(
                    "scoreboard players get {} {objective}",
                    crate::state::owned::holder(state)
                ),
            }
        };
        let mut fields = BTreeMap::from([("@revision".into(), read_score("presence"))]);
        if !global {
            fields.insert("@suppressed".into(), read_score("suppressed"));
        }
        for field in &state.fields {
            fields.insert(field.name.clone(), read_score(&field.name));
        }
        for field in &state.data_fields {
            let cmd = crate::state::owned::with_data_field(state, &field.name, |descriptor| {
                match &owner_path {
                    Some(owner) => format!(
                        "data get storage {} {owner}.{}",
                        descriptor.storage, descriptor.path
                    ),
                    None => format!(
                        "data get storage {} {}",
                        descriptor.storage, descriptor.path
                    ),
                }
            });
            fields.insert(field.name.clone(), cmd);
        }
        Ok(fields)
    }

    /// Generate the native command that arms or releases one authored breakpoint.
    ///
    /// A breakpoint belongs to the program's debugger State and applies to every
    /// invocation reaching its label. Releasing it resumes loaded instances on
    /// their next eligible tick.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::breakpoint_control", module = "sand::advanced::compiler",
        summary = "Build the command that arms or releases a compiled breakpoint.",
        context = "Controls a label that was explicitly declared in the matching program.",
        minecraft = "Sets the canonical debugger State score for the breakpoint.",
        use_when = ["Connecting an editor or local console to Sand's debugger"],
        avoid_when = ["Setting arbitrary gameplay scores"],
        params(program = "Resolved program used to build the installed pack.", label = "Authored breakpoint label.", armed = "Whether new and pending invocations wait here."),
        returns = "A single checked console command, or diagnostics.",
        example = "use sand::advanced::compiler::{Compiler, Program}; fn arm(program: &Program) { let _ = Compiler::breakpoint_control(program, \"reward\", true); }",
    )]
    pub fn breakpoint_control(
        program: &Program,
        label: &str,
        armed: bool,
    ) -> Result<String, Vec<Diagnostic>> {
        validate::validate(program)?;
        let id = format!("{}:sand.breakpoints", program.pack.namespace);
        let state = program
            .modules
            .iter()
            .flat_map(|module| &module.states)
            .find(|state| {
                state.scope == "global"
                    && state.id.to_string() == id
                    && state.fields.iter().any(|field| field.name == label)
            })
            .ok_or_else(|| {
                vec![Diagnostic::error(
                    "SAND_PROGRAM_BREAKPOINT",
                    &id,
                    "",
                    format!("unknown breakpoint `{label}`"),
                )]
            })?;
        let objective = sand_commands::ObjectiveName::logical(format!("{id}.{label}"));
        let holder = crate::state::owned::holder(state);
        Ok(format!(
            "scoreboard players set {holder} {objective} {}",
            i32::from(armed)
        ))
    }

    /// Find the native pending queue or ownership ledger for each compiled wait.
    ///
    /// Run the returned command against the world with this exact pack installed.
    /// Empty owned ledgers may be absent until a wait has been admitted.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::advanced::compiler::Compiler::wait_inspection", module = "sand::advanced::compiler",
        summary = "Find read-only queries for compiled pending waits.",
        context = "Server waits use queues and executor-owned waits use retained ownership ledgers.",
        minecraft = "Returns inspection commands without adding functions or changing datapack bytes.",
        use_when = ["Connecting a debugger to the compiled wait runtime"],
        avoid_when = ["Inspecting gameplay state fields"],
        params(program = "Resolved owned program compiled for the installed target."),
        returns = "Pending-wait probes in stable site order, or compilation diagnostics.",
        example = "use sand::advanced::compiler::{Compiler, Program}; fn probe(program: &Program) { let _ = Compiler::wait_inspection(program); }",
    )]
    pub fn wait_inspection(program: &Program) -> Result<Vec<WaitInspection>, Vec<Diagnostic>> {
        let target = validate::validate(program)?;
        let mut sites = lower::lower(program, &target, None)?.wait_sites;
        sites.sort_by(|a, b| a.site.cmp(&b.site));
        Ok(sites)
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
