//! Ordered objectives backed by one ordinary canonical State field.
use super::{identity, runtime};
use crate::{
    IntoCommands,
    condition::Condition,
    entity::{
        EntityContext, EntityKind, EntityScore, EntityStateField, PlayerKind, StateFieldKind,
    },
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
};
use sand_commands::{CommandError, CommandResult, ObjectiveName};
use sand_components::ResourceLocation;
use std::sync::Arc;

type Objective<K> = (
    Arc<dyn Fn(EntityContext<K>) -> Condition>,
    Arc<dyn Fn(EntityContext<K>) -> Actions>,
);

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
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Progression", module = "sand::systems",
 summary = "Ordered objectives and committed rewards over canonical State.", context = "Ordered objectives use one attached canonical integer State field. Each advance checks only the current objective and commits its successor before isolated reward actions. Explicit reset permits replay.",
 minecraft = "Exports portable State guards and mutations; synchronous calls sharing the progress field are serialized through reward callbacks.",
 use_when = ["Authoring quests and round phases from canonical State and events"], avoid_when = ["Automatic task persistence or transactional arbitrary rewards"],

 example = "use sand::systems::Progression;")]
pub struct Progression<K: EntityKind = PlayerKind> {
    pub(super) id: ResourceLocation,
    progress: EntityScore<i32>,
    objectives: Vec<Objective<K>>,
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
            progress,
            objectives: vec![],
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
            self.progress.field_reference().objective
        ));
        crate::function::request_numeric_objectives([lock.clone()])?;
        Ok(lock)
    }
    fn schema(&self) -> CommandResult<&'static crate::state::StateDescriptor<'static>> {
        let error = |field, detail| CommandError::new("progression", field, detail);
        let reference = self.progress.field_reference();
        let schema = inventory::iter::<crate::state::StateDescriptor>
            .into_iter()
            .find(|s| s.id == reference.component)
            .ok_or_else(|| error("progress", "requires registered canonical State"))?;
        self.validate_schema(schema)
    }
    fn validate_schema<'a>(
        &self,
        schema: &'a crate::state::StateDescriptor<'static>,
    ) -> CommandResult<&'a crate::state::StateDescriptor<'static>> {
        let error = |field, detail| CommandError::new("progression", field, detail);
        if self.objectives.is_empty() || self.objectives.len() > 256 {
            return Err(error("objectives", "requires 1..=256 objectives"));
        }
        let reference = self.progress.field_reference();
        let field = schema
            .fields
            .iter()
            .find(|f| {
                ObjectiveName::logical(f.objective).as_str() == reference.objective
                    && f.field == reference.descriptor
            })
            .ok_or_else(|| error("progress", "field metadata does not match registered State"))?;
        if field.field.kind != StateFieldKind::Score
            || field.auto_tick
            || field.criterion != "dummy"
            || field.field.default != 0
        {
            return Err(error(
                "progress",
                "requires a non-ticking dummy Score with default zero",
            ));
        }
        if field
            .field
            .bounds
            .is_some_and(|(min, max)| min > 0 || max < self.objectives.len() as i32)
        {
            return Err(error(
                "progress",
                "bounds must include zero through objective count",
            ));
        }
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
    pub(super) fn ready(&self) -> CommandResult<Actions> {
        let schema = self.schema()?;
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
            ObjectiveName::logical(schema.presence_objective),
            schema.version
        )]);
        Ok(actions)
    }
    pub(super) fn reset_write(&self) -> CommandResult<Actions> {
        let schema = self.schema()?;
        Ok(self
            .progress
            .bind_to(
                "@s",
                matches!(
                    schema.scope,
                    crate::state::StateScope::Entity | crate::state::StateScope::Living
                ),
            )
            .set(0))
    }
    fn build(&self, part: &str) -> CommandResult<Actions> {
        let schema = self.schema()?;
        let dirty = matches!(
            schema.scope,
            crate::state::StateScope::Entity | crate::state::StateScope::Living
        );
        let progress = self.progress.bind_to("@s", dirty);
        let lock = self.lock()?;
        if part == "load" {
            return Ok(format!("scoreboard players set #busy {lock} 0").into_commands());
        }
        let mut body = self.ready()?;
        if part == "attached" {
            body.extend([Cmd::Return(1)]);
            return Ok(body);
        }
        if part == "completed" {
            body.extend([
                progress
                    .matches(self.objectives.len() as i32..=self.objectives.len() as i32)
                    .map_err(|e| CommandError::new("progression", "progress", e.to_string()))?
                    .guard_actions(false, &[], Cmd::Return(1).into_commands()),
                Cmd::Return(0).into_commands(),
            ]);
            return Ok(body);
        }
        body.extend([format!(
            "execute if score #busy {lock} matches 1 run return 0"
        )]);
        if part == "reset" {
            body.extend([self.reset_write()?]);
            return Ok(body);
        }
        for (index, (condition, reward)) in self.objectives.iter().enumerate() {
            let mut commit = format!("scoreboard players set #busy {lock} 1").into_commands();
            commit.extend([
                progress.set(index as i32 + 1),
                runtime::isolated(reward(EntityContext::default())),
                format!("scoreboard players set #busy {lock} 0").into_commands(),
                Cmd::Return(1).into_commands(),
            ]);
            let eligible = progress
                .matches(index as i32..=index as i32)
                .map_err(|e| CommandError::new("progression", "progress", e.to_string()))?
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
