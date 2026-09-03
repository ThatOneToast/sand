//! Target query capabilities and lowering into typed execution-scoped contexts.

use sand_commands::Selector;
use sand_commands::TargetArgument;
use sand_commands::selector::{AnyTarget, PlayersOnly, Target};

use crate::entity::context::EntityContext;
use crate::entity::kind::{AnyEntity, PlayerKind};
use crate::function::register_dyn_fn_dedup;

/// Owner category used by generated State query implementations.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateQueryScope {
    Entity,
    Player,
}

/// Scope proof for State components and bundles that can be queried as an
/// entity collection. Global State deliberately does not implement this
/// contract.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "global State is a singleton resource and cannot be used as an entity query",
    label = "use the generated `global()` accessor for global State"
)]
pub trait QueryableStateScope: crate::entity::state::StateScopeMarker {
    const QUERY_SCOPE: StateQueryScope;
}

impl QueryableStateScope for crate::entity::state::EntityStateScope {
    const QUERY_SCOPE: StateQueryScope = StateQueryScope::Entity;
}

impl QueryableStateScope for crate::entity::state::LivingStateScope {
    const QUERY_SCOPE: StateQueryScope = StateQueryScope::Entity;
}

impl QueryableStateScope for crate::entity::state::PlayerStateScope {
    const QUERY_SCOPE: StateQueryScope = StateQueryScope::Player;
}

/// Compiler-facing contract implemented by `#[derive(StateQuery)]` and, via
/// the shared bundle-member contract, every queryable State and StateBundle.
#[doc(hidden)]
pub trait StateQuerySpec: 'static {
    type Item;

    fn each(body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String>;

    /// Run a query body against the current Minecraft executor.
    ///
    /// Unlike [`StateQuerySpec::each`], this does not create an entity scan.
    /// Generated implementations guard every command with the query's
    /// required and forbidden component-presence predicates. This is the
    /// canonical bridge for event handlers, whose dispatcher has already
    /// selected and bound the event owner as `@s`.
    fn current(body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String>;
}

impl<T> StateQuerySpec for T
where
    T: crate::entity::state::StateBundleMember,
    T::Scope: QueryableStateScope,
{
    type Item = T::Bound;

    fn each(body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String> {
        lower_state_query_each(
            <T::Scope as QueryableStateScope>::QUERY_SCOPE,
            T::presence_requirements(),
            Vec::new(),
            T::bind_member("@s"),
            body,
        )
    }

    fn current(body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String> {
        lower_state_query_current(
            T::presence_requirements(),
            Vec::new(),
            T::bind_member("@s"),
            body,
        )
    }
}

/// Source-level query operations for scoped State components, State bundles,
/// and composed StateQuery declarations.
///
/// The prelude imports this trait so `query.each(...)` and
/// `query.current(...)` remain discoverable on a `#[system]` parameter before
/// macro expansion. Global State does not implement this trait.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::StateQueryOperations",
    aliases = ["sand::prelude::StateQueryOperations"],
    module = "sand::entity",
    summary = "Provides source-level `each` and `current` operations for scoped State components, State bundles, and composed StateQuery declarations.",
    context = "The prelude imports this extension trait so direct State and StateBundle system parameters expose their concrete bound views in completion and hover documentation. Entity, living, and player scopes implement the query contract; global State remains a singleton resource accessed through `global()`.",
    minecraft = "Direct State queries test the component's existing presence/version objective. Bundle queries require every flattened component marker, while composed StateQuery declarations retain their required, optional, and forbidden rules. No additional storage, lifecycle, or component marker is created.",
    use_when = ["Iterating owners that possess one scoped State component", "Requiring every component in a StateBundle", "Applying a query to an event dispatcher’s already-current executor"],
    avoid_when = ["Combining optional or forbidden components; derive StateQuery for composition", "Accessing global State; use the generated global method"],
    example = "use sand::prelude::*;\n\n#[derive(State)]\n#[state(namespace = \"rpg\", scope = entity)]\nstruct Health {\n    #[state(default = 20)]\n    current: Score,\n}\n\nfn regenerate(query: Health) -> Vec<String> {\n    query.each(|health| health.current.add(1))\n}",
)]
pub trait StateQueryOperations: StateQuerySpec + Sized {
    /// Iterate over owners matching this query and expose its concrete bound item.
    ///
    /// This creates the scope-appropriate outer scan (`@e` for entity/living
    /// State and `@a` for player State), filters by real component presence,
    /// and binds the closure argument to the matching owner.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StateQueryOperations::each",
        aliases = ["sand::prelude::StateQueryOperations::each"],
        module = "sand::entity",
        kind = "trait_method",
        summary = "Scans owners matching this query and passes each query's concrete bound item to the body.",
        context = "On a direct State query the item is that State's normal bound view. On a StateBundle it is the normal nested bundle view, with every flattened component required. A derived StateQuery instead yields its generated composition item.",
        minecraft = "Emits one scope-appropriate entity or player scan filtered by the canonical component presence/version objectives, then invokes a deduplicated generated function as and at each matching owner. Compatible adjacent systems continue to share the existing scan plan.",
        use_when = ["A tick or load-time system needs to visit every owner matching a State, StateBundle, or StateQuery"],
        avoid_when = ["An event dispatcher already established the executor; use current", "Querying global singleton State"],
        params(body = "Receives the concrete holder-bound query item and returns the commands to emit for each match."),
        returns = "The generated outer-scan command, or no commands when the body emits nothing.",
        example = "use sand::prelude::*;\n\n#[derive(State)]\n#[state(namespace = \"rpg\", scope = living)]\nstruct Health {\n    #[state(default = 20)]\n    current: Score,\n}\n\nfn regenerate(query: Health) -> Vec<String> {\n    query.each(|health| health.current.add(1))\n}",
    )]
    fn each(self, body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String> {
        <Self as StateQuerySpec>::each(body)
    }

    /// Apply this query to the already-current Minecraft executor without scanning.
    ///
    /// Every emitted body command is guarded by this query's required and
    /// forbidden component-presence predicates against `@s`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StateQueryOperations::current",
        aliases = ["sand::prelude::StateQueryOperations::current"],
        module = "sand::entity",
        kind = "trait_method",
        summary = "Applies this query to the already-current Minecraft executor without creating an outer entity scan.",
        context = "Event dispatch establishes the relevant owner as `@s` before the handler runs. Direct State and bundle parameters therefore expose their concrete bound views while preserving the same presence semantics as `each`.",
        minecraft = "Guards every body command against the query's required and forbidden component presence/version objectives on `@s`; it emits no `execute as @e` or `execute as @a` scan.",
        use_when = ["An event system or other dispatch path has already selected the owner as the current executor"],
        avoid_when = ["The system must discover matching owners; use each", "Querying global singleton State"],
        params(body = "Receives the concrete holder-bound query item for `@s` and returns commands to guard and emit."),
        returns = "The body commands guarded by this query's presence predicates for the current executor.",
        example = "use sand::prelude::*;\n\n#[derive(State)]\n#[state(namespace = \"rpg\", scope = player)]\nstruct Health {\n    #[state(default = 20)]\n    current: Score,\n}\n\nfn heal_current(query: Health) -> Vec<String> {\n    query.current(|health| health.current.add(1))\n}",
    )]
    fn current(self, body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String> {
        <Self as StateQuerySpec>::current(body)
    }
}

impl<Q: StateQuerySpec> StateQueryOperations for Q {}

/// Canonical lowering shared by generated State, StateBundle, and StateQuery
/// implementations.
#[doc(hidden)]
pub fn lower_state_query_each<Item>(
    scope: StateQueryScope,
    mut requirements: Vec<(String, u32)>,
    mut forbidden: Vec<(String, u32)>,
    item: Item,
    body: impl FnOnce(Item) -> Vec<String>,
) -> Vec<String> {
    requirements.sort();
    requirements.dedup();
    forbidden.sort();
    forbidden.dedup();

    let mut selector = match scope {
        StateQueryScope::Entity => Selector::all_entities(),
        StateQueryScope::Player => Selector::all_players(),
    };
    for (objective, version) in requirements {
        selector = selector
            .score_typed(
                sand_commands::ObjectiveName::try_dynamic(objective)
                    .expect("State derives validated presence objectives"),
                sand_commands::selector::ScoreRange::exact(version as i32),
            )
            .expect("State queries contain unique presence filters");
    }

    let inner = guard_state_query_commands(body(item), &[], &forbidden);
    if inner.is_empty() {
        return Vec::new();
    }
    let path = register_dyn_fn_dedup("sand/entity_query", inner);
    vec![format!(
        "execute as {selector} at @s run function __sand_local:{path}"
    )]
}

/// Canonical current-executor lowering shared by every generated State query.
#[doc(hidden)]
pub fn lower_state_query_current<Item>(
    mut requirements: Vec<(String, u32)>,
    mut forbidden: Vec<(String, u32)>,
    item: Item,
    body: impl FnOnce(Item) -> Vec<String>,
) -> Vec<String> {
    requirements.sort();
    requirements.dedup();
    forbidden.sort();
    forbidden.dedup();
    guard_state_query_commands(body(item), &requirements, &forbidden)
}

fn guard_state_query_commands(
    commands: Vec<String>,
    requirements: &[(String, u32)],
    forbidden: &[(String, u32)],
) -> Vec<String> {
    commands
        .into_iter()
        .map(|command| {
            let mut guards = requirements
                .iter()
                .map(|(objective, version)| format!("if score @s {objective} matches {version}"))
                .collect::<Vec<_>>();
            guards.extend(forbidden.iter().map(|(objective, version)| {
                format!("unless score @s {objective} matches {version}")
            }));
            if guards.is_empty() {
                command
            } else {
                format!("execute {} run {command}", guards.join(" "))
            }
        })
        .collect()
}

/// Generated zero-sized query parameter used inside `#[system]` bodies.
#[doc(hidden)]
pub struct StateQueryHandle<Q>(std::marker::PhantomData<fn() -> Q>);

impl<Q> StateQueryHandle<Q> {
    #[doc(hidden)]
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<Q: StateQuerySpec> StateQueryHandle<Q> {
    #[doc(hidden)]
    pub fn each(self, body: impl FnOnce(Q::Item) -> Vec<String>) -> Vec<String> {
        Q::each(body)
    }

    /// Run once for the current executor when it satisfies the query.
    #[doc(hidden)]
    pub fn current(self, body: impl FnOnce(Q::Item) -> Vec<String>) -> Vec<String> {
        Q::current(body)
    }
}

/// Adds query execution and typed-state filtering to the canonical [`Target`]
/// value.
///
/// This is an extension trait because target construction and command
/// rendering live in `sand-commands`, while `.each(...)` registers a generated
/// function in `sand-core`. The default prelude imports this trait, so authors
/// use the methods directly on `Target` without naming the trait.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::TargetExecution",
    aliases = ["sand::prelude::TargetExecution"],
    module = "sand::entity",
    summary = "Adds State filtering and iteration to the canonical Target value.",
    context = "This prelude-imported extension keeps target construction and query execution on one value while respecting the dependency boundary between command rendering and generated-function registration.",
    minecraft = "State filters become selector score constraints and iteration lowers to execute as/at plus one deduplicated generated function.",
    use_when = ["Iterating a Target", "Filtering a Target with typed State"],
    avoid_when = ["Representing the bound executor inside the callback; use EntityContext"],
    example = "use sand::prelude::*; let commands = Target::entities().each(|entity| vec![entity.add_tag(\"seen\")]);",
)]
pub trait TargetExecution: Sized {
    /// The execution-scoped entity kind passed to an iteration body.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::entity::TargetExecution::Kind", aliases = ["sand::prelude::TargetExecution::Kind"], module = "sand::entity", kind = "associated_type", summary = "The execution-scoped entity kind produced while iterating a target.", context = "Entity targets yield AnyEntity contexts and player-only targets yield PlayerKind contexts.", minecraft = "Models the kind of @s while the generated iteration function runs.", use_when = ["Writing generic code over target iteration"], avoid_when = ["Constructing a target"], example = "use sand::entity::TargetExecution;")]
    type Kind: crate::entity::kind::EntityKind;

    /// Restricts this target with a typed State predicate.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::entity::TargetExecution::state", aliases = ["sand::prelude::TargetExecution::state"], module = "sand::entity", summary = "Restricts a Target with a typed State predicate.", context = "Adds the State field's objective and score range to the existing canonical target rather than constructing a parallel query wrapper.", minecraft = "Lowers the predicate to a selector scores entry.", use_when = ["Selecting owners whose typed State satisfies a predicate"], avoid_when = ["Composing component presence; use StateQuery"], params(predicate = "The typed State predicate to apply."), returns = "The filtered target or an objective validation error.", example = "use sand::prelude::*; fn filter(target: impl TargetExecution) { let _ = target; }")]
    fn state(
        self,
        predicate: crate::entity::state::StatePredicate,
    ) -> sand_commands::CommandResult<Self>;

    /// Runs `body` with `@s` bound to each matching target.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::entity::TargetExecution::each", aliases = ["sand::prelude::TargetExecution::each"], module = "sand::entity", summary = "Runs a generated command body once for each matching target.", context = "Iteration is a capability of the canonical Target representation; the callback receives EntityContext rather than another target wrapper.", minecraft = "Lowers to execute as <target> at @s run function <generated>.", use_when = ["Applying commands to every entity or player selected by a Target"], avoid_when = ["Passing the target directly to one command is sufficient"], params(body = "The command-producing callback evaluated with a bound EntityContext."), returns = "The generated execute command, or an empty list for an empty body.", example = "use sand::prelude::*; let commands = Target::players().each(|player| vec![player.add_tag(\"ready\")]);")]
    fn each(self, body: impl FnOnce(&EntityContext<Self::Kind>) -> Vec<String>) -> Vec<String>;
}

impl<A> TargetExecution for Target<AnyTarget, A> {
    type Kind = AnyEntity;

    fn state(
        self,
        predicate: crate::entity::state::StatePredicate,
    ) -> sand_commands::CommandResult<Self> {
        self.score(
            sand_commands::ObjectiveName::try_dynamic(predicate.objective)?,
            predicate.selector_range,
        )
    }

    fn each(self, body: impl FnOnce(&EntityContext<Self::Kind>) -> Vec<String>) -> Vec<String> {
        lower_each(self.into_target_selector(), body)
    }
}

impl<A> TargetExecution for Target<PlayersOnly, A> {
    type Kind = PlayerKind;

    fn state(
        self,
        predicate: crate::entity::state::StatePredicate,
    ) -> sand_commands::CommandResult<Self> {
        self.score(
            sand_commands::ObjectiveName::try_dynamic(predicate.objective)?,
            predicate.selector_range,
        )
    }

    fn each(self, body: impl FnOnce(&EntityContext<Self::Kind>) -> Vec<String>) -> Vec<String> {
        lower_each(self.into_target_selector(), body)
    }
}

// ── Shared lowering ────────────────────────────────────────────────────────────

fn lower_each<K: crate::entity::kind::EntityKind>(
    selector: Selector,
    body: impl FnOnce(&EntityContext<K>) -> Vec<String>,
) -> Vec<String> {
    let inner = body(&EntityContext::new());
    if inner.is_empty() {
        return Vec::new();
    }
    let path = register_dyn_fn_dedup("sand/entity_query", inner);
    vec![format!(
        "execute as {selector} at @s run function __sand_local:{path}"
    )]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::state::{EntityFlag, EntityScore, EntityStateField};

    #[test]
    fn entities_each_lowers_to_execute_as_at_run_function() {
        let cmds = Target::entities()
            .entity_type("minecraft:zombie")
            .tag("hostile")
            .within_blocks(15.0)
            .nearest()
            .each(|entity| vec![entity.add_tag("observed")]);

        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with(
            "execute as @e[type=minecraft:zombie,tag=hostile,distance=..15,sort=nearest,limit=1] at @s run function __sand_local:sand/entity_query/"
        ));
    }

    #[test]
    fn empty_each_body_emits_no_commands() {
        let cmds = Target::entities().each(|_| Vec::new());
        assert!(cmds.is_empty());
    }

    #[test]
    fn structurally_identical_bodies_dedup_to_the_same_function() {
        let a = Target::entities().tag("a").each(|e| vec![e.add_tag("x")]);
        let b = Target::entities().tag("b").each(|e| vec![e.add_tag("x")]);
        let fn_a = a[0].rsplit("function ").next().unwrap();
        let fn_b = b[0].rsplit("function ").next().unwrap();
        assert_eq!(fn_a, fn_b);
    }

    #[test]
    fn players_each_lowers_with_player_selector() {
        let cmds = Target::players()
            .tag("ready")
            .nearest()
            .each(|p| vec![p.add_tag("chosen")]);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with(
            "execute as @a[tag=ready,sort=nearest,limit=1] at @s run function __sand_local:sand/entity_query/"
        ));
    }

    #[test]
    fn typed_state_predicates_merge_into_one_score_map() {
        let level = EntityScore::<i32>::new("rpg", "mob", "level", 1, None);
        let sick = EntityFlag::new("rpg", "mob", "sick", false);
        let commands = Target::entities()
            .entity_type("minecraft:zombie")
            .state(level.matches(10..=20).unwrap())
            .unwrap()
            .state(sick.is_enabled())
            .unwrap()
            .each(|entity| vec![entity.add_tag("matched")]);
        assert!(commands[0].contains("scores={"));
        assert!(commands[0].contains(&format!("{}=10..20", level.objective())));
        assert!(commands[0].contains(&format!("{}=1", sick.objective())));
        assert_eq!(commands[0].matches("scores={").count(), 1);
    }
}
