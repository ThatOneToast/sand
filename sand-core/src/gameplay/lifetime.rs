//! Owned, bounded runtime work. Marker instances are implementation details;
//! author data remains ordinary State attached through StateComposition.
use super::{identity, runtime};
use crate::{
    IntoCommands, McFunction,
    entity::{EntityContext, EntityKind, MarkerKind, PlayerKind, StateComposition},
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
};
use sand_commands::{Anchor, CommandError, CommandResult, ObjectiveName};
use sand_components::{
    ComponentContent, DatapackComponent, DimensionId, ResourceLocation, effect::Ticks,
};
use std::{marker::PhantomData, sync::Arc};

pub(super) type Callback<K> = Arc<dyn Fn(TaskContext<K>) -> Actions>;

/// Execution context of one owned runtime instance. Its owner is resolved by a
/// persistent identity, never by proximity or a shared temporary selector.
/// Values describe generated actions; they are not host-side entity handles.
#[derive(Clone)]
#[sand_macros::api(
registry = sand_api_contract,
path = "sand::systems::TaskContext", module = "sand::systems",
summary = "Execution context of one owned runtime instance.",
context = "Execution context of one owned runtime instance. Its owner is resolved by a persistent identity, never by proximity or a shared temporary selector. Values describe generated actions; they are not host-side entity handles.",
minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
use_when = ["Composing reusable owner-bound gameplay"],
avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
example = "use sand::systems::TaskContext;",
)]
pub struct TaskContext<K: EntityKind = PlayerKind> {
    pub(super) scope: ResourceLocation,
    marker: PhantomData<K>,
}
impl<K: EntityKind> TaskContext<K> {
    pub(super) fn new(scope: ResourceLocation) -> Self {
        Self {
            scope,
            marker: PhantomData,
        }
    }
    /// Run actions as the available owner at its current position.
    /// Missing owners produce no actions. Nested traversal captures its own ID.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::TaskContext::owner", module = "sand::systems",
    kind = "method",
    summary = "Run actions as the available owner at its current position.",
    context = "Call from instance context to resolve the original owner. The body runs as that owner at its current position and does nothing if the owner is unavailable.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(body = "Closure that authors typed actions in the callback context described above."),
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{TaskContext, Lifetime};\n\nfn demonstrate(value: TaskContext) {\n    let _result = value.owner(|_| cmd::say(\"Owner callback\"));\n}",
    )]
    pub fn owner<R: IntoCommands>(&self, body: impl FnOnce(EntityContext<K>) -> R) -> Actions {
        Cmd::OwnedContext(Box::new(identity::OwnedContext {
            scope: self.scope.clone(),
            kind: K::LABEL,
            field: "owner",
            body: body(EntityContext::default()).into_commands(),
            child: None,
            damage: None,
        }))
        .into_commands()
    }
    /// Start an independent child for this exact parent instance, retaining the
    /// original owner. Call from instance context. Register the child definition
    /// normally; Lifetime, Job, Sequence, Scene, Projectile and Ability support
    /// this operation. Admission guards and capacity still apply. Status effects
    /// are deliberately excluded because refreshing can share an existing stack.
    ///
    /// Stopping a parent ends loaded descendants in its configured dimensions;
    /// other descendants invalidate on their next loaded tick. Ancestor ledgers
    /// preserve cancellation through unloaded intermediate parents. No chunks are
    /// loaded. Nesting is limited to 32 generations; deeper starts are no-ops.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::TaskContext::start_child", module = "sand::systems",
summary = "Start work owned by this exact parent instance.", context = "Uses the child definition's normal admission and original owner. Cancelling a sibling parent does not cancel this parent's children. Register child definitions normally.",
minecraft = "Captures ancestor identities before owner traversal and validates bounded ancestry ledgers on loaded ticks. Stopping parents invalidates descendants without force-loading chunks.",
use_when = ["An encounter must own and clean up its scenes, jobs or abilities"], avoid_when = ["The new work should outlive its parent or is a shared status effect"],
params(child = "A registered definition exposing its canonical Lifetime through AsRef."), returns = "Retained child-admission actions in parent-instance context.",
example = "use sand::systems::{TaskContext, Scene}; fn demo(ctx: TaskContext, scene: Scene) { let _ = ctx.start_child(&scene); }")]
    pub fn start_child(&self, child: impl AsRef<Lifetime<K>>) -> Actions {
        Cmd::OwnedContext(Box::new(identity::OwnedContext {
            scope: self.scope.clone(),
            kind: K::LABEL,
            field: "owner",
            body: Actions::default(),
            child: Some(child.as_ref().id.clone()),
            damage: None,
        }))
        .into_commands()
    }

    /// Test a condition on the available original owner. Missing owners are
    /// false. Use this from instance context, including Sequence wait predicates.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::TaskContext::owner_matches", module = "sand::systems",
summary = "Test a typed condition on the original owner.", context = "Preserves instance context while resolving the owner. Missing owners produce false; predicates may read canonical State or native properties.",
minecraft = "Uses an isolated function predicate and captured owner identity, without shared author-visible scratch state.",
use_when = ["Gating an owned sequence on owner State"], avoid_when = ["Actions rather than a boolean test are needed; use owner"],
params(condition = "Authors the condition on the bound owner."), returns = "A retained condition evaluated at runtime.",
example = "use sand::systems::TaskContext; use sand::prelude::Condition; fn demo(ctx: TaskContext) { let _ = ctx.owner_matches(|_| Condition::all([])); }")]
    pub fn owner_matches(
        &self,
        condition: impl FnOnce(EntityContext<K>) -> crate::condition::Condition,
    ) -> crate::condition::Condition {
        let actions = self.owner(|owner| {
            let mut result =
                condition(owner).guard_actions(false, &[], Cmd::Return(1).into_commands());
            result.extend([Cmd::Return(0)]);
            result
        });
        let body = Actions(vec![Cmd::ReturnRun(Box::new(
            actions.0.into_iter().next().expect("owner context action"),
        ))]);
        let path = crate::function::register_dyn_fn_dedup("sand/runtime/owner_matches", body);
        crate::condition::Condition::function_internal(format!("__sand_local:{path}"))
    }

    /// True when this live parent's direct children have all stopped, including
    /// completion, cancellation and expiry. No children is immediately true.
    /// Unloaded children remain outstanding until their cleanup runs. Combine
    /// with Sequence::wait_until and a timeout to join concurrent child work.
    /// This tests settlement, not whether every child completed successfully.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::TaskContext::children_finished", module = "sand::systems",
summary = "Test whether this instance has no outstanding direct children.", context = "Children that stop for any reason settle the join. Unloaded children remain outstanding. A missing or stopped parent is false; use a bounded wait for joins.",
minecraft = "Reads the parent's live ancestry-ledger child count. Admission increments before child callbacks; cleanup decrements once without reviving stopped parent records.",
use_when = ["Joining concurrently started scenes or jobs"], avoid_when = ["Distinguishing successful completion from cancellation is required"],
returns = "A runtime condition in parent-instance context.", example = "use sand::systems::TaskContext; fn demo(ctx: TaskContext) { let _ = ctx.children_finished(); }")]
    pub fn children_finished(&self) -> crate::condition::Condition {
        let path = crate::function::register_dyn_fn_dedup(
            "sand/runtime/children_finished",
            Cmd::ReturnRun(Box::new(Cmd::Function(
                identity::resource(&self.scope, "children_done").to_string(),
            )))
            .into_commands(),
        );
        crate::condition::Condition::function_internal(format!("__sand_local:{path}"))
    }

    /// Access the current marker through Sand's canonical entity context.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::TaskContext::instance", module = "sand::systems",
    kind = "method",
    summary = "Access the current marker through Sand's canonical entity context.",
    context = "Use inside a lifetime callback before traversing to owner or target context. The returned marker context can access State attached with Lifetime::owns.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "The current marker context for typed per-instance State access.",
    example = "use sand::prelude::*;\nuse sand::systems::{TaskContext, Lifetime};\n\nfn demonstrate(value: TaskContext) {\n    let _result = value.instance();\n}",
    )]
    pub fn instance(&self) -> EntityContext<MarkerKind> {
        EntityContext::default()
    }
    /// End this instance. Cleanup is idempotent, including inside callbacks.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::TaskContext::stop", module = "sand::systems",
    kind = "method",
    summary = "End this instance.",
    context = "Call from instance context to end only this instance. Reentrant stop calls are ignored and framework State detachment continues after the stop callback.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{TaskContext, Lifetime};\n\nfn demonstrate(value: TaskContext) {\n    let _result = value.stop();\n}",
    )]
    pub fn stop(&self) -> Actions {
        identity::call(&self.scope, "stop")
    }
}

/// A reusable definition of owner-bound work lasting a bounded number of loaded
/// ticks. Register once with `#[datapack_component]`, then use `start` and `cancel`.
///
/// Each instance owns its timer and attached State. Owner death, absence, reload,
/// explicit cancellation and expiry end it. Cleanup runs once before removal.
/// Unloaded instances neither tick nor force-load chunks; reload invalidates them
/// and they clean up on their next loaded tick. Non-player owners must remain in
/// the instance's dimension. Include custom dimensions explicitly with `dimensions`.
///
/// A definition admits at most 128 instances by default. Saturated definitions
/// reject starts before callbacks. IDs never wrap or reuse. Removing private
/// markers through external commands bypasses cleanup and can consume capacity
/// until reload. Stop callbacks cannot prevent framework cleanup with `return`.
#[derive(Clone)]
#[sand_macros::api(
registry = sand_api_contract,
path = "sand::systems::Lifetime", module = "sand::systems",
summary = "A reusable definition of owner-bound work lasting a bounded number of loaded ticks.",
context = "A reusable definition of owner-bound work lasting a bounded number of loaded ticks. Register once with `#[datapack_component]`, then use `start` and `cancel`.  Each instance owns its timer and attached State. Owner death, absence, reload, explicit cancellation and expiry end it. Cleanup runs once before removal. Unloaded instances neither tick nor force-load chunks; reload invalidates them and they clean up on their next loaded tick. Non-player owners must remain in the instance's dimension. Include custom dimensions explicitly with `dimensions`.  A definition admits at most 128 instances by default. Saturated definitions reject starts before callbacks. IDs never wrap or reuse. Removing private markers through external commands bypasses cleanup and can consume capacity until reload. Stop callbacks cannot prevent framework cleanup with `return`.",
minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
use_when = ["Composing reusable owner-bound gameplay"],
avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
example = "use sand::systems::Lifetime;",
)]
pub struct Lifetime<K: EntityKind = PlayerKind> {
    pub(super) id: ResourceLocation,
    pub(super) guard: Option<crate::condition::Condition>,
    pub(super) commit: Actions,
    pub(super) status: Option<super::status::StatusPolicy>,
    pub(super) invalid: Option<(&'static str, &'static str)>,
    required: Vec<Actions>,
    pub(super) diagnostics: Vec<(crate::condition::Condition, String)>,
    duration: Ticks,
    interval: Ticks,
    capacity: u32,
    origin: Anchor,
    dimensions: Vec<DimensionId>,
    pub(super) step: Option<&'static str>,
    pub(super) start: Option<Callback<K>>,
    tick: Option<Callback<K>>,
    pub(super) stop: Option<Callback<K>>,
    owned_schemas: Vec<String>,
    attach: Vec<fn(&'static str) -> Vec<String>>,
    detach: Vec<fn(&'static str) -> Vec<String>>,
}
impl<K: EntityKind> Lifetime<K> {
    pub(super) fn capacity_limit(&self) -> u32 {
        self.capacity
    }
    /// Create an inert definition with a required duration, feet origin and
    /// callbacks every tick. Register the same definition before calling it.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::new", module = "sand::systems",
    kind = "method",
    summary = "Create an inert definition with a required duration, feet origin and callbacks every tick.",
    context = "Creates an inert definition with a positive loaded-tick duration, feet origin and capacity 128. Register it as a datapack component before calling start.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(id = "Unique resource identity for the registered definition.", duration = "Positive duration in loaded Minecraft ticks."),
    returns = "The Lifetime definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Lifetime};\n\nfn demonstrate() {\n    let _result: Lifetime = Lifetime::new(\"demo:aura\".parse().unwrap(), Ticks::seconds(5));\n}",
    )]
    pub fn new(id: ResourceLocation, duration: Ticks) -> Self {
        Self {
            id,
            step: None,
            guard: None,
            commit: Actions::default(),
            status: None,
            invalid: None,
            required: vec![],
            diagnostics: vec![],
            duration,
            interval: Ticks::new(1),
            capacity: 128,
            origin: Anchor::Feet,
            dimensions: ["overworld", "the_nether", "the_end"]
                .into_iter()
                .map(|name| DimensionId::minecraft(name).expect("vanilla dimension"))
                .collect(),
            start: None,
            tick: None,
            stop: None,
            owned_schemas: vec![],
            attach: vec![],
            detach: vec![],
        }
    }
    /// Require a canonical State query to match the owner. Scoped State, bundles
    /// and composed `StateQuery` declarations share their normal membership rules.
    /// A missing required component, obsolete schema version or present forbidden
    /// component rejects admission and ends active work on its next loaded tick.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Lifetime::requires", module = "sand::systems", kind = "method",
        summary = "Binds owned work to canonical owner State membership.",
        context = "Scoped State, bundles and composed StateQuery requirements are checked at admission and before each loaded tick. A query mismatch triggers ordinary lifetime cleanup.",
        minecraft = "Uses existing composition presence objectives and schema versions; creates no parallel membership state.",
        use_when = ["Ending an ability when its owner leaves an archetype or match"],
        avoid_when = ["Work should survive removal of the specified State"],
        returns = "The Lifetime definition with this configuration; register the completed definition before invoking its runtime actions.",
        example = "use sand::prelude::*;\nuse sand::systems::Lifetime;\n\n#[derive(State)]\n#[state(namespace = \"demo\", scope = player)]\nstruct GameplayState { points: Score }\n\nfn demonstrate(value: Lifetime) {\n    let _configured = value.requires::<GameplayState>();\n}",
    )]
    pub fn requires<S: crate::entity::StateQueryOperations>(mut self) -> Self {
        self.required
            .push(<S as crate::entity::query::StateQuerySpec>::current(|_| {
                Cmd::Return(1)
            }));
        self
    }
    /// Set the interval between tick callbacks; expiry still counts every tick.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::every", module = "sand::systems",
    kind = "method",
    summary = "Set the interval between tick callbacks; expiry still counts every tick.",
    context = "Changes callback cadence without changing expiry cadence. The interval must be positive; duration still decreases on every loaded tick.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(interval = "Positive number of loaded ticks between callbacks; expiry still advances every tick."),
    returns = "The Lifetime definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Lifetime};\n\nfn demonstrate(value: Lifetime) {\n    let _result = value.every(Ticks::seconds(1));\n}",
    )]
    pub fn every(mut self, interval: Ticks) -> Self {
        self.interval = interval;
        self
    }
    /// Bound simultaneous instances, including unloaded instances, per definition.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::capacity", module = "sand::systems",
    kind = "method",
    summary = "Bound simultaneous instances, including unloaded instances, per definition.",
    context = "Bounds simultaneously admitted instances across all owners, including unloaded instances. A full definition rejects starts before user callbacks or ability costs.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(capacity = "Maximum admitted instances across all owners, including unloaded instances."),
    returns = "The Lifetime definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Lifetime};\n\nfn demonstrate(value: Lifetime) {\n    let _result = value.capacity(64);\n}",
    )]
    pub fn capacity(mut self, capacity: u32) -> Self {
        self.capacity = capacity;
        self
    }
    /// Choose the spawn anchor. Rotation always follows the owner.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::origin", module = "sand::systems",
    kind = "method",
    summary = "Choose the spawn anchor.",
    context = "Chooses the owner anchor used to create a marker. Eyes is useful for aimed projectiles; subsequent owner traversal uses the owner current position.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(origin = "Owner anchor used for the initial marker position and facing."),
    returns = "The Lifetime definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Lifetime};\n\nfn demonstrate(value: Lifetime) {\n    let _result = value.origin(sand::command::Anchor::Eyes);\n}",
    )]
    pub fn origin(mut self, origin: Anchor) -> Self {
        self.origin = origin;
        self
    }
    /// Replace the dimensions scanned for this definition. No chunks are loaded.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::dimensions", module = "sand::systems",
    kind = "method",
    summary = "Replace the dimensions scanned for this definition.",
    context = "Replaces the default three vanilla scan dimensions. Include every dimension where instances can exist; the list must be nonempty and does not force-load chunks.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(dimensions = "Nonempty set of dimensions whose loaded instances participate in ticking and cleanup."),
    returns = "The Lifetime definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Lifetime};\n\nfn demonstrate(value: Lifetime) {\n    let _result = value.dimensions([\"minecraft:overworld\".parse().unwrap()]);\n}",
    )]
    pub fn dimensions(mut self, dimensions: impl IntoIterator<Item = DimensionId>) -> Self {
        self.dimensions = dimensions.into_iter().collect();
        self
    }
    /// Attach canonical State to each marker and detach it during cleanup.
    /// Use entity-scoped State, as marker instances are not players or living mobs.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::owns", module = "sand::systems",
    kind = "method",
    summary = "Attach canonical State to each marker and detach it during cleanup.",
    context = "Attaches canonical entity-scoped State to each marker before its start callback. Cleanup detaches the same composition; player, living and global State are rejected.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "The Lifetime definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::Lifetime;\n\n#[derive(State)]\n#[state(namespace = \"demo\", scope = entity)]\nstruct GameplayState { points: Score }\n\nfn demonstrate(value: Lifetime) {\n    let _configured = value.owns::<GameplayState>();\n}",
    )]
    pub fn owns<S: StateComposition>(mut self) -> Self {
        self.owned_schemas.extend(
            S::composition_schemas()
                .into_iter()
                .map(|schema| schema.id()),
        );
        self.attach.push(S::composition_attach);
        self.detach.push(S::composition_detach);
        self
    }
    /// Run once after admission and State attachment, in instance context.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::on_start", module = "sand::systems",
    kind = "method",
    summary = "Run once after admission and State attachment, in instance context.",
    context = "Builds actions in instance context after successful admission and owned State attachment. Ability costs and cooldowns have already committed before this callback runs.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(body = "Closure that authors typed actions in the callback context described above."),
    returns = "The Lifetime definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Lifetime};\n\nfn demonstrate(value: Lifetime) {\n    let _result = value.on_start(|ctx| ctx.owner(|_| cmd::say(\"Started\")));\n}",
    )]
    pub fn on_start<R: IntoCommands>(
        mut self,
        body: impl Fn(TaskContext<K>) -> R + 'static,
    ) -> Self {
        self.start = Some(Arc::new(move |ctx| body(ctx).into_commands()));
        self
    }
    /// Run periodically in instance context, after owner validation.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::on_tick", module = "sand::systems",
    kind = "method",
    summary = "Run periodically in instance context, after owner validation.",
    context = "Builds actions in instance context at the configured every interval. The owner and required State are validated first; stopping here prevents further timer work.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(body = "Closure that authors typed actions in the callback context described above."),
    returns = "The Lifetime definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Lifetime};\n\nfn demonstrate(value: Lifetime) {\n    let _result = value.on_tick(|ctx| ctx.owner(|_| cmd::say(\"Ticked\")));\n}",
    )]
    pub fn on_tick<R: IntoCommands>(
        mut self,
        body: impl Fn(TaskContext<K>) -> R + 'static,
    ) -> Self {
        self.tick = Some(Arc::new(move |ctx| body(ctx).into_commands()));
        self
    }
    /// Run once during cleanup, before State detachment. Owner traversal may be
    /// empty after disconnect or death; instance cleanup does not depend on it.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::on_stop", module = "sand::systems",
    kind = "method",
    summary = "Run once during cleanup, before State detachment.",
    context = "Builds instance-context actions before State detachment and marker removal. Owner traversal may be empty after disconnect; an early return cannot bypass cleanup.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(body = "Closure that authors typed actions in the callback context described above."),
    returns = "The Lifetime definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Lifetime};\n\nfn demonstrate(value: Lifetime) {\n    let _result = value.on_stop(|ctx| ctx.owner(|_| cmd::say(\"Stopped\")));\n}",
    )]
    pub fn on_stop<R: IntoCommands>(
        mut self,
        body: impl Fn(TaskContext<K>) -> R + 'static,
    ) -> Self {
        self.stop = Some(Arc::new(move |ctx| body(ctx).into_commands()));
        self
    }
    /// Attempt to start an instance owned by the current entity. Invalid owner,
    /// exhausted IDs or capacity produce no instance and no start callback.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::start", module = "sand::systems",
    kind = "method",
    summary = "Attempt to start an instance owned by the current entity.",
    context = "Attempts to create work for the current entity executor. Required State, owner identity and global capacity are checked before attaching State or invoking callbacks.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{Lifetime};\n\nfn demonstrate(value: Lifetime) {\n    let _result = value.start();\n}",
    )]
    pub fn start(&self) -> Actions {
        identity::call(&self.id, "start")
    }
    /// Cancel this owner's loaded instances across the configured dimensions.
    /// Unloaded instances observe the owner cancellation generation when loaded again.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Lifetime::cancel", module = "sand::systems",
    kind = "method",
    summary = "Cancel this owner's loaded instances across the configured dimensions.",
    context = "Invalidates this definition for the current owner and cleans loaded instances immediately. Unloaded instances observe the new cancellation generation on return.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{Lifetime};\n\nfn demonstrate(value: Lifetime) {\n    let _result = value.cancel();\n}",
    )]
    pub fn cancel(&self) -> Actions {
        identity::call(&self.id, "cancel")
    }
}

impl<K: EntityKind> IntoDatapack for Lifetime<K> {
    fn into_datapack(self) -> DatapackRegistration {
        let mut registration = DatapackRegistration::new()
            .lifecycle(LifecycleContribution::load(format!(
                "function {}",
                identity::resource(&self.id, "load")
            )))
            .lifecycle(LifecycleContribution::tick(format!(
                "function {}",
                identity::resource(&self.id, "tick")
            )));
        let mut parts = vec![
            "load",
            "children_done",
            "children_check",
            "child_added",
            "child_removed",
            "start_child",
            "admit_child",
            "init_child",
            "retire",
            "cancel_descendants",
            "stop_descendant",
            "ancestors_valid",
            "ancestor_next",
            "ancestor_present",
            "tick",
            "start",
            "admit",
            "init",
            "step",
            "resume_tick",
            "after_tick",
            "resume_step",
            "finish_step",
            "resolve_stop",
            "stop_instance",
            "resolve_cleanup",
            "cleanup",
            "validate",
            "owner_valid",
            "diagnose",
            "stop",
            "cancel",
            "cancel_owner",
            "commit",
            "started",
            "ticked",
            "stopped",
        ];
        if self.status.is_some() {
            parts.extend(["status_admit", "status_refresh", "status_remove"]);
        }
        for part in parts {
            registration = registration.component(RuntimeFunction {
                location: identity::resource(&self.id, part),
                definition: self.clone(),
                part,
            });
        }
        registration
    }
}
struct RuntimeFunction<K: EntityKind> {
    location: ResourceLocation,
    definition: Lifetime<K>,
    part: &'static str,
}
impl<K: EntityKind> RuntimeFunction<K> {
    fn build(&self) -> CommandResult<McFunction> {
        let d = &self.definition;
        if let Some((field, message)) = d.invalid {
            return Err(CommandError::new("lifetime", field, message));
        }
        for (field, value) in [
            ("duration", d.duration.get()),
            ("interval", d.interval.get()),
            ("capacity", d.capacity),
        ] {
            if value == 0 || value > i32::MAX as u32 {
                return Err(CommandError::new(
                    "lifetime",
                    field,
                    "must be in 1..=2147483647",
                ));
            }
        }
        if d.dimensions.is_empty() {
            return Err(CommandError::new(
                "lifetime",
                "dimensions",
                "at least one dimension is required",
            ));
        }
        for id in &d.owned_schemas {
            let descriptor = crate::inventory::iter::<crate::state::StateDescriptor>
                .into_iter()
                .find(|descriptor| descriptor.id == id);
            if !descriptor.is_some_and(|descriptor| {
                matches!(descriptor.scope, crate::state::StateScope::Entity)
            }) {
                return Err(CommandError::new(
                    "lifetime",
                    "owned_state",
                    format!(
                        "marker instances require registered entity-scoped State; `{id}` is not entity-scoped"
                    ),
                ));
            }
        }
        let id = crate::function::resolve_export_location(&d.id)?;
        let owner = identity::objective(&d.id)?.to_string();
        let objectives = [
            "epoch",
            "remaining",
            "phase",
            "active",
            "generation",
            "visited",
        ]
        .map(|name| ObjectiveName::logical(format!("sand lifetime {id} {name}")));
        crate::function::request_numeric_objectives(objectives.clone())?;
        let [epoch, remaining, phase, active, generation, visited] =
            objectives.map(|o| o.to_string());
        let tag = identity::task_tag(&d.id)?;
        let resource = |part| identity::resource(&id, part).to_string();
        let storage = format!("{}:__sand_runtime", id.namespace());
        let task_ledger = resource("task_ledger");
        let context = || TaskContext {
            scope: d.id.clone(),
            marker: PhantomData,
        };
        let continuation = match self.part {
            "resume_tick" => Some("after_tick"),
            "resume_step" => Some("finish_step"),
            "resolve_stop" => Some("stop_instance"),
            "resolve_cleanup" => Some("cleanup"),
            _ => None,
        };
        if let Some(destination) = continuation {
            return Ok(McFunction::new(self.location.clone())
                .commands(runtime::resolve(&id, destination)?));
        }
        let callback = match self.part {
            "started" => d.start.as_ref(),
            "ticked" => d.tick.as_ref(),
            "stopped" => d.stop.as_ref(),
            _ => None,
        };
        if ["commit", "started", "ticked", "stopped"].contains(&self.part) {
            let mut actions = Actions::default();
            if self.part == "commit" {
                actions.extend([context().owner(|_| d.commit.clone())]);
            }
            actions.extend([callback.map(|f| f(context())).unwrap_or_default()]);
            return Ok(McFunction::new(self.location.clone()).commands(actions));
        }
        let required: Vec<_> = d
            .required
            .iter()
            .map(|query| {
                let mut body = query.clone();
                body.extend([Cmd::Return(0)]);
                let path = crate::function::register_dyn_fn_dedup("sand/runtime/required", body);
                crate::condition::Condition::function_internal(format!("__sand_local:{path}"))
            })
            .collect();
        if self.part == "diagnose" {
            let message =
                |text: String| Cmd::Raw(format!("tellraw @s {}", serde_json::json!({"text":text})));
            let mut actions =
                Cmd::Raw("execute unless entity @s[type=minecraft:player] run return 0".into())
                    .into_commands();
            actions.extend([message(format!("Activation checks for {id}"))]);
            for (condition, label) in &d.diagnostics {
                actions.extend([condition.guard_actions(
                    true,
                    &[],
                    message(label.clone()).into_commands(),
                )]);
            }
            for condition in &required {
                actions.extend([condition.guard_actions(
                    true,
                    &[],
                    message("Required owner State is detached or obsolete".into()).into_commands(),
                )]);
            }
            actions.extend([Cmd::Raw(format!(
                "execute if score #live {active} matches {}.. run tellraw @s {}",
                d.capacity,
                serde_json::json!({"text":"Lifetime capacity is exhausted"})
            ))]);
            actions.extend([Cmd::Raw(format!(
                "execute if score #next {owner} matches 2147483646.. run tellraw @s {}",
                serde_json::json!({"text":"Runtime identity space is exhausted"})
            ))]);
            actions.extend([Cmd::Raw(format!("tellraw @s {}", serde_json::json!([{"text":"Active instances: "},{"score":{"name":"#live","objective":active}},{"text":format!(" / {}",d.capacity)}]))) ]);
            return Ok(McFunction::new(self.location.clone()).commands(actions));
        }
        if ["start", "start_child", "owner_valid"].contains(&self.part) {
            let mut required = required;
            if self.part != "owner_valid" {
                required.extend(d.guard.clone());
            }
            let call = if self.part == "start_child" {
                if d.status.is_some() {
                    return Ok(McFunction::new(self.location.clone()).commands([Cmd::Return(0)]));
                }
                format!("$function {} {{ancestors:$(ancestors),parent:$(parent),parent_ledger:\"$(parent_ledger)\"}}", resource("admit_child")).into_commands()
            } else if self.part == "start" {
                identity::call(&d.id, "admit")
            } else {
                Cmd::Return(1).into_commands()
            };
            let mut actions =
                crate::condition::Condition::all(required).guard_actions(false, &[], call);
            if self.part == "owner_valid" {
                actions.extend([Cmd::Return(0)]);
            }
            return Ok(McFunction::new(self.location.clone()).commands(actions));
        }
        let mut dimensions = d
            .dimensions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        dimensions.sort();
        dimensions.dedup();
        let mut lines:Vec<String> = match self.part {
            "load"=> vec![
                format!("data remove storage {task_ledger} active"),
                format!("scoreboard players add #next {owner} 0"),
                format!("scoreboard players add #epoch {epoch} 0"),
                format!("execute if score #epoch {epoch} matches ..2147483646 run scoreboard players add #epoch {epoch} 1"),
                format!("scoreboard players set #live {active} 0"),
            ],
            "tick"=> {
                let mut lines=vec![format!("execute store result score #now {visited} run time query gametime")];
                lines.extend(dimensions.iter().map(|dim|format!("execute in {dim} as @e[type=minecraft:marker,tag={tag},distance=0..] at @s run function {}",resource("step"))));
                lines
            },
            "admit"|"admit_child"=>{
                let kind = match K::LABEL { "entity"=>"@s".into(), kind=>format!("@s[type=minecraft:{kind}]") };
                vec![format!("execute {} run return 0", dimensions.iter().map(|dim| format!("unless dimension {dim}")).collect::<Vec<_>>().join(" ")),format!("execute unless entity {kind} run return 0"),
                    "execute if entity @s[nbt={Health:0.0f}] run return 0".into(),
                    format!("execute unless score #epoch {epoch} matches 1..2147483646 run return 0"),
                    format!("execute unless score #live {active} matches ..{} run return 0",d.capacity-1),
                    format!("execute unless score #next {owner} matches ..2147483645 run return 0"),
                    format!("execute unless score @s {owner} matches 1.. run scoreboard players add #next {owner} 1"),
                    format!("execute unless score @s {owner} matches 1.. run scoreboard players operation @s {owner} = #next {owner}"),
                    format!("scoreboard players add @s {generation} 0"),
                    format!("execute unless score @s {generation} matches ..2147483646 run return 0"),
                    format!("execute store result storage {storage} spawn.generation int 1 run scoreboard players get @s {generation}"),
                    format!("execute store result storage {storage} spawn.owner int 1 run scoreboard players get @s {owner}"),
                    format!("scoreboard players add #next {owner} 1"),
                    format!("execute store result storage {storage} spawn.instance int 1 run scoreboard players get #next {owner}"),
                    format!("execute store result storage {storage} spawn.epoch int 1 run scoreboard players get #epoch {epoch}"),
                    format!("execute at @s rotated as @s anchored {} positioned ^ ^ ^ summon minecraft:marker run function {} with storage {storage} spawn", d.origin,resource("init")),
                    format!("data remove storage {storage} spawn")]
            },
            "init"=>{
                let mut lines=vec!["tag @s add sand.owned".into(),
                    "execute unless data entity @s data.ancestors run data modify entity @s data.ancestors set value []".into(),
                    format!("data modify entity @s data.stop set value \"{}\"", resource("stop")),
                    format!("execute unless data storage {task_ledger} active run data modify storage {task_ledger} active set value []"),
                    format!("$data modify storage {task_ledger} active append value {{instance:$(instance),owner:$(owner),generation:$(generation),children:0}}"),
                    format!("tag @s add {tag}"),"$data merge entity @s {data:{owner:$(owner),instance:$(instance),epoch:$(epoch),generation:$(generation)}}".into(),format!("$scoreboard players set @s {owner} $(instance)"),format!("$scoreboard players set @s {epoch} $(epoch)"),format!("scoreboard players set @s {remaining} {}",d.duration.get()),format!("scoreboard players set @s {phase} {}",d.interval.get()),format!("scoreboard players set @s {active} 1"),format!("scoreboard players add #live {active} 1"),"tp @s ~ ~ ~ ~ ~".into()];
                lines.push(format!("function {}",resource("commit")));
                for attach in &d.attach { lines.extend(attach("@s")); }
                lines.push(format!("execute unless entity @s[type=minecraft:marker,tag={tag}] run return 0"));
                lines.push(format!("function {}",resource("started"))); lines
            },
            "step"=> vec![
                format!("execute unless score @s {epoch} = #epoch {epoch} run return run function {}",resource("stop")),
                // Moving into a later-scanned dimension cannot spend another tick.
                format!("execute if score @s {visited} = #now {visited} run return 0"),
                format!("scoreboard players operation @s {visited} = #now {visited}"),
                format!("execute store result score @s {active} run function {} with entity @s data",resource("validate")),
                format!("execute unless score @s {active} matches 1 run return run function {}",resource("stop")),
                format!("scoreboard players remove @s {phase} 1"),
                format!("execute if score @s {phase} matches ..0 run function {}",resource("ticked")),
                format!("function {} with entity @s data",resource("resume_tick")),
            ],
            "after_tick"=> {
                let mut lines=Vec::new();
                if let Some(step)=d.step {
                    lines.push(format!("function {}",resource(step)));
                }
                lines.push(format!("function {} with entity @s data",resource("resume_step")));
                lines
            },
            "finish_step"=>vec![
                format!("execute if score @s {phase} matches ..0 run scoreboard players set @s {phase} {}",d.interval.get()),
                format!("scoreboard players remove @s {remaining} 1"),
                format!("execute if score @s {remaining} matches ..0 run function {}",resource("stop")),
            ],
            "validate"=>{
                let mut lines = vec![format!("execute if data entity @s data.ancestors[0] unless function {} run return 0", resource("ancestors_valid"))];
                lines.extend(identity::selectors(K::LABEL,&owner,"$(owner)").into_iter().map(|selector|format!("$execute as {selector} unless entity @s[nbt={{Health:0.0f}}] if score @s {generation} matches $(generation) run return run function {}",resource("owner_valid"))).collect::<Vec<_>>()); lines.push("return 0".into()); lines
            },
            "stop"=>vec![format!("function {} with entity @s data",resource("resolve_stop"))],
            "stop_instance"=>vec![
                format!("execute if entity @s[tag={tag}.stopping] run return 0"),
                format!("tag @s add {tag}.stopping"),
                format!("function {} with entity @s data", resource("retire")),
                format!("function {} with entity @s data", resource("cancel_descendants")),
                format!("execute if score @s {epoch} = #epoch {epoch} run scoreboard players remove #live {active} 1"),
                format!("function {}",resource("stopped")),
                format!("function {} with entity @s data",resource("resolve_cleanup")),
            ],
            "cleanup"=>{
                let mut lines=vec![format!("tag @s remove {tag}")];
                for detach in d.detach.iter().rev() { lines.extend(detach("@s")); }
                for objective in [&owner,&epoch,&remaining,&phase,&active,&visited] { lines.push(format!("scoreboard players reset @s {objective}")); }
                lines.push("kill @s".into());lines
            },
            "cancel"=>vec![format!("execute unless score @s {owner} matches 1.. run return 0"),format!("execute store result storage {storage} cancel.generation int 1 run scoreboard players get @s {generation}"),format!("execute if score @s {generation} matches ..2147483646 run scoreboard players add @s {generation} 1"),format!("execute store result storage {storage} cancel.owner int 1 run scoreboard players get @s {owner}"),format!("function {} with storage {storage} cancel",resource("cancel_owner")),format!("data remove storage {storage} cancel")],
            "cancel_owner"=> std::iter::once(format!("$data remove storage {task_ledger} active[{{owner:$(owner),generation:$(generation)}}]")).chain(dimensions.iter().map(|dim|format!("$execute in {dim} as @e[type=minecraft:marker,tag={tag},distance=0..,nbt={{data:{{owner:$(owner),generation:$(generation)}}}}] at @s run function {}",resource("stop")))).collect(),
            "init_child"=> vec![
                "$data modify entity @s data.ancestors set value $(ancestors)".into(),
                format!("function {} with entity @s data.ancestors[-1]",resource("child_added")),
                format!("$function {} {{owner:$(owner),instance:$(instance),epoch:$(epoch),generation:$(generation)}}", resource("init")),
            ],
            "retire"=>vec![
                format!("execute if data entity @s data.ancestors[-1] run function {} with entity @s data.ancestors[-1]",resource("child_removed")),
                format!("$data remove storage {task_ledger} active[{{instance:$(instance)}}]")],
            "children_done"=>vec![format!("return run function {} with entity @s data",resource("children_check"))],
            "children_check"=>vec![format!("$execute if data storage {task_ledger} active[{{instance:$(instance),children:0}}] run return 1"),"return 0".into()],
            "child_added"|"child_removed"=>vec![
                "$execute unless data storage $(ledger) active[{instance:$(instance)}] run return 0".into(),
                format!("$execute store result score #children {active} run data get storage $(ledger) active[{{instance:$(instance)}}].children"),
                format!("scoreboard players {} #children {active} 1", if self.part=="child_added" {"add"} else {"remove"}),
                format!("$execute store result storage $(ledger) active[{{instance:$(instance)}}].children int 1 run scoreboard players get #children {active}"),
            ],
            "cancel_descendants"=>dimensions.iter().map(|dim| format!("$execute in {dim} as @e[type=minecraft:marker,tag=sand.owned,distance=0..,nbt={{data:{{ancestors:[{{ledger:\"{task_ledger}\",instance:$(instance)}}]}}}}] at @s run function {} with entity @s data", resource("stop_descendant"))).collect(),
            "stop_descendant"=>vec!["$function $(stop)".into()],
            "ancestors_valid"=>vec![
                format!("data modify storage {storage} ancestry set from entity @s data.ancestors"),
                format!("execute store result score #ancestors {active} run function {}", resource("ancestor_next")),
                format!("data remove storage {storage} ancestry"),
                format!("return run scoreboard players get #ancestors {active}"),
            ],
            "ancestor_next"=>vec![
                format!("execute unless data storage {storage} ancestry[0] run return 1"),
                format!("execute store result score #ancestor {active} run function {} with storage {storage} ancestry[0]",resource("ancestor_present")),
                format!("execute unless score #ancestor {active} matches 1 run return 0"),
                format!("data remove storage {storage} ancestry[0]"),
                format!("return run function {}",resource("ancestor_next")),
            ],
            "ancestor_present"=>vec!["$execute if data storage $(ledger) active[{instance:$(instance)}] run return 1".into(), "return 0".into()],
            "status_admit"|"status_refresh"|"status_remove"=>vec![],
            _=>unreachable!()
        };
        if self.part == "admit_child" {
            let index = lines
                .iter()
                .position(|line| line.contains("summon minecraft:marker"))
                .expect("admission summons an instance");
            lines[index] = lines[index].replace(&resource("init"), &resource("init_child"));
            lines.splice(index..index, [
                format!("$data modify storage {storage} spawn.ancestors set value $(ancestors)"),
                format!("$data modify storage {storage} spawn.ancestors append value {{ledger:\"$(parent_ledger)\",instance:$(parent)}}"),
            ]);
        }
        if let Some(policy) = d.status {
            if policy.limit == 0 || policy.limit > i32::MAX as u32 {
                return Err(CommandError::new(
                    "status",
                    "stack_limit",
                    "must be in 1..=2147483647",
                ));
            }
            let ledger = identity::resource(&id, "ledger");
            let entry = "owners.o$(owner)";
            let refresh = ObjectiveName::logical(format!("sand lifetime {id} refresh"));
            crate::function::request_numeric_objectives([refresh.clone()])?;
            match self.part {
                "load" => lines.push(format!("data remove storage {ledger} owners")),
                "admit" => {
                    lines.retain(|line| !line.starts_with("execute unless score #live"));
                    let spawn = lines
                        .iter_mut()
                        .find(|line| line.contains("summon minecraft:marker"))
                        .expect("spawn action");
                    *spawn = format!(
                        "function {} with storage {storage} spawn",
                        resource("status_admit")
                    );
                }
                "status_admit" => {
                    lines = vec![
                        format!(
                            "$execute unless data storage {ledger} {entry}{{generation:$(generation)}} run data remove storage {ledger} {entry}"
                        ),
                        format!(
                            "$execute store result score @s {active} run data get storage {ledger} {entry}.count"
                        ),
                    ];
                    if policy.refresh {
                        lines.push(format!("execute if score @s {active} matches 1.. run return run function {} with storage {storage} spawn",resource("status_refresh")));
                    }
                    lines.extend([
                        format!("execute if score @s {active} matches {}.. run return 0",policy.limit),
                        format!("execute unless score #live {active} matches ..{} run return 0",d.capacity-1),
                        format!("$execute at @s rotated as @s anchored {} positioned ^ ^ ^ summon minecraft:marker run function {} {{owner:$(owner),instance:$(instance),epoch:$(epoch),generation:$(generation)}}", d.origin,resource("init")),
                    ]);
                }
                "status_refresh" => {
                    lines = vec![
                        format!(
                            "$execute store result score @s {active} run data get storage {ledger} {entry}.refresh"
                        ),
                        format!("execute if score @s {active} matches 2147483647 run return 0"),
                        format!("scoreboard players add @s {active} 1"),
                        format!(
                            "$execute store result storage {ledger} {entry}.refresh int 1 run scoreboard players get @s {active}"
                        ),
                    ]
                }
                "init" => {
                    let last = lines
                        .iter()
                        .position(|line| line.ends_with(&resource("commit")))
                        .expect("admission commit");
                    lines.splice(last..last, [
                        format!("$execute store result score @s {active} run data get storage {ledger} {entry}.count"),
                        format!("scoreboard players add @s {active} 1"),
                        format!("$execute store result storage {ledger} {entry}.count int 1 run scoreboard players get @s {active}"),
                        format!("$data modify storage {ledger} {entry}.generation set value $(generation)"),
                        format!("$execute store result score @s {refresh} run data get storage {ledger} {entry}.refresh"),
                    ]);
                }
                "step" | "finish_step" if policy.refresh => {
                    let proxy = crate::function::register_dyn_fn_dedup(
                        "sand/status/read_refresh",
                        vec![format!(
                            "$execute store result score @s {active} run data get storage {ledger} {entry}.refresh"
                        )],
                    );
                    let read = [
                        format!("function __sand_local:{proxy} with entity @s data"),
                        format!(
                            "execute unless score @s {refresh} = @s {active} run scoreboard players set @s {remaining} {}",
                            d.duration.get()
                        ),
                        format!("scoreboard players operation @s {refresh} = @s {active}"),
                    ];
                    let objective = if self.part == "step" {
                        &phase
                    } else {
                        &remaining
                    };
                    let insertion = lines
                        .iter()
                        .position(|line| {
                            line == &format!("scoreboard players remove @s {objective} 1")
                        })
                        .expect("lifetime timer");
                    lines.splice(insertion..insertion, read);
                }
                "stop_instance" => {
                    let index = lines
                        .iter()
                        .position(|line| line.ends_with(&resource("stopped")))
                        .expect("cleanup callback");
                    lines.insert(index, format!("execute if score @s {epoch} = #epoch {epoch} run function {} with entity @s data", resource("status_remove")));
                }
                "cleanup" => {
                    lines.insert(
                        lines.len() - 1,
                        format!("scoreboard players reset @s {refresh}"),
                    );
                }
                "status_remove" => {
                    lines = vec![
                        format!(
                            "$execute unless data storage {ledger} {entry}{{generation:$(generation)}} run return 0"
                        ),
                        format!(
                            "$execute store result score @s {active} run data get storage {ledger} {entry}.count"
                        ),
                        format!("scoreboard players remove @s {active} 1"),
                        format!(
                            "$execute store result storage {ledger} {entry}.count int 1 run scoreboard players get @s {active}"
                        ),
                        format!(
                            "$execute if score @s {active} matches ..0 run data remove storage {ledger} {entry}"
                        ),
                    ]
                }
                _ => {}
            }
        }
        Ok(McFunction::new(self.location.clone()).commands(lines))
    }
}
impl<K: EntityKind> DatapackComponent for RuntimeFunction<K> {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }
    fn component_dir(&self) -> &'static str {
        "function"
    }
    fn file_extension(&self) -> &'static str {
        "mcfunction"
    }
    fn to_json(&self) -> serde_json::Value {
        match self.try_content().expect("lifetime must validate") {
            ComponentContent::Text(s) => serde_json::Value::String(s),
            ComponentContent::Json(v) => v,
        }
    }
    fn try_content(&self) -> sand_components::error::Result<ComponentContent> {
        self.build()
            .map_err(|error| sand_components::SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "lifetime".into(),
                field: error.field.clone(),
                message: error.to_string(),
            })?
            .try_content()
    }
}

impl<K: EntityKind> AsRef<Lifetime<K>> for Lifetime<K> {
    fn as_ref(&self) -> &Lifetime<K> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_lifetimes_fail_before_runtime_resources_are_emitted() {
        for (definition, field) in [
            (
                Lifetime::<PlayerKind>::new("test:task".parse().unwrap(), Ticks::new(0)),
                "duration",
            ),
            (
                Lifetime::<PlayerKind>::new("test:task".parse().unwrap(), Ticks::new(1))
                    .every(Ticks::new(0)),
                "interval",
            ),
            (
                Lifetime::<PlayerKind>::new("test:task".parse().unwrap(), Ticks::new(1))
                    .capacity(0),
                "capacity",
            ),
            (
                Lifetime::<PlayerKind>::new("test:task".parse().unwrap(), Ticks::new(1))
                    .dimensions([]),
                "dimensions",
            ),
        ] {
            let _scope = crate::function::ExportFunctionRegistryScope::enter();
            let function = RuntimeFunction {
                location: identity::resource(&definition.id, "load"),
                definition,
                part: "load",
            };
            assert_eq!(
                function
                    .build()
                    .err()
                    .expect("invalid definition must fail")
                    .field,
                field
            );
            assert!(crate::function::take_numeric_objectives().is_empty());
        }
    }
}
