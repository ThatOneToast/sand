//! Resource-backed abilities composed from consumer-owned State.
//!
//! Constructing these values registers nothing. State owns persistence, bounds,
//! defaults and auto-ticking; callers own triggers and synchronous targeting.

use crate::command::{CommandError, CommandResult};
use crate::component::{IntoCommands, McFunction};
use crate::entity::{EntityCooldownAccessor, EntityScoreAccessor, EntityTimerAccessor};
use crate::prelude::*;
use crate::registration::{DatapackRegistration, IntoDatapack, LifecycleContribution};

/// A view of a consumer-owned integer State resource and its capacity expression.
///
/// Declare a zero lower bound and the capacity upper bound on the State schema.
/// This view does not inspect or override those bounds and does not install a
/// second storage model or clamp policy. Capacity should match the schema's upper
/// bound. Multiple abilities may copy the same view without scheduling maintenance.
#[derive(Clone)]
#[sand_macros::api(path = "sand::systems::ability::ResourcePool", module = "sand::systems::ability", summary = "A view of a consumer-owned integer State resource and its capacity expression.", context = "A view of a consumer-owned integer State resource and its capacity expression.  Declare a zero lower bound and the capacity upper bound on the State schema. This view does not inspect or override those bounds and does not install a second storage model or clamp policy. Capacity should match the schema's upper bound. Multiple abilities may copy the same view without scheduling maintenance.", minecraft = "Uses canonical State writes and grouped Conditions in the ordinary export pipeline.", use_when = ["Composing shared resources and accepted-target abilities"], avoid_when = ["Transactional rollback or asynchronous target ownership is required"], example = "use sand::systems::ability::{Ability, ResourcePool, ResourceMaintenance};")]
pub struct ResourcePool {
    value: EntityScoreAccessor,
    capacity: StatCurve,
}

impl ResourcePool {
    /// Bind a State score and the expression used to skip restoration at capacity.
    /// No resources or runtime work are registered by construction.
    #[sand_macros::api(path = "sand::systems::ability::ResourcePool::new", params(value = "The bound consumer State score, declared with zero lower bound and capacity upper bound.", capacity = "Capacity expression matching the State upper bound."), returns = "An inert resource view.", module = "sand::systems::ability", summary = "Bind a State score and the expression used to skip restoration at capacity.", context = "Bind a State score and the expression used to skip restoration at capacity. No resources or runtime work are registered by construction.", minecraft = "Uses canonical State writes and grouped Conditions in the ordinary export pipeline.", use_when = ["Composing shared resources and accepted-target abilities"], avoid_when = ["Transactional rollback or asynchronous target ownership is required"], example = "use sand::systems::ability::{Ability, ResourcePool, ResourceMaintenance};")]
    pub fn new(value: EntityScoreAccessor, capacity: impl Into<StatCurve>) -> Self {
        Self {
            value,
            capacity: capacity.into(),
        }
    }

    /// Restore a positive amount when the consumer's auto-ticking Timer elapses.
    /// The timer restarts even at capacity; State bounds saturate the addition.
    /// Register this action once per pool through ResourceMaintenance, independently
    /// of abilities. Cadence must be positive and fit a signed Minecraft score.
    #[sand_macros::api(path = "sand::systems::ability::ResourcePool::regenerate", params(timer = "The consumer-owned auto-ticking regeneration timer.", amount = "Positive restoration amount, saturated by State bounds.", cadence = "Positive interval in signed-score ticks."), returns = "Grouped restoration actions, or an invalid amount/duration error.", module = "sand::systems::ability", summary = "Restore a positive amount when the consumer's auto-ticking Timer elapses.", context = "Restore a positive amount when the consumer's auto-ticking Timer elapses. The timer restarts even at capacity; State bounds saturate the addition. Register this action once per pool through ResourceMaintenance, independently of abilities. Cadence must be positive and fit a signed Minecraft score.", minecraft = "Uses canonical State writes and grouped Conditions in the ordinary export pipeline.", use_when = ["Composing shared resources and accepted-target abilities"], avoid_when = ["Transactional rollback or asynchronous target ownership is required"], example = "use sand::systems::ability::{Ability, ResourcePool, ResourceMaintenance};")]
    pub fn regenerate(
        &self,
        timer: EntityTimerAccessor,
        amount: i32,
        cadence: Ticks,
    ) -> CommandResult<Actions> {
        duration(cadence)?;
        if amount <= 0 {
            return Err(CommandError::new(
                "resource pool",
                "amount",
                "regeneration must be positive",
            ));
        }
        Ok(when(timer.elapsed()).then_all(mcfunction![
            when(StatCurve::from(self.value).lt(self.capacity.clone())).then_one(self.value.add(amount));
            timer.start(cadence);
        ]))
    }
}

/// Eligibility and acceptance policy for one resource-backed activation.
///
/// Bind fields to the current player, supply immediate equipment/other Conditions,
/// and call attempt with synchronous target-search Actions. At a terminal target,
/// call accept with its Condition and effect Actions. All eligibility is sampled
/// together before writes. An accepted invocation spends once and starts cooldown
/// before effects; rejection leaves both unchanged. Early returns inside effect callbacks
/// can skip their remaining actions without undoing payment. A later native operation may
/// fail: Minecraft commands are not transactional and no rollback is promised.
///
/// Use a single terminal callback per attempt (for example BlockRaycast::on_hit).
/// Repeated accepts during an active cooldown reject. Callback context must retain
/// the same player; asynchronous targeting needs its own lifecycle protocol.
#[derive(Clone)]
#[sand_macros::api(path = "sand::systems::ability::Ability", module = "sand::systems::ability", summary = "Eligibility and acceptance policy for one resource-backed activation.", context = "Eligibility and acceptance policy for one resource-backed activation.  Bind fields to the current player, supply immediate equipment/other Conditions, and call attempt with synchronous target-search Actions. At a terminal target, call accept with its Condition and effect Actions. All eligibility is sampled together before writes. An accepted invocation spends once and starts cooldown before effects; rejection leaves both unchanged. Early returns inside effect callbacks can skip their remaining actions without undoing payment. A later native operation may fail: Minecraft commands are not transactional and no rollback is promised.  Use a single terminal callback per attempt (for example BlockRaycast::on_hit). Repeated accepts during an active cooldown reject. Callback context must retain the same player; asynchronous targeting needs its own lifecycle protocol.", minecraft = "Uses canonical State writes and grouped Conditions in the ordinary export pipeline.", use_when = ["Composing shared resources and accepted-target abilities"], avoid_when = ["Transactional rollback or asynchronous target ownership is required"], example = "use sand::systems::ability::{Ability, ResourcePool, ResourceMaintenance};")]
pub struct Ability {
    pool: ResourcePool,
    cooldown: EntityCooldownAccessor,
    cost: i32,
    duration: Ticks,
    eligibility: Condition,
}

impl Ability {
    /// Configure a cost, cooldown duration and canonical eligibility Condition.
    /// Compose ItemLocation::matches with other Conditions for immediate equipment
    /// checks. Nonnegative costs and positive signed-score durations are required.
    /// Construction registers no watcher, timer or resource.
    #[sand_macros::api(path = "sand::systems::ability::Ability::new", params(pool = "The shared resource view.", cooldown = "The bound consumer auto-ticking cooldown.", cost = "Nonnegative resource cost.", ticks = "Positive cooldown duration in signed-score ticks.", eligibility = "Immediate item-location and gameplay conditions."), returns = "An inert ability policy, or invalid cost/duration error.", module = "sand::systems::ability", summary = "Configure a cost, cooldown duration and canonical eligibility Condition.", context = "Configure a cost, cooldown duration and canonical eligibility Condition. Compose ItemLocation::matches with other Conditions for immediate equipment checks. Nonnegative costs and positive signed-score durations are required. Construction registers no watcher, timer or resource.", minecraft = "Uses canonical State writes and grouped Conditions in the ordinary export pipeline.", use_when = ["Composing shared resources and accepted-target abilities"], avoid_when = ["Transactional rollback or asynchronous target ownership is required"], example = "use sand::systems::ability::{Ability, ResourcePool, ResourceMaintenance};")]
    pub fn new(
        pool: ResourcePool,
        cooldown: EntityCooldownAccessor,
        cost: i32,
        ticks: Ticks,
        eligibility: Condition,
    ) -> CommandResult<Self> {
        duration(ticks)?;
        if cost < 0 {
            return Err(CommandError::new(
                "ability",
                "cost",
                "cost must be nonnegative",
            ));
        }
        Ok(Self {
            pool,
            cooldown,
            cost,
            duration: ticks,
            eligibility,
        })
    }

    /// Query the same resource, cooldown and caller Conditions used by activation.
    /// Useful for UI; evaluating this condition never mutates State.
    #[sand_macros::api(path = "sand::systems::ability::Ability::ready", returns = "The complete eligibility Condition, without writes.", module = "sand::systems::ability", summary = "Query the same resource, cooldown and caller Conditions used by activation.", context = "Query the same resource, cooldown and caller Conditions used by activation. Useful for UI; evaluating this condition never mutates State.", minecraft = "Uses canonical State writes and grouped Conditions in the ordinary export pipeline.", use_when = ["Composing shared resources and accepted-target abilities"], avoid_when = ["Transactional rollback or asynchronous target ownership is required"], example = "use sand::systems::ability::{Ability, ResourcePool, ResourceMaintenance};")]
    pub fn ready(&self) -> Condition {
        Condition::all([
            self.eligibility.clone(),
            self.cooldown.ready(),
            self.pool
                .value
                .matches(self.cost..)
                .expect("validated nonnegative cost"),
        ])
    }

    /// Run synchronous target-search actions only while eligible, without payment.
    /// Target rejection or a miss must not call accept with a true condition.
    #[sand_macros::api(path = "sand::systems::ability::Ability::attempt", params(target = "Synchronous target-search actions preserving the current player."), returns = "Grouped target-search actions guarded by eligibility.", module = "sand::systems::ability", summary = "Run synchronous target-search actions only while eligible, without payment.", context = "Run synchronous target-search actions only while eligible, without payment. Target rejection or a miss must not call accept with a true condition.", minecraft = "Uses canonical State writes and grouped Conditions in the ordinary export pipeline.", use_when = ["Composing shared resources and accepted-target abilities"], avoid_when = ["Transactional rollback or asynchronous target ownership is required"], example = "use sand::systems::ability::{Ability, ResourcePool, ResourceMaintenance};")]
    pub fn attempt(&self, target: impl IntoCommands) -> Actions {
        when(self.ready()).then_one(target)
    }

    /// Recheck eligibility and target acceptance together, then commit once.
    /// The grouped branch is evaluated before cost/cooldown writes, so those writes
    /// cannot suppress later effects. Effects may be Actions or typed function calls.
    /// A native failure after acceptance leaves the committed cost and cooldown.
    #[sand_macros::api(path = "sand::systems::ability::Ability::accept", params(target = "Acceptance condition for the current terminal target.", effects = "Effects issued after payment, including typed function calls."), returns = "One grouped acceptance branch with cost, cooldown and effects in order.", module = "sand::systems::ability", summary = "Recheck eligibility and target acceptance together, then commit once.", context = "Recheck eligibility and target acceptance together, then commit once. The grouped branch is evaluated before cost/cooldown writes, so those writes cannot suppress later effects. Effects may be Actions or typed function calls. A native failure after acceptance leaves the committed cost and cooldown.", minecraft = "Uses canonical State writes and grouped Conditions in the ordinary export pipeline.", use_when = ["Composing shared resources and accepted-target abilities"], avoid_when = ["Transactional rollback or asynchronous target ownership is required"], example = "use sand::systems::ability::{Ability, ResourcePool, ResourceMaintenance};")]
    pub fn accept(&self, target: Condition, effects: impl IntoCommands) -> Actions {
        when(self.ready().and(target)).then_all(mcfunction![
            self.pool.value.subtract(self.cost);
            self.cooldown.start(self.duration);
            effects;
        ])
    }
}

/// Explicit installation of one shared maintenance action in the tick lifecycle.
///
/// Register once per pool, not once per ability. The identity is a consumer-owned
/// function resource: repeated installation of that identity is an export collision
/// (even for identical definitions), rather than silently running maintenance twice.
/// Different identities are independent; callers must not register the same pool
/// under multiple identities. Constructing and discarding this value adds no work.
#[sand_macros::api(path = "sand::systems::ability::ResourceMaintenance", module = "sand::systems::ability", summary = "Explicit installation of one shared maintenance action in the tick lifecycle.", context = "Explicit installation of one shared maintenance action in the tick lifecycle.  Register once per pool, not once per ability. The identity is a consumer-owned function resource: repeated installation of that identity is an export collision (even for identical definitions), rather than silently running maintenance twice. Different identities are independent; callers must not register the same pool under multiple identities. Constructing and discarding this value adds no work.", minecraft = "Uses canonical State writes and grouped Conditions in the ordinary export pipeline.", use_when = ["Composing shared resources and accepted-target abilities"], avoid_when = ["Transactional rollback or asynchronous target ownership is required"], example = "use sand::systems::ability::{Ability, ResourcePool, ResourceMaintenance};")]
pub struct ResourceMaintenance {
    id: ResourceLocation,
    actions: Actions,
}

impl ResourceMaintenance {
    /// Give a maintenance action a unique function identity for ordinary registration.
    /// Use Target::players().each to bind consumer State inside each player's context.
    /// ResourcePool::regenerate validates the cadence before installation.
    #[sand_macros::api(path = "sand::systems::ability::ResourceMaintenance::new", params(id = "Unique consumer-owned maintenance function identity.", actions = "Maintenance actions, usually selected per player."), returns = "An inert definition to register once as a datapack component.", module = "sand::systems::ability", summary = "Give a maintenance action a unique function identity for ordinary registration.", context = "Give a maintenance action a unique function identity for ordinary registration. Use Target::players().each to bind consumer State inside each player's context. ResourcePool::regenerate validates the cadence before installation.", minecraft = "Uses canonical State writes and grouped Conditions in the ordinary export pipeline.", use_when = ["Composing shared resources and accepted-target abilities"], avoid_when = ["Transactional rollback or asynchronous target ownership is required"], example = "use sand::systems::ability::{Ability, ResourcePool, ResourceMaintenance};")]
    pub fn new(id: ResourceLocation, actions: Actions) -> Self {
        Self { id, actions }
    }
}

impl IntoDatapack for ResourceMaintenance {
    fn into_datapack(self) -> DatapackRegistration {
        DatapackRegistration::new()
            .lifecycle(LifecycleContribution::tick(cmd::function(
                FunctionId::custom(self.id.clone()),
            )))
            .component(McFunction::new(self.id).command(self.actions))
    }
}

fn duration(ticks: Ticks) -> CommandResult<()> {
    if ticks.get() == 0 || ticks.get() > i32::MAX as u32 {
        Err(CommandError::new(
            "ability resource",
            "duration",
            "duration must be in 1..=i32::MAX ticks",
        ))
    } else {
        Ok(())
    }
}
