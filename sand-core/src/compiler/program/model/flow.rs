//! Owned gameplay state machines compiled by the shared Rust StateFlow backend.
use super::{FieldReference, FunctionReference, ScoreComparison};
use sand_components::ResourceLocation;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A state machine bound to one player or global integer State field.
///
/// The shared Rust/portable lifecycle runs automatic State maintenance, then
/// flows, then authored module tick actions. Writes from a module's tick body
/// therefore affect flow selection on the following cycle. Within a flow,
/// transitions run before tick hooks, and later hooks observe earlier writes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::Flow", module = "sand::advanced::compiler",
 summary = "A state machine bound to one player or global integer State field.",
 context = "Owned StateFlow definitions use the same transition selection and ordered hook compiler as Rust authoring.",
 minecraft = "Runs deterministic per-player or global transitions and named function hooks through canonical State lifecycle and flow lowering.",
 use_when = ["Authoring gameplay state machines from a language or interface without Cargo"],
 avoid_when = ["A one-off conditional action does not need a persistent state machine"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(id = "Stable namespaced flow identity.", state = "Canonical State field whose integer value selects the current phase.", transitions = "Priority-ordered transitions; ties preserve declaration order.", hooks = "Ordered enter, exit, and tick function hooks."),
)]
pub struct Flow {
    /// Stable identity owning generated private helpers and interval counters.
    #[schemars(with = "String")]
    pub id: ResourceLocation,
    /// The field's owning scope determines execution: each player or one server
    /// invocation for global State. Lifecycle initializes its declared default.
    pub state: FieldReference,
    /// At most one transition executes per cycle. Equal-priority guards retain
    /// declaration order; identical guards targeting different states conflict.
    #[serde(default)]
    pub transitions: Vec<FlowTransition>,
    /// Hooks retain authored order. Tick hooks observe state changes made by
    /// earlier hooks in the same cycle; raw score writes bypass enter/exit hooks.
    #[serde(default)]
    pub hooks: Vec<FlowHook>,
}

/// One guarded transition between integer phases.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::FlowTransition", module = "sand::advanced::compiler",
 summary = "One guarded transition between integer phases.",
 context = "Owned StateFlow definitions use the same transition selection and ordered hook compiler as Rust authoring.",
 minecraft = "Runs deterministic per-player or global transitions and named function hooks through canonical State lifecycle and flow lowering.",
 use_when = ["Authoring gameplay state machines from a language or interface without Cargo"],
 avoid_when = ["A one-off conditional action does not need a persistent state machine"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(from = "Current phase required for this transition.", to = "Destination phase written between exit and enter hooks.", when = "Optional inclusive score comparison evaluated in the flow context.", priority = "Higher priorities execute before lower ones."),
)]
pub struct FlowTransition {
    /// Required current phase; arbitrary signed integer encodings are supported.
    pub from: i32,
    /// Exact destination phase; must differ from the source and satisfy any
    /// bounds on the owning State field.
    pub to: i32,
    /// No guard means unconditional once the source phase is selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<ScoreComparison>,
    /// Higher priorities first; ties retain declaration order.
    #[serde(default)]
    pub priority: i32,
}

/// A named function invoked at a state-machine lifecycle point.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::FlowHook", module = "sand::advanced::compiler",
 summary = "A named function invoked at a state-machine lifecycle point.",
 context = "Owned StateFlow definitions use the same transition selection and ordered hook compiler as Rust authoring.",
 minecraft = "Runs deterministic per-player or global transitions and named function hooks through canonical State lifecycle and flow lowering.",
 use_when = ["Authoring gameplay state machines from a language or interface without Cargo"],
 avoid_when = ["A one-off conditional action does not need a persistent state machine"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(phase = "Enter, exit, or tick execution point.", state = "Integer phase associated with this hook.", function = "Validated internal or explicitly external function reference.", cadence = "Matching ticks between tick invocations; enter/exit hooks require one."),
)]
pub struct FlowHook {
    /// Enter/exit hooks run only on flow-owned transitions.
    pub phase: FlowHookPhase,
    /// Phase that triggers this hook.
    pub state: i32,
    /// Inherits the flow's player or server execution context.
    pub function: FunctionReference,
    /// Tick hooks count matching cycles, including the entering cycle. Counters
    /// reset when a cycle observes another phase. Range: 1 through i32::MAX.
    /// Enter and exit hooks require the default value of one.
    #[serde(default = "every_tick")]
    pub cadence: u32,
}

fn every_tick() -> u32 {
    1
}

/// The state-machine lifecycle point that invokes a hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::FlowHookPhase", module = "sand::advanced::compiler",
 summary = "The state-machine lifecycle point that invokes a hook.",
 context = "Owned StateFlow definitions use the same transition selection and ordered hook compiler as Rust authoring.",
 minecraft = "Runs deterministic per-player or global transitions and named function hooks through canonical State lifecycle and flow lowering.",
 use_when = ["Authoring gameplay state machines from a language or interface without Cargo"],
 avoid_when = ["A one-off conditional action does not need a persistent state machine"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Enter = "After a flow-owned transition writes this phase.", Exit = "Before a flow-owned transition leaves this phase.", Tick = "After transition selection on each matching cycle."),
)]
pub enum FlowHookPhase {
    /// After a flow-owned transition writes this phase.
    Enter,
    /// Before a flow-owned transition leaves this phase.
    Exit,
    /// After transition selection, in authored hook order.
    Tick,
}
