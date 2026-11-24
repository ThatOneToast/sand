//! Ordered objectives backed by one ordinary canonical State field.
use super::{identity, runtime};
use crate::{
    IntoCommands,
    condition::{Condition, ScoreRange},
    entity::{
        EntityContext, EntityKind, EntityScore, EntityScoreAccessor, EntityStateField, PlayerKind,
        StateFieldKind,
    },
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
    state::owned::PlayerScore,
};
use sand_commands::{CommandError, CommandResult, ObjectiveName};
use sand_components::ResourceLocation;
use std::sync::Arc;

type Objective<K> = (
    Arc<dyn Fn(EntityContext<K>) -> Condition>,
    Arc<dyn Fn(EntityContext<K>) -> Actions>,
);
type Branch<K> = (usize, usize, Arc<dyn Fn(EntityContext<K>) -> Condition>);

#[derive(Clone)]
enum ProgressField {
    Declared(EntityScore<i32>),
    Owned(PlayerScore),
}

enum BoundProgressField {
    Declared(EntityScoreAccessor<i32>),
    Owned(PlayerScore),
}

impl BoundProgressField {
    fn matches(&self, value: i32) -> CommandResult<Condition> {
        match self {
            Self::Declared(field) => field
                .matches(value..=value)
                .map_err(|error| CommandError::new("progression", "progress", error.to_string())),
            Self::Owned(field) => Ok(field.value_matches(ScoreRange::Eq(value))),
        }
    }

    fn set(&self, value: i32) -> Actions {
        match self {
            Self::Declared(field) => field.set(value),

            Self::Owned(field) => field.set(value).into_commands(),
        }
    }
}

struct BoundTrack {
    field: BoundProgressField,
    presence: ObjectiveName,
    revision: u32,
}

/// An ordered quest or phase track over an attached canonical integer State field.
///
/// Progress is the next objective's zero-based index; `len` means complete.
/// Call [`Self::advance`] from events or a registered system after updating the
/// objective's canonical State. One call consumes at most one objective, even if
/// later conditions are already true. Rewards execute as the owner after progress
/// commits. Repeated calls at completion do nothing. Rewards are ordinary actions:
/// item delivery may fail and no arbitrary-action rollback is promised. Compose
/// checked reward delivery separately when its success must gate progression.
///
/// Calls to advance/reset sharing the same field are rejected during a reward,
/// including synchronous callbacks and calls for other owners. Outside that brief
/// synchronous window owners are independent. Reset explicitly enables replay.
/// Do not write the progress field from reward callbacks or competing systems.
/// Player reconnect/reload follows the State schema, with no automatic reset.
/// Reordering objectives requires an explicit State migration for saved indices.
/// Conditional forward branches can skip objectives; skipped rewards do not run.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Progression", module = "sand::systems",
 summary = "Ordered objectives and committed rewards over canonical State.", context = "Ordered objectives use one attached canonical integer State field. Each advance checks only the current objective and commits its successor before isolated reward actions. Explicit reset permits replay.",
 minecraft = "Exports portable State guards and mutations; synchronous calls sharing the progress field are serialized through reward callbacks.",
 use_when = ["Authoring quests and round phases from canonical State and events"], avoid_when = ["Automatic task persistence or transactional arbitrary rewards"],

 example = "use sand::systems::Progression;")]
pub struct Progression<K: EntityKind = PlayerKind> {
    pub(super) id: ResourceLocation,
    progress: ProgressField,
    objectives: Vec<Objective<K>>,
    branches: Vec<Branch<K>>,
}
impl<K: EntityKind> Progression<K> {
    /// Define an ordered track. The field must be a registered, non-ticking Score
    /// with default zero and bounds covering every objective and completion.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Progression::new", module = "sand::systems",
 summary = "Define an ordered canonical State progression.", context = "Ordered objectives use one attached canonical integer State field. Each advance checks only the current objective and commits its successor before isolated reward actions. Explicit reset permits replay.",
 minecraft = "Exports portable State guards and mutations; synchronous calls sharing the progress field are serialized through reward callbacks.",
 use_when = ["Authoring quests and round phases from canonical State and events"], avoid_when = ["Automatic task persistence or transactional arbitrary rewards"],
 params(id = "Unique registered definition identity.", progress = "Attached owner-scoped integer Score storing the next objective index."),
 returns = "The configured progression or retained owner-context actions.", example = "use sand::systems::Progression;")]
    pub fn new(id: ResourceLocation, progress: EntityScore<i32>) -> Self {
        Self {
            id,
            progress: ProgressField::Declared(progress),
            objectives: vec![],
            branches: vec![],
        }
    }
    pub(crate) fn owned(id: ResourceLocation, progress: PlayerScore) -> Self {
        Self {
            id,
            progress: ProgressField::Owned(progress),
            objectives: vec![],
            branches: vec![],
        }
    }

    /// Append an owner-context condition and its once-per-reset reward actions.
    /// One to 256 objectives are supported. Definition order is persisted meaning.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Progression::objective", module = "sand::systems",
 summary = "Append one guarded objective and reward.", context = "Ordered objectives use one attached canonical integer State field. Each advance checks only the current objective and commits its successor before isolated reward actions. Explicit reset permits replay.",
 minecraft = "Exports portable State guards and mutations; synchronous calls sharing the progress field are serialized through reward callbacks.",
 use_when = ["Authoring quests and round phases from canonical State and events"], avoid_when = ["Automatic task persistence or transactional arbitrary rewards"],
 params(condition = "Owner-context completion predicate.", reward = "Owner-context actions after successor progress commits."),
 returns = "The configured progression or retained owner-context actions.", example = "use sand::systems::Progression;")]
    pub fn objective<R: IntoCommands>(
        mut self,
        condition: impl Fn(EntityContext<K>) -> Condition + 'static,
        reward: impl Fn(EntityContext<K>) -> R + 'static,
    ) -> Self {
        self.objectives.push((
            Arc::new(condition),
            Arc::new(move |owner| reward(owner).into_commands()),
        ));
        self
    }
    /// Choose a forward successor after the current objective becomes eligible.
    ///
    /// Indices are zero-based; `to == objective_count` completes the track.
    /// The first matching branch from this objective wins, otherwise progression
    /// advances to the next index. Conditions are sampled before progress writes
    /// and rewards. Only the current objective's reward runs; skipped objectives
    /// never receive rewards. Forward-only edges preserve once-per-reset rewards.
    /// Reordering objectives or changing branch meanings requires a State migration.
    /// At most 256 branches are supported; invalid indices fail export.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Progression::branch", module = "sand::systems",
    summary = "Choose a guarded forward successor in a canonical State quest.", context = "Branches are checked in declaration order only after the current objective is eligible. Forward edges cannot replay earlier rewards; unmatched branches retain sequential progression.",
    minecraft = "Samples conditions before the State index commits, then runs only the current objective reward under the shared progression lock.",
    use_when = ["Optional quest objectives and mutually exclusive forward quest paths"], avoid_when = ["Loops, arbitrary task persistence, or rewriting saved phase meanings without migration"],
    params(from = "Zero-based current objective index.", to = "Strictly later objective index, or objective count for completion.", condition = "Owner-context branch guard sampled before State writes and rewards."),
    returns = "The configured progression.", example = "use sand::systems::Progression;")]
    pub fn branch(
        mut self,
        from: usize,
        to: usize,
        condition: impl Fn(EntityContext<K>) -> Condition + 'static,
    ) -> Self {
        self.branches.push((from, to, Arc::new(condition)));
        self
    }
    /// Check and consume the current objective for the executing owner. Missing
    /// State, an invalid index, a false condition, completion or reentry is a no-op.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Progression::advance", module = "sand::systems",
 summary = "Consume at most one eligible objective.", context = "Ordered objectives use one attached canonical integer State field. Each advance checks only the current objective and commits its successor before isolated reward actions. Explicit reset permits replay.",
 minecraft = "Exports portable State guards and mutations; synchronous calls sharing the progress field are serialized through reward callbacks.",
 use_when = ["Authoring quests and round phases from canonical State and events"], avoid_when = ["Automatic task persistence or transactional arbitrary rewards"],

 returns = "The configured progression or retained owner-context actions.", example = "use sand::systems::Progression;")]
    pub fn advance(&self) -> Actions {
        identity::call(&self.id, "progression_advance")
    }
    /// Write zero without invoking rewards. Missing State or synchronous reentry
    /// is a no-op. This is an explicit replay policy, not automatic reload repair.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Progression::reset", module = "sand::systems",
 summary = "Explicitly reset a track to its first objective.", context = "Ordered objectives use one attached canonical integer State field. Each advance checks only the current objective and commits its successor before isolated reward actions. Explicit reset permits replay.",
 minecraft = "Exports portable State guards and mutations; synchronous calls sharing the progress field are serialized through reward callbacks.",
 use_when = ["Authoring quests and round phases from canonical State and events"], avoid_when = ["Automatic task persistence or transactional arbitrary rewards"],

 returns = "The configured progression or retained owner-context actions.", example = "use sand::systems::Progression;")]
    pub fn reset(&self) -> Actions {
        identity::call(&self.id, "progression_reset")
    }
    /// Test terminal progress in owner context, including current schema membership.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Progression::completed", module = "sand::systems",
 summary = "Test attached terminal progress.", context = "Ordered objectives use one attached canonical integer State field. Each advance checks only the current objective and commits its successor before isolated reward actions. Explicit reset permits replay.",
 minecraft = "Exports portable State guards and mutations; synchronous calls sharing the progress field are serialized through reward callbacks.",
 use_when = ["Authoring quests and round phases from canonical State and events"], avoid_when = ["Automatic task persistence or transactional arbitrary rewards"],

 returns = "An owner-context Condition requiring the current State schema and terminal index.", example = "use sand::systems::Progression;")]
    pub fn completed(&self) -> Condition {
        Condition::function_internal(
            identity::resource(&self.id, "progression_completed").to_string(),
        )
    }
    pub(super) fn attached(&self) -> Condition {
        Condition::function_internal(
            identity::resource(&self.id, "progression_attached").to_string(),
        )
    }
    pub(super) fn lock(&self) -> CommandResult<ObjectiveName> {
        let pack_namespace = identity::pack_namespace()?;
        let lock = ObjectiveName::logical(format!(
            "sand progression {pack_namespace} {} lock",
            self.progress_objective()
        ));
        crate::function::request_numeric_objectives([lock.clone()])?;
        Ok(lock)
    }
    fn progress_objective(&self) -> String {
        match &self.progress {
            ProgressField::Declared(field) => field.field_reference().objective,
            ProgressField::Owned(field) => field.objective.to_string(),
        }
    }

    fn schema(&self) -> CommandResult<&'static crate::state::StateDescriptor<'static>> {
        let error = |field, detail| CommandError::new("progression", field, detail);
        let ProgressField::Declared(progress) = &self.progress else {
            unreachable!("owned progression binds through portable State")
        };
        let reference = progress.field_reference();
        let schema = inventory::iter::<crate::state::StateDescriptor>
            .into_iter()
            .find(|s| s.id == reference.component)
            .ok_or_else(|| error("progress", "requires registered canonical State"))?;
        self.validate_schema(schema)
    }
    fn validate_track(&self) -> CommandResult<()> {
        let error = |field, detail| CommandError::new("progression", field, detail);
        if self.objectives.is_empty() || self.objectives.len() > 256 {
            return Err(error("objectives", "requires 1..=256 objectives"));
        }
        if self.branches.len() > 256
            || self.branches.iter().any(|(from, to, _)| {
                from >= to || *from >= self.objectives.len() || *to > self.objectives.len()
            })
        {
            return Err(error(
                "branches",
                "requires at most 256 forward branches with from < to <= objective count",
            ));
        }
        Ok(())
    }

    fn validate_schema<'a>(
        &self,
        schema: &'a crate::state::StateDescriptor<'static>,
    ) -> CommandResult<&'a crate::state::StateDescriptor<'static>> {
        let error = |field, detail| CommandError::new("progression", field, detail);
        self.validate_track()?;
        let ProgressField::Declared(progress) = &self.progress else {
            unreachable!("owned progression binds through portable State")
        };
        let reference = progress.field_reference();
        let field = schema
            .fields
            .iter()
            .find(|f| {
                ObjectiveName::logical(f.objective).as_str() == reference.objective
                    && f.field == reference.descriptor
            })
            .ok_or_else(|| error("progress", "field metadata does not match registered State"))?;
        if field.field.kind != StateFieldKind::Score || field.criterion != "dummy" {
            return Err(error(
                "progress",
                "requires a non-ticking dummy Score with default zero",
            ));
        }
        self.validate_progress_field(field.auto_tick, field.field.default, field.field.bounds)?;
        match schema.scope {
            crate::state::StateScope::Global(_) => {
                return Err(error(
                    "progress",
                    "requires owner-scoped State; global tracks cannot isolate owners",
                ));
            }
            crate::state::StateScope::Player if K::LABEL != "player" => {
                return Err(error(
                    "progress",
                    "player State requires a player progression",
                ));
            }
            crate::state::StateScope::Living if !matches!(K::LABEL, "player" | "zombie") => {
                return Err(error(
                    "progress",
                    "living State requires a known living owner kind",
                ));
            }
            _ => {}
        }
        Ok(schema)
    }
    fn validate_progress_field(
        &self,
        timer: bool,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> CommandResult<()> {
        if timer || default != 0 {
            return Err(CommandError::new(
                "progression",
                "progress",
                "requires a non-ticking dummy Score with default zero",
            ));
        }
        if bounds.is_some_and(|(min, max)| min > 0 || max < self.objectives.len() as i32) {
            return Err(CommandError::new(
                "progression",
                "progress",
                "bounds must include zero through objective count",
            ));
        }
        Ok(())
    }

    fn bind(&self) -> CommandResult<BoundTrack> {
        match &self.progress {
            ProgressField::Declared(field) => {
                let schema = self.schema()?;
                let dirty = matches!(
                    schema.scope,
                    crate::state::StateScope::Entity | crate::state::StateScope::Living
                );
                Ok(BoundTrack {
                    field: BoundProgressField::Declared(field.bind_to("@s", dirty)),
                    presence: ObjectiveName::logical(schema.presence_objective),
                    revision: schema.version,
                })
            }
            ProgressField::Owned(field) => {
                self.validate_track()?;
                if K::LABEL != "player" {
                    return Err(CommandError::new(
                        "progression",
                        "progress",
                        "requires player State",
                    ));
                }
                self.validate_progress_field(field.timer, field.default, field.bounds)?;
                Ok(BoundTrack {
                    field: BoundProgressField::Owned(field.clone()),
                    presence: field.presence.clone(),
                    revision: field.revision,
                })
            }
        }
    }

    pub(super) fn ready(&self) -> CommandResult<Actions> {
        Ok(Self::require_presence(&self.bind()?))
    }

    fn require_presence(track: &BoundTrack) -> Actions {
        let mut actions = Actions::default();
        if K::LABEL == "player" {
            actions.extend(["execute unless entity @s[type=minecraft:player] run return 0"]);
        } else if K::LABEL != "entity" {
            actions.extend([format!(
                "execute unless entity @s[type=minecraft:{}] run return 0",
                K::LABEL
            )]);
        } else {
            actions.extend(["execute unless entity @s run return 0"]);
        }
        actions.extend([format!(
            "execute unless score @s {} matches {} run return 0",
            track.presence, track.revision,
        )]);
        actions
    }

    pub(super) fn reset_write(&self) -> CommandResult<Actions> {
        Ok(self.bind()?.field.set(0))
    }

    fn build(&self, part: &str) -> CommandResult<Actions> {
        let track = self.bind()?;
        let progress = &track.field;
        let lock = self.lock()?;
        if part == "load" {
            return Ok(format!("scoreboard players set #busy {lock} 0").into_commands());
        }
        let mut body = Self::require_presence(&track);
        if part == "attached" {
            body.extend([Cmd::Return(1)]);
            return Ok(body);
        }
        if part == "completed" {
            body.extend([
                progress
                    .matches(self.objectives.len() as i32)?
                    .guard_actions(false, &[], Cmd::Return(1).into_commands()),
                Cmd::Return(0).into_commands(),
            ]);
            return Ok(body);
        }
        body.extend([format!(
            "execute if score #busy {lock} matches 1 run return 0"
        )]);
        if part == "reset" {
            body.extend([progress.set(0)]);
            return Ok(body);
        }
        for (index, (condition, reward)) in self.objectives.iter().enumerate() {
            let commit_to = |next: usize| {
                let mut commit = progress.set(next as i32);
                commit.extend([
                    runtime::isolated(reward(EntityContext::default())),
                    format!("scoreboard players set #busy {lock} 0").into_commands(),
                    Cmd::Return(1).into_commands(),
                ]);
                commit
            };
            let mut commit = format!("scoreboard players set #busy {lock} 1").into_commands();
            for (branch, (_, to, guard)) in self
                .branches
                .iter()
                .enumerate()
                .filter(|(_, (from, _, _))| *from == index)
            {
                commit.extend([guard(EntityContext::default()).guard_actions(
                    false,
                    &[],
                    Cmd::ReturnRun(Box::new(Cmd::AnonymousFunction {
                        prefix: format!("sand/progression/branch_{branch}"),
                        body: commit_to(*to),
                    }))
                    .into_commands(),
                )]);
            }
            commit.extend([commit_to(index + 1)]);
            let eligible = progress
                .matches(index as i32)?
                .and(condition(EntityContext::default()));
            body.extend([eligible.guard_actions(
                false,
                &[],
                Cmd::ReturnRun(Box::new(Cmd::AnonymousFunction {
                    prefix: format!("sand/progression/objective_{index}"),
                    body: commit,
                }))
                .into_commands(),
            )]);
        }
        body.extend([Cmd::Return(0)]);
        Ok(body)
    }
}
impl<K: EntityKind> IntoDatapack for Progression<K> {
    fn into_datapack(self) -> DatapackRegistration {
        let mut registration =
            DatapackRegistration::new().lifecycle(LifecycleContribution::load(format!(
                "function {}",
                identity::resource(&self.id, "progression_load")
            )));
        for part in ["load", "advance", "reset", "completed", "attached"] {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.id,
                &format!("progression_{part}"),
                move || definition.build(part),
            ));
        }
        registration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        entity::StateFieldDescriptor,
        state::{StateDescriptor, StateLifecycleDescriptor, StateScope},
    };

    const PHASE: StateFieldDescriptor =
        StateFieldDescriptor::new("phase", StateFieldKind::Score, 0, Some((0, 2)));
    const TOO_SMALL: StateFieldDescriptor =
        StateFieldDescriptor::new("small", StateFieldKind::Score, 0, Some((0, 1)));
    const NONZERO: StateFieldDescriptor =
        StateFieldDescriptor::new("nonzero", StateFieldKind::Score, 1, None);
    const TICKING: StateFieldDescriptor =
        StateFieldDescriptor::new("ticking", StateFieldKind::Score, 0, None);
    const FIELDS: &[StateLifecycleDescriptor] = &[
        StateLifecycleDescriptor::new("progression_test:entity.phase", PHASE),
        StateLifecycleDescriptor::new("progression_test:entity.small", TOO_SMALL),
        StateLifecycleDescriptor::new("progression_test:entity.nonzero", NONZERO),
        StateLifecycleDescriptor::new("progression_test:entity.ticking", TICKING).auto_tick(),
    ];
    const ENTITY: StateDescriptor = StateDescriptor::new(
        "progression_test:entity",
        1,
        StateScope::Entity,
        "progression_test_entity_present",
        "unused",
        FIELDS,
        &[],
        &[],
    );
    const GLOBAL: StateDescriptor = StateDescriptor::new(
        "progression_test:global",
        1,
        StateScope::Global("#global"),
        "progression_test_global_present",
        "unused",
        &[StateLifecycleDescriptor::new(
            "progression_test:global.phase",
            PHASE,
        )],
        &[],
        &[],
    );
    const PLAYER: StateDescriptor = StateDescriptor::new(
        "progression_test:player",
        1,
        StateScope::Player,
        "progression_test_player_present",
        "unused",
        &[StateLifecycleDescriptor::new(
            "progression_test:player.phase",
            PHASE,
        )],
        &[],
        &[],
    );

    fn track(field: EntityScore<i32>) -> Progression<crate::entity::AnyEntity> {
        Progression::new("progression_test:track".parse().unwrap(), field)
            .objective(|_| Condition::all([]), |_| Actions::default())
            .objective(|_| Condition::all([]), |_| Actions::default())
    }
    #[test]
    fn branches_reject_cycles_missing_objectives_and_unbounded_work() {
        let field = EntityScore::new("progression_test", "entity", "phase", 0, Some((0, 2)));
        for (from, to) in [(0, 0), (1, 0), (2, 3), (0, 3)] {
            assert_eq!(
                track(field)
                    .branch(from, to, |_| Condition::all([]))
                    .validate_schema(&ENTITY)
                    .unwrap_err()
                    .field,
                "branches"
            );
        }
        assert!(
            track(field)
                .branch(0, 2, |_| Condition::all([]))
                .validate_schema(&ENTITY)
                .is_ok()
        );
        let mut many = track(field);
        for _ in 0..257 {
            many = many.branch(0, 2, |_| Condition::all([]));
        }
        assert_eq!(many.validate_schema(&ENTITY).unwrap_err().field, "branches");
    }
    #[test]
    fn validates_canonical_membership_bounds_and_owner_scope() {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        let valid = EntityScore::new("progression_test", "entity", "phase", 0, Some((0, 2)));
        assert!(track(valid).validate_schema(&ENTITY).is_ok());
        for (field, message) in [
            (
                EntityScore::new("progression_test", "entity", "small", 0, Some((0, 1))),
                "bounds",
            ),
            (
                EntityScore::new("progression_test", "entity", "nonzero", 1, None),
                "default zero",
            ),
            (
                EntityScore::new("progression_test", "entity", "ticking", 0, None),
                "non-ticking",
            ),
            (
                EntityScore::new("progression_test", "global", "phase", 0, Some((0, 2))),
                "owner-scoped",
            ),
            (
                EntityScore::new("progression_test", "player", "phase", 0, Some((0, 2))),
                "player progression",
            ),
            (
                EntityScore::new("progression_test", "entity", "phase", 0, None),
                "metadata",
            ),
        ] {
            assert!(
                track(field)
                    .validate_schema(match field.component_id().as_str() {
                        "progression_test:global" => &GLOBAL,
                        "progression_test:player" => &PLAYER,
                        _ => &ENTITY,
                    })
                    .unwrap_err()
                    .to_string()
                    .contains(message),
                "{message}"
            );
        }
        assert!(
            track(EntityScore::new(
                "progression_test",
                "missing",
                "phase",
                0,
                None
            ))
            .schema()
            .unwrap_err()
            .to_string()
            .contains("registered")
        );
        let empty =
            Progression::<PlayerKind>::new("progression_test:empty".parse().unwrap(), valid);
        assert_eq!(
            empty.validate_schema(&ENTITY).unwrap_err().field,
            "objectives"
        );
        let mut oversized = track(valid);
        for _ in 0..255 {
            oversized = oversized.objective(|_| Condition::all([]), |_| Actions::default());
        }
        assert_eq!(
            oversized.validate_schema(&ENTITY).unwrap_err().field,
            "objectives"
        );
    }
}
