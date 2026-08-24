//! Unified typed state transitions and enter/exit/tick hooks.
//!
//! Register a flow from a normal exported component factory:
//!
//! ```rust,ignore
//! #[datapack_component(Load)]
//! fn register_boss_flow() {
//!     StateFlow::players(&PHASE)
//!         .transition(BossPhase::Fighting, BossPhase::Enraged)
//!         .when(HEALTH.of("@s").lte(50))
//!         .priority(100)
//!         .done()
//!         .on_enter(BossPhase::Enraged, cmd::call(start_enrage))
//!         .on_exit(BossPhase::Enraged, cmd::call(stop_enrage))
//!         .on_tick_every(BossPhase::Enraged, Ticks::new(5), cmd::call(enraged_tick))
//!         .register();
//! }
//! ```
//!
//! Higher-priority transitions run first. Equal priorities preserve declaration
//! order. A private per-subject lock prevents lower-priority transitions from
//! running after one succeeds in the same cycle. Enter/exit hooks run only for
//! transitions owned by this flow; direct low-level score writes deliberately
//! bypass them. Tick hooks run after transitions, so a newly entered state can
//! tick in the same server tick.

use std::any::type_name;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, OnceLock};

use sand_commands::Selector;

use crate::condition::Condition;
use crate::state::{GameState, GameStateRef, Ticks, TypedGameState};

#[doc = "**API Contract:** Run `sand api show sand::state::IntoStateCommands` for the canonical contract."]
/// Convert hook commands into one deterministic command list.
pub trait IntoStateCommands {
    /// Converts a hook body into the deterministic command sequence emitted for a state transition.
    #[doc = "**API Contract:** Run `sand api show sand::state::IntoStateCommands::into_state_commands` for the canonical contract."]
    fn into_state_commands(self) -> Vec<String>;
}

impl IntoStateCommands for String {
    fn into_state_commands(self) -> Vec<String> {
        vec![self]
    }
}

impl IntoStateCommands for &str {
    fn into_state_commands(self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl IntoStateCommands for Vec<String> {
    fn into_state_commands(self) -> Vec<String> {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FlowTransition {
    from: i32,
    to: i32,
    guard_plans: Vec<Vec<String>>,
    priority: i32,
    order: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HookKind {
    Enter,
    Exit,
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FlowHook {
    kind: HookKind,
    state: i32,
    commands: Vec<String>,
    cadence: u32,
    order: usize,
}

#[doc = "**API Contract:** Run `sand api show sand::state::StateFlow` for the canonical contract."]
/// One cohesive state machine registration.
pub struct StateFlow<S: TypedGameState> {
    id: String,
    state_type: &'static str,
    objective: String,
    default_score: Option<i32>,
    subjects: Selector,
    transitions: Vec<FlowTransition>,
    hooks: Vec<FlowHook>,
    next_order: usize,
    marker: std::marker::PhantomData<fn() -> S>,
}

impl<S: TypedGameState> StateFlow<S> {
    /// Create a player-scoped flow. The generated tick entry scans `@a`, binds
    /// each subject to `@s`, and evaluates that subject independently.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateFlow::players` for the canonical contract."]
    pub fn players(state: &GameState<S>) -> Self {
        Self::for_subjects(state, Selector::all_players())
    }

    /// Create a flow for an explicit selector.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateFlow::for_subjects` for the canonical contract."]
    pub fn for_subjects(state: &GameState<S>, subjects: Selector) -> Self {
        let objective = state.objective_name();
        let state_type = type_name::<S>();
        let id = format!("{state_type}:{objective}:{subjects}");
        Self {
            id,
            state_type,
            objective,
            default_score: state.default_score(),
            subjects,
            transitions: Vec::new(),
            hooks: Vec::new(),
            next_order: 0,
            marker: std::marker::PhantomData,
        }
    }

    /// Add a stable user label to the generated identity.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateFlow::named` for the canonical contract."]
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.id.push(':');
        self.id.push_str(&name.into());
        self
    }

    /// Starts a transition from one typed state variant to another.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateFlow::transition` for the canonical contract."]
    pub fn transition(self, from: S, to: S) -> FlowTransitionBuilder<S> {
        FlowTransitionBuilder {
            flow: self,
            from,
            to,
            guard: None,
            priority: 0,
        }
    }

    /// Registers commands that run whenever the selected state is entered.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateFlow::on_enter` for the canonical contract."]
    pub fn on_enter(mut self, state: S, commands: impl IntoStateCommands) -> Self {
        self.push_hook(HookKind::Enter, state, 1, commands);
        self
    }

    /// Registers commands that run whenever the selected state is exited.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateFlow::on_exit` for the canonical contract."]
    pub fn on_exit(mut self, state: S, commands: impl IntoStateCommands) -> Self {
        self.push_hook(HookKind::Exit, state, 1, commands);
        self
    }

    /// Run every server tick while the subject remains in `state`.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateFlow::on_tick` for the canonical contract."]
    pub fn on_tick(mut self, state: S, commands: impl IntoStateCommands) -> Self {
        self.push_hook(HookKind::Tick, state, 1, commands);
        self
    }

    /// Run every `cadence` matching ticks. The counter resets when the subject
    /// leaves the state; cadence zero is diagnosed during export.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateFlow::on_tick_every` for the canonical contract."]
    pub fn on_tick_every(
        mut self,
        state: S,
        cadence: Ticks,
        commands: impl IntoStateCommands,
    ) -> Self {
        self.push_hook(HookKind::Tick, state, cadence.get(), commands);
        self
    }

    fn push_hook(
        &mut self,
        kind: HookKind,
        state: S,
        cadence: u32,
        commands: impl IntoStateCommands,
    ) {
        let hook = FlowHook {
            kind,
            state: state.to_score(),
            commands: commands.into_state_commands(),
            cadence,
            order: self.next_order,
        };
        self.next_order += 1;
        if !self.hooks.iter().any(|existing| {
            existing.kind == hook.kind
                && existing.state == hook.state
                && existing.commands == hook.commands
                && existing.cadence == hook.cadence
        }) {
            self.hooks.push(hook);
        }
    }

    /// Register this flow for the current export. Returns an empty command list
    /// so it composes naturally inside `#[datapack_component(Load)]`.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateFlow::register` for the canonical contract."]
    pub fn register(self) -> Vec<String> {
        register_flow(self.erase());
        Vec::new()
    }

    fn erase(self) -> FlowRegistration {
        FlowRegistration {
            id: self.id,
            state_type: self.state_type,
            objective: self.objective,
            default_score: self.default_score,
            subjects: self.subjects.to_string(),
            transitions: self.transitions,
            hooks: self.hooks,
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::state::FlowTransitionBuilder` for the canonical contract."]
/// Nested builder for one transition.
pub struct FlowTransitionBuilder<S: TypedGameState> {
    flow: StateFlow<S>,
    from: S,
    to: S,
    guard: Option<Condition>,
    priority: i32,
}

impl<S: TypedGameState> FlowTransitionBuilder<S> {
    /// Restricts this transition to invocations where the guard holds.
    #[doc = "**API Contract:** Run `sand api show sand::state::FlowTransitionBuilder::when` for the canonical contract."]
    pub fn when(mut self, guard: Condition) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Sets this transition's ordering priority relative to competing transitions.
    #[doc = "**API Contract:** Run `sand api show sand::state::FlowTransitionBuilder::priority` for the canonical contract."]
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Commits this transition and returns the completed state-flow definition.
    #[doc = "**API Contract:** Run `sand api show sand::state::FlowTransitionBuilder::done` for the canonical contract."]
    pub fn done(mut self) -> StateFlow<S> {
        let guard_plans = match self.guard {
            Some(guard) => guard
                .rendered_plans(false)
                .into_iter()
                .map(|plan| plan.into_iter().collect())
                .collect(),
            None => vec![Vec::new()],
        };
        let transition = FlowTransition {
            from: self.from.to_score(),
            to: self.to.to_score(),
            guard_plans,
            priority: self.priority,
            order: self.flow.next_order,
        };
        self.flow.next_order += 1;
        if !self.flow.transitions.iter().any(|existing| {
            existing.from == transition.from
                && existing.to == transition.to
                && existing.guard_plans == transition.guard_plans
                && existing.priority == transition.priority
        }) {
            self.flow.transitions.push(transition);
        }
        self.flow
    }
}

#[doc = "**API Contract:** Run `sand api show sand::state::StateTransitionBuilder` for the canonical contract."]
/// Compact one-off guarded transition builder on [`GameStateRef`].
pub struct StateTransitionBuilder<'a, S: TypedGameState> {
    state: &'a GameStateRef<'a, S>,
    from: Option<S>,
    guard: Option<Condition>,
}

impl<'a, S: TypedGameState> StateTransitionBuilder<'a, S> {
    pub(crate) fn new(state: &'a GameStateRef<'a, S>) -> Self {
        Self {
            state,
            from: None,
            guard: None,
        }
    }

    /// Restricts the transition builder to a specific source state.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateTransitionBuilder::from` for the canonical contract."]
    pub fn from(mut self, state: S) -> Self {
        self.from = Some(state);
        self
    }

    /// Restricts this state transition to invocations where the guard holds.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateTransitionBuilder::when` for the canonical contract."]
    pub fn when(mut self, guard: Condition) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Emit deterministic guarded write commands using the same low-level
    /// `GameStateRef::set` representation as manual state writes.
    #[doc = "**API Contract:** Run `sand api show sand::state::StateTransitionBuilder::to` for the canonical contract."]
    pub fn to(self, state: S) -> Vec<String> {
        let mut condition = self
            .from
            .map(|from| self.state.is(from))
            .unwrap_or_else(|| Condition::entity_raw(self.state.selector()));
        if let Some(guard) = self.guard {
            condition = condition.and(guard);
        }
        condition.execute_commands(false, &self.state.set(state))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowRegistration {
    id: String,
    state_type: &'static str,
    objective: String,
    default_score: Option<i32>,
    subjects: String,
    transitions: Vec<FlowTransition>,
    hooks: Vec<FlowHook>,
}

fn registry() -> &'static Mutex<Vec<FlowRegistration>> {
    static REGISTRY: OnceLock<Mutex<Vec<FlowRegistration>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

fn register_flow(flow: FlowRegistration) {
    registry()
        .lock()
        .expect("state flow registry poisoned")
        .push(flow);
}

pub(crate) fn drain_flows() -> Vec<FlowRegistration> {
    std::mem::take(&mut *registry().lock().expect("state flow registry poisoned"))
}

pub(crate) fn resolve_state_flow_plan(
    namespace: &str,
    registrations: Vec<FlowRegistration>,
) -> Result<crate::transition::TransitionPlan, String> {
    let mut by_id: BTreeMap<String, FlowRegistration> = BTreeMap::new();
    for registration in registrations {
        match by_id.get(&registration.id) {
            Some(existing) if existing == &registration => {}
            Some(existing) => {
                return Err(flow_error(
                    &registration,
                    format!(
                        "duplicate flow identity conflicts with an earlier registration for selector `{}`",
                        existing.subjects
                    ),
                    "give the flows distinct `.named(...)` labels or register one canonical flow",
                ));
            }
            None => {
                by_id.insert(registration.id.clone(), registration);
            }
        }
    }

    let mut plan = crate::transition::TransitionPlan::default();
    let mut generated_paths = BTreeSet::new();
    for (_, mut flow) in by_id {
        validate_flow(&flow)?;
        flow.transitions
            .sort_by_key(|transition| (std::cmp::Reverse(transition.priority), transition.order));
        flow.hooks.sort_by_key(|hook| hook.order);

        let key = flow_key(&flow.id);
        let root = format!("__sand_transition/flow_{key}");
        let lock = format!("__sf_{key}r");
        plan.load_commands.push(format!(
            "scoreboard objectives add {} dummy",
            flow.objective
        ));
        plan.load_commands
            .push(format!("scoreboard objectives add {lock} dummy"));
        plan.private_objectives.insert(
            lock.clone(),
            (
                flow.id.clone(),
                format!("state flow for {}", flow.state_type),
            ),
        );
        plan.global_tick_commands.push(format!(
            "execute as {} at @s run function {namespace}:{root}",
            flow.subjects
        ));

        let mut root_commands = Vec::new();
        if let Some(default_score) = flow.default_score {
            root_commands.push(format!(
                "execute unless score @s {} matches -2147483648..2147483647 run scoreboard players set @s {} {default_score}",
                flow.objective, flow.objective
            ));
        }
        root_commands.push(format!("scoreboard players set @s {lock} 0"));
        for (index, transition) in flow.transitions.iter().enumerate() {
            let helper = format!("{root}/transition_{index}");
            insert_path(&mut generated_paths, &flow, &helper)?;
            let mut commands = hook_commands(&flow.hooks, HookKind::Exit, transition.from);
            commands.push(format!(
                "scoreboard players set @s {} {}",
                flow.objective, transition.to
            ));
            commands.extend(hook_commands(&flow.hooks, HookKind::Enter, transition.to));
            commands.push(format!("scoreboard players set @s {lock} 1"));
            plan.functions
                .push(crate::transition::GeneratedTransitionFunction {
                    tracker_id: flow.id.clone(),
                    source: format!(
                        "{} {} -> {} priority {}",
                        flow.state_type, transition.from, transition.to, transition.priority
                    ),
                    path: helper.clone(),
                    commands,
                });

            for guard in &transition.guard_plans {
                let guards = if guard.is_empty() {
                    String::new()
                } else {
                    format!(" {}", guard.join(" "))
                };
                root_commands.push(format!(
                    "execute if score @s {lock} matches 0 if score @s {} matches {}{} run function {namespace}:{helper}",
                    flow.objective, transition.from, guards
                ));
            }
        }

        for (index, hook) in flow
            .hooks
            .iter()
            .filter(|hook| hook.kind == HookKind::Tick)
            .enumerate()
        {
            let helper = format!("{root}/tick_{index}");
            insert_path(&mut generated_paths, &flow, &helper)?;
            let mut commands = hook.commands.clone();
            if hook.cadence > 1 {
                let counter = format!("__sf_{key}t{index:x}");
                if counter.len() > 16 {
                    return Err(flow_error(
                        &flow,
                        "too many interval tick hooks generated an objective longer than 16 characters",
                        "split the flow or reduce interval hook registrations",
                    ));
                }
                plan.load_commands
                    .push(format!("scoreboard objectives add {counter} dummy"));
                plan.private_objectives.insert(
                    counter.clone(),
                    (flow.id.clone(), format!("interval tick hook {index}")),
                );
                root_commands.push(format!(
                    "execute unless score @s {} matches {} run scoreboard players set @s {counter} 0",
                    flow.objective, hook.state
                ));
                root_commands.push(format!(
                    "execute if score @s {} matches {} run scoreboard players add @s {counter} 1",
                    flow.objective, hook.state
                ));
                root_commands.push(format!(
                    "execute if score @s {} matches {} if score @s {counter} matches {}.. run function {namespace}:{helper}",
                    flow.objective, hook.state, hook.cadence
                ));
                commands.push(format!("scoreboard players set @s {counter} 0"));
            } else {
                root_commands.push(format!(
                    "execute if score @s {} matches {} run function {namespace}:{helper}",
                    flow.objective, hook.state
                ));
            }
            plan.functions
                .push(crate::transition::GeneratedTransitionFunction {
                    tracker_id: flow.id.clone(),
                    source: format!(
                        "{} tick state {} cadence {}",
                        flow.state_type, hook.state, hook.cadence
                    ),
                    path: helper,
                    commands,
                });
        }

        insert_path(&mut generated_paths, &flow, &root)?;
        plan.functions
            .push(crate::transition::GeneratedTransitionFunction {
                tracker_id: flow.id.clone(),
                source: format!("state flow for {}", flow.state_type),
                path: root,
                commands: root_commands,
            });
    }
    Ok(plan)
}

fn validate_flow(flow: &FlowRegistration) -> Result<(), String> {
    if flow.transitions.is_empty() && flow.hooks.is_empty() {
        return Err(flow_error(
            flow,
            "flow has no transitions or hooks",
            "add a transition/hook or remove the empty registration",
        ));
    }
    let mut conflicts: BTreeMap<(i32, i32, Vec<Vec<String>>), i32> = BTreeMap::new();
    for transition in &flow.transitions {
        if transition.from == transition.to {
            return Err(flow_error(
                flow,
                format!(
                    "transition {} -> {} has identical source and destination",
                    transition.from, transition.to
                ),
                "remove the transition or choose a different target state",
            ));
        }
        let key = (
            transition.from,
            transition.priority,
            transition.guard_plans.clone(),
        );
        if let Some(existing_to) = conflicts.insert(key, transition.to)
            && existing_to != transition.to
        {
            return Err(flow_error(
                flow,
                format!(
                    "conflicting transitions from score {} share the same guard and priority {} but target {} and {}",
                    transition.from, transition.priority, existing_to, transition.to
                ),
                "assign distinct priorities or make the guards mutually exclusive",
            ));
        }
    }
    for hook in &flow.hooks {
        if hook.commands.is_empty() {
            return Err(flow_error(
                flow,
                format!(
                    "{:?} hook for score {} has no commands",
                    hook.kind, hook.state
                ),
                "remove the hook or provide at least one command",
            ));
        }
        if hook.kind == HookKind::Tick && hook.cadence == 0 {
            return Err(flow_error(
                flow,
                format!("tick hook for score {} has zero cadence", hook.state),
                "use Ticks::new(1) or a larger supported interval",
            ));
        }
    }
    Ok(())
}

fn hook_commands(hooks: &[FlowHook], kind: HookKind, state: i32) -> Vec<String> {
    hooks
        .iter()
        .filter(|hook| hook.kind == kind && hook.state == state)
        .flat_map(|hook| hook.commands.clone())
        .collect()
}

fn insert_path(
    paths: &mut BTreeSet<String>,
    flow: &FlowRegistration,
    path: &str,
) -> Result<(), String> {
    if paths.insert(path.to_string()) {
        Ok(())
    } else {
        Err(flow_error(
            flow,
            format!("generated helper path collision at `{path}`"),
            "add a distinct `.named(...)` label",
        ))
    }
}

fn flow_error(flow: &FlowRegistration, problem: impl AsRef<str>, fix: &str) -> String {
    format!(
        "error[SAND-STATE-FLOW]: invalid state flow\n\nState type: {}\nObjective: {}\nSelector: {}\nFlow: {}\n\n{}\n\nCorrection: {fix}",
        flow.state_type,
        flow.objective,
        flow.subjects,
        flow.id,
        problem.as_ref()
    )
}

fn flow_key(id: &str) -> String {
    format!("{:08x}", fnv1a(id) & 0xFFFF_FFFF)
}

fn fnv1a(value: &str) -> u64 {
    let mut hash = 14_695_981_039_346_656_037;
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Phase {
        Idle = 0,
        Fighting = 1,
        Enraged = 2,
        Defeated = 3,
    }

    impl TypedGameState for Phase {
        fn to_score(self) -> i32 {
            self as i32
        }

        fn from_score(score: i32) -> Option<Self> {
            match score {
                0 => Some(Self::Idle),
                1 => Some(Self::Fighting),
                2 => Some(Self::Enraged),
                3 => Some(Self::Defeated),
                _ => None,
            }
        }
    }

    static PHASE: GameState<Phase> = GameState::with_default_score("boss_phase", 0);

    #[test]
    fn one_off_builder_matches_low_level_state_write() {
        let state = PHASE.of("@s");
        assert_eq!(
            state
                .transition()
                .from(Phase::Fighting)
                .when(Condition::entity_raw("@s[tag=low]"))
                .to(Phase::Enraged),
            vec![
                "execute if score @s boss_phase matches 1 if entity @s[tag=low] run scoreboard players set @s boss_phase 2"
            ]
        );
    }

    #[test]
    fn flow_priority_hook_order_and_tick_cadence_are_deterministic() {
        let flow = StateFlow::players(&PHASE)
            .transition(Phase::Fighting, Phase::Enraged)
            .when(Condition::entity_raw("@s[tag=low]"))
            .priority(100)
            .done()
            .transition(Phase::Fighting, Phase::Defeated)
            .when(Condition::entity_raw("@s[tag=dead]"))
            .priority(50)
            .done()
            .on_exit(Phase::Fighting, "say exit")
            .on_enter(Phase::Enraged, "say enter")
            .on_tick(Phase::Enraged, "say tick")
            .on_tick_every(Phase::Enraged, Ticks::new(5), "say every five")
            .erase();
        let plan = resolve_state_flow_plan("boss", vec![flow]).unwrap();
        let root = plan
            .functions
            .iter()
            .find(|function| function.path.matches('/').count() == 1)
            .unwrap();
        let high = root
            .commands
            .iter()
            .position(|command| command.contains("tag=low"))
            .unwrap();
        let low = root
            .commands
            .iter()
            .position(|command| command.contains("tag=dead"))
            .unwrap();
        assert!(high < low);
        assert!(
            root.commands
                .iter()
                .any(|command| command.contains("matches 5.."))
        );
        let transition = plan
            .functions
            .iter()
            .find(|function| function.path.ends_with("transition_0"))
            .unwrap();
        assert_eq!(
            transition.commands[..3],
            [
                "say exit",
                "scoreboard players set @s boss_phase 2",
                "say enter"
            ]
        );
    }

    #[test]
    fn conflicting_equal_priority_transitions_are_diagnosed() {
        let flow = StateFlow::players(&PHASE)
            .transition(Phase::Fighting, Phase::Enraged)
            .priority(1)
            .done()
            .transition(Phase::Fighting, Phase::Defeated)
            .priority(1)
            .done()
            .erase();
        let error = resolve_state_flow_plan("boss", vec![flow]).unwrap_err();
        assert!(error.contains("error[SAND-STATE-FLOW]"));
        assert!(error.contains("same guard and priority"));
    }
}
