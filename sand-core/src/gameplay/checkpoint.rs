//! Explicit checkpoint projections entirely within canonical State.
use super::{identity, runtime};
use crate::{
    IntoCommands,
    entity::state::StateFieldReference,
    entity::{
        EntityContext, EntityFlag, EntityScore, EntityStateField, PlayerKind, StateFieldKind,
    },
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
    state::{StateDescriptor, StateScope},
};
use sand_commands::{CommandError, CommandResult, ObjectiveName};
use sand_components::ResourceLocation;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

type PlayerAction = Arc<dyn Fn(EntityContext<PlayerKind>) -> Actions>;

/// An explicit integer-only save/restore policy using two canonical State schemas.
///
/// Each pair maps one live integer Score to a saved integer Score. All saved
/// fields and the initially-disabled validity Flag belong to one separately
/// derived State component. Its version and migration hooks are the saved format;
/// changing a pair's meaning requires an author migration (or invalidation).
/// Current live/saved schema membership, every field's presence and representable
/// bounds are checked before the first field write. Invalid input rejects the
/// operation without a partial field copy. Global, fixed-point, criterion-driven
/// and ticking fields are deliberately excluded. One to 64 pairs are supported.
///
/// Save commits validity last. Restore requires validity, runs optional preparation
/// such as Session cancellation, resolves the original online player by UUID, then
/// revalidates and restores the current saved values. Preparation may have its own
/// effects even when revalidation rejects. Post-restore actions may explicitly
/// start fresh work; private task state, positions, inventories and entities are
/// never saved, recreated or resumed automatically. State retains its normal
/// reload/reconnect persistence and migration behavior.
///
/// A pack checkpoint lock covers both callbacks and rejects synchronous save,
/// restore and clear reentry. Callback returns cannot skip lock/scratch cleanup.
/// Do not mutate saved fields from other synchronous native listeners. No general
/// callback rollback, cross-server save format or automatic checkpoint schedule
/// is implied. Detach the saved State through its normal lifecycle to reclaim it.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Checkpoint", module = "sand::systems",
 summary = "Explicit canonical State checkpoint save and restore.", context = "Explicit integer field pairs snapshot canonical player-owned State into one canonical saved State schema. Current schema membership and all value bounds are checked before writes; restore revalidates after preparation and resolves the original online player by UUID.",
 minecraft = "Copies selected canonical integer scores with normal dirty tracking. Saved State schema migrations define snapshot migration; ephemeral tasks are never copied or automatically restored.",
 use_when = ["Saving explicit gameplay checkpoints and preparing fresh session restoration"], avoid_when = ["Arbitrary world, inventory, entity, or coroutine persistence"],

 example = "use sand::systems::Checkpoint;")]
pub struct Checkpoint {
    id: ResourceLocation,
    valid: EntityFlag,
    fields: Vec<(EntityScore<i32>, EntityScore<i32>)>,
    before: Option<PlayerAction>,
    after: Option<PlayerAction>,
}
impl Checkpoint {
    /// Define a snapshot whose validity belongs to one saved State component.
    /// The flag must default to disabled. Attach all referenced schemas normally.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Checkpoint::new", module = "sand::systems",
 summary = "Define an explicit State checkpoint.", context = "Explicit integer field pairs snapshot canonical player-owned State into one canonical saved State schema. Current schema membership and all value bounds are checked before writes; restore revalidates after preparation and resolves the original online player by UUID.",
 minecraft = "Copies selected canonical integer scores with normal dirty tracking. Saved State schema migrations define snapshot migration; ephemeral tasks are never copied or automatically restored.",
 use_when = ["Saving explicit gameplay checkpoints and preparing fresh session restoration"], avoid_when = ["Arbitrary world, inventory, entity, or coroutine persistence"],
 params(id = "Unique registered definition identity.", valid = "Initially-disabled canonical Flag in the saved State schema."),
 returns = "The configured checkpoint or retained player-context actions.", example = "use sand::systems::Checkpoint;")]
    pub fn new(id: ResourceLocation, valid: EntityFlag) -> Self {
        Self {
            id,
            valid,
            fields: vec![],
            before: None,
            after: None,
        }
    }
    /// Append a live/saved integer field pair. Saved fields share the validity
    /// component; live components must differ from the saved component. Sources
    /// and destinations are unique and disjoint; both must be ordinary Scores.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Checkpoint::field", module = "sand::systems",
 summary = "Append an explicit integer snapshot field pair.", context = "Explicit integer field pairs snapshot canonical player-owned State into one canonical saved State schema. Current schema membership and all value bounds are checked before writes; restore revalidates after preparation and resolves the original online player by UUID.",
 minecraft = "Copies selected canonical integer scores with normal dirty tracking. Saved State schema migrations define snapshot migration; ephemeral tasks are never copied or automatically restored.",
 use_when = ["Saving explicit gameplay checkpoints and preparing fresh session restoration"], avoid_when = ["Arbitrary world, inventory, entity, or coroutine persistence"],
 params(live = "Canonical live integer Score.", saved = "Canonical saved integer Score in the validity schema."),
 returns = "The configured checkpoint or retained player-context actions.", example = "use sand::systems::Checkpoint;")]
    pub fn field(mut self, live: EntityScore<i32>, saved: EntityScore<i32>) -> Self {
        self.fields.push((live, saved));
        self
    }
    /// Prepare ephemeral work after initial validation and before the restore.
    /// Cancel sessions/tasks here. Fields and schema membership are revalidated
    /// afterward; preparation effects are not rolled back if restoration rejects.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Checkpoint::before_restore", module = "sand::systems",
 summary = "Prepare owned work before a revalidated restore.", context = "Explicit integer field pairs snapshot canonical player-owned State into one canonical saved State schema. Current schema membership and all value bounds are checked before writes; restore revalidates after preparation and resolves the original online player by UUID.",
 minecraft = "Copies selected canonical integer scores with normal dirty tracking. Saved State schema migrations define snapshot migration; ephemeral tasks are never copied or automatically restored.",
 use_when = ["Saving explicit gameplay checkpoints and preparing fresh session restoration"], avoid_when = ["Arbitrary world, inventory, entity, or coroutine persistence"],
 params(action = "Player-context cancellation or other explicit preparation."),
 returns = "The configured checkpoint or retained player-context actions.", example = "use sand::systems::Checkpoint;")]
    pub fn before_restore<R: IntoCommands>(
        mut self,
        action: impl Fn(EntityContext<PlayerKind>) -> R + 'static,
    ) -> Self {
        self.before = Some(Arc::new(move |owner| action(owner).into_commands()));
        self
    }
    /// Continue after all live fields commit. Explicitly starting fresh sessions
    /// is supported; the checkpoint never restores a private task continuation.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Checkpoint::after_restore", module = "sand::systems",
 summary = "Continue after a committed restore.", context = "Explicit integer field pairs snapshot canonical player-owned State into one canonical saved State schema. Current schema membership and all value bounds are checked before writes; restore revalidates after preparation and resolves the original online player by UUID.",
 minecraft = "Copies selected canonical integer scores with normal dirty tracking. Saved State schema migrations define snapshot migration; ephemeral tasks are never copied or automatically restored.",
 use_when = ["Saving explicit gameplay checkpoints and preparing fresh session restoration"], avoid_when = ["Arbitrary world, inventory, entity, or coroutine persistence"],
 params(action = "Player-context actions after every live field has committed."),
 returns = "The configured checkpoint or retained player-context actions.", example = "use sand::systems::Checkpoint;")]
    pub fn after_restore<R: IntoCommands>(
        mut self,
        action: impl Fn(EntityContext<PlayerKind>) -> R + 'static,
    ) -> Self {
        self.after = Some(Arc::new(move |owner| action(owner).into_commands()));
        self
    }
    /// Copy all selected live scores into saved State and enable validity last.
    /// Invalid membership/value bounds reject before modifying saved fields.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Checkpoint::save", module = "sand::systems",
 summary = "Save selected canonical gameplay values.", context = "Explicit integer field pairs snapshot canonical player-owned State into one canonical saved State schema. Current schema membership and all value bounds are checked before writes; restore revalidates after preparation and resolves the original online player by UUID.",
 minecraft = "Copies selected canonical integer scores with normal dirty tracking. Saved State schema migrations define snapshot migration; ephemeral tasks are never copied or automatically restored.",
 use_when = ["Saving explicit gameplay checkpoints and preparing fresh session restoration"], avoid_when = ["Arbitrary world, inventory, entity, or coroutine persistence"],

 returns = "The configured checkpoint or retained player-context actions.", example = "use sand::systems::Checkpoint;")]
    pub fn save(&self) -> Actions {
        Self::invoke(&self.id, crate::compiler::program::CheckpointCommand::Save)
    }
    /// Restore the valid saved projection after preparation and revalidation.
    /// Missing/offline original players cause no live writes after preparation.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Checkpoint::restore", module = "sand::systems",
 summary = "Restore selected gameplay values with explicit task policy.", context = "Explicit integer field pairs snapshot canonical player-owned State into one canonical saved State schema. Current schema membership and all value bounds are checked before writes; restore revalidates after preparation and resolves the original online player by UUID.",
 minecraft = "Copies selected canonical integer scores with normal dirty tracking. Saved State schema migrations define snapshot migration; ephemeral tasks are never copied or automatically restored.",
 use_when = ["Saving explicit gameplay checkpoints and preparing fresh session restoration"], avoid_when = ["Arbitrary world, inventory, entity, or coroutine persistence"],

 returns = "The configured checkpoint or retained player-context actions.", example = "use sand::systems::Checkpoint;")]
    pub fn restore(&self) -> Actions {
        Self::invoke(
            &self.id,
            crate::compiler::program::CheckpointCommand::Restore,
        )
    }
    /// Disable the saved-valid flag without changing live or saved field values.
    /// Live State may be absent; the saved schema must still be current/attached.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Checkpoint::clear", module = "sand::systems",
 summary = "Explicitly invalidate a saved checkpoint.", context = "Explicit integer field pairs snapshot canonical player-owned State into one canonical saved State schema. Current schema membership and all value bounds are checked before writes; restore revalidates after preparation and resolves the original online player by UUID.",
 minecraft = "Copies selected canonical integer scores with normal dirty tracking. Saved State schema migrations define snapshot migration; ephemeral tasks are never copied or automatically restored.",
 use_when = ["Saving explicit gameplay checkpoints and preparing fresh session restoration"], avoid_when = ["Arbitrary world, inventory, entity, or coroutine persistence"],

 returns = "The configured checkpoint or retained player-context actions.", example = "use sand::systems::Checkpoint;")]
    pub fn clear(&self) -> Actions {
        Self::invoke(&self.id, crate::compiler::program::CheckpointCommand::Clear)
    }

    fn schemas(&self) -> CommandResult<BTreeMap<String, &'static StateDescriptor<'static>>> {
        let error = |field, detail| CommandError::new("checkpoint", field, detail);
        if self.fields.is_empty() || self.fields.len() > 64 {
            return Err(error("fields", "requires 1..=64 explicit field pairs"));
        }
        let valid_ref = self.valid.field_reference();
        let saved_component = &valid_ref.component;
        let mut schemas = BTreeMap::new();
        for reference in std::iter::once(valid_ref.clone()).chain(
            self.fields
                .iter()
                .flat_map(|(live, saved)| [live.field_reference(), saved.field_reference()]),
        ) {
            let schema = inventory::iter::<StateDescriptor>
                .into_iter()
                .find(|s| s.id == reference.component)
                .ok_or_else(|| error("state", "requires registered canonical State"))?;
            let field = schema
                .fields
                .iter()
                .find(|f| {
                    ObjectiveName::logical(f.objective).as_str() == reference.objective
                        && f.field == reference.descriptor
                })
                .ok_or_else(|| error("state", "field metadata does not match registered State"))?;
            if matches!(schema.scope, StateScope::Global(_)) {
                return Err(error(
                    "state",
                    "requires owner-scoped State; global snapshots cannot isolate players",
                ));
            }
            if field.auto_tick || field.criterion != "dummy" {
                return Err(error(
                    "state",
                    "snapshot fields cannot tick automatically or use a native criterion",
                ));
            }
            if reference == valid_ref {
                if field.field.kind != StateFieldKind::Flag || field.field.default != 0 {
                    return Err(error("valid", "requires a Flag initially disabled"));
                }
            } else if field.field.kind != StateFieldKind::Score {
                return Err(error(
                    "fields",
                    "only ordinary integer Scores are supported",
                ));
            }
            schemas.insert(reference.component, schema);
        }
        let mut live_ids = BTreeSet::new();
        let mut saved_ids = BTreeSet::new();
        for (live, saved) in &self.fields {
            let live = live.field_reference();
            let saved = saved.field_reference();
            if saved.component != *saved_component || live.component == *saved_component {
                return Err(error(
                    "fields",
                    "saved fields and validity must share one schema, distinct from live schemas",
                ));
            }
            if !live_ids.insert(live.objective) || !saved_ids.insert(saved.objective) {
                return Err(error("fields", "live and saved fields must be unique"));
            }
        }
        if !live_ids.is_disjoint(&saved_ids) {
            return Err(error("fields", "live and saved fields must be disjoint"));
        }
        Ok(schemas)
    }
    fn dirty(schemas: &BTreeMap<String, &StateDescriptor>, field: StateFieldReference) -> bool {
        matches!(
            schemas[&field.component].scope,
            StateScope::Entity | StateScope::Living
        )
    }

    fn plan(&self) -> CommandResult<CheckpointPlan> {
        let schemas = self.schemas()?;
        let valid = self.valid.field_reference();
        let mut save = Actions::default();
        let mut restore = Actions::default();
        let mut fields = Vec::with_capacity(self.fields.len());
        for (live, saved) in &self.fields {
            fields.push(CheckpointPair::new(
                live.objective().to_string(),
                saved.objective().to_string(),
                live.descriptor().bounds,
                saved.descriptor().bounds,
            )?);
            let live = live.bind_to("@s", Self::dirty(&schemas, live.field_reference()));
            let saved = saved.bind_to("@s", Self::dirty(&schemas, saved.field_reference()));
            save.extend([saved.set(live)]);
            restore.extend([live.set(saved)]);
        }
        let flag = self
            .valid
            .bind_to("@s", Self::dirty(&schemas, valid.clone()));
        save.extend([flag.enable()]);
        let clear = flag.disable().into_commands();
        let presence = schemas
            .into_iter()
            .map(|(id, schema)| {
                (
                    id,
                    (
                        ObjectiveName::logical(schema.presence_objective),
                        schema.version,
                    ),
                )
            })
            .collect();
        Ok(CheckpointPlan {
            presence,
            valid: valid.objective,
            saved_component: valid.component,
            fields,
            save,
            restore,
            clear,
        })
    }

    pub(crate) fn portable(
        spec: &crate::compiler::program::CheckpointDefinition,
        states: &BTreeMap<String, crate::compiler::program::State>,
        before: Option<String>,
        after: Option<String>,
    ) -> CommandResult<DatapackRegistration> {
        use crate::compiler::program::FieldReference;
        use crate::state::owned::PlayerScore;

        let error = |field, message| CommandError::new("checkpoint", field, message);
        if spec.fields.is_empty() || spec.fields.len() > 64 {
            return Err(error("fields", "requires 1..=64 explicit field pairs"));
        }
        let mut presence = BTreeMap::new();
        let mut score = |reference: &FieldReference| {
            let id = reference.state.to_string();
            let field = states
                .get(&id)
                .and_then(|state| PlayerScore::from_state(state, &reference.field))
                .filter(|score| !score.timer)
                .ok_or_else(|| error("fields", "requires a non-ticking player State score"))?;
            presence.insert(id, (field.presence.clone(), field.revision));
            Ok(field)
        };
        let valid = score(&spec.valid)?;
        if valid.default != 0 || valid.bounds.is_some_and(|(min, max)| min > 0 || max < 1) {
            return Err(error(
                "valid",
                "requires a field initially disabled and able to hold 0 and 1",
            ));
        }
        let mut save = Actions::default();
        let mut restore = Actions::default();
        let mut fields = Vec::with_capacity(spec.fields.len());
        let mut seen_live = BTreeSet::new();
        let mut seen_saved = BTreeSet::new();
        for pair in &spec.fields {
            if pair.live.state == spec.valid.state
                || pair.saved.state != spec.valid.state
                || pair.saved == spec.valid
            {
                return Err(error(
                    "fields",
                    "saved fields and validity must share one schema, distinct from live schemas",
                ));
            }
            let live = score(&pair.live)?;
            let saved = score(&pair.saved)?;
            let live_name = live.objective.to_string();
            let saved_name = saved.objective.to_string();
            if !seen_live.insert(live_name.clone()) || !seen_saved.insert(saved_name.clone()) {
                return Err(error("fields", "live and saved fields must be unique"));
            }
            // The shared preflight establishes that both current values are present
            // and the source fits the destination before any write. Portable
            // player State does not have archetype dirty objectives.
            save.extend([format!(
                "scoreboard players operation @s {saved_name} = @s {live_name}"
            )]);
            restore.extend([format!(
                "scoreboard players operation @s {live_name} = @s {saved_name}"
            )]);
            fields.push(CheckpointPair::new(
                live_name,
                saved_name,
                live.bounds,
                saved.bounds,
            )?);
        }
        if !seen_live.is_disjoint(&seen_saved) || seen_saved.contains(valid.objective.as_str()) {
            return Err(error(
                "fields",
                "live, saved, and validity fields must be disjoint",
            ));
        }
        save.extend([valid.set(1)]);
        let plan = CheckpointPlan {
            presence,
            valid: valid.objective.to_string(),
            saved_component: spec.valid.state.to_string(),
            fields,
            save,
            restore,
            clear: valid.set(0).into_commands(),
        };
        let callback = |function: String| -> PlayerAction {
            Arc::new(move |_| Cmd::Function(function.clone()).into_commands())
        };
        let before = before.map(callback);
        let after = after.map(callback);
        let id = spec.id.clone();
        Ok(register_checkpoint(&spec.id, move |part| {
            plan.build(&id, part, before.as_ref(), after.as_ref())
        }))
    }

    pub(crate) fn invoke(
        id: &ResourceLocation,
        command: crate::compiler::program::CheckpointCommand,
    ) -> Actions {
        use crate::compiler::program::CheckpointCommand;
        let part = match command {
            CheckpointCommand::Save => "checkpoint_save",
            CheckpointCommand::Restore => "checkpoint_restore",
            CheckpointCommand::Clear => "checkpoint_clear",
        };
        identity::call(id, part)
    }

    fn build(&self, part: &str) -> CommandResult<Actions> {
        self.plan()?
            .build(&self.id, part, self.before.as_ref(), self.after.as_ref())
    }
}

struct CheckpointPair {
    live: String,
    saved: String,
    range: (i32, i32),
}

impl CheckpointPair {
    fn new(
        live: String,
        saved: String,
        live_bounds: Option<(i32, i32)>,
        saved_bounds: Option<(i32, i32)>,
    ) -> CommandResult<Self> {
        let live_bounds = live_bounds.unwrap_or((i32::MIN, i32::MAX));
        let saved_bounds = saved_bounds.unwrap_or((i32::MIN, i32::MAX));
        let min = live_bounds.0.max(saved_bounds.0);
        let max = live_bounds.1.min(saved_bounds.1);
        if min > max {
            return Err(CommandError::new(
                "checkpoint",
                "fields",
                "a field pair has no common representable integer range",
            ));
        }
        Ok(Self {
            live,
            saved,
            range: (min, max),
        })
    }
}

struct CheckpointPlan {
    presence: BTreeMap<String, (ObjectiveName, u32)>,
    valid: String,
    saved_component: String,
    fields: Vec<CheckpointPair>,
    save: Actions,
    restore: Actions,
    clear: Actions,
}

impl CheckpointPlan {
    fn ready(&self, restore: bool, saved_only: bool) -> Actions {
        let mut actions =
            "execute unless entity @s[type=minecraft:player] run return 0".into_commands();
        for (schema, (presence, version)) in &self.presence {
            if !saved_only || schema == &self.saved_component {
                actions.extend([format!(
                    "execute unless score @s {presence} matches {version} run return 0"
                )]);
            }
        }
        let valid = &self.valid;
        actions.extend([format!(
            "execute unless score @s {valid} matches {} run return 0",
            if restore { "1" } else { "0..1" }
        )]);
        if !saved_only {
            for pair in &self.fields {
                for objective in [&pair.live, &pair.saved] {
                    actions.extend([format!(
                        "execute unless score @s {objective} matches -2147483648..2147483647 run return 0",
                    )]);
                }
                let source = if restore { &pair.saved } else { &pair.live };
                let (min, max) = pair.range;
                actions.extend([format!(
                    "execute unless score @s {source} matches {min}..{max} run return 0",
                )]);
            }
        }
        actions
    }

    fn build(
        &self,
        location: &ResourceLocation,
        part: &str,
        before: Option<&PlayerAction>,
        after: Option<&PlayerAction>,
    ) -> CommandResult<Actions> {
        let id = crate::function::resolve_export_location(location)?;
        let pack_namespace = identity::pack_namespace()?;
        let lock =
            ObjectiveName::logical(format!("sand checkpoint {pack_namespace} transaction lock"));
        let result = ObjectiveName::logical(format!("sand checkpoint {pack_namespace} result"));
        crate::function::request_numeric_objectives([lock.clone(), result.clone()])?;
        let scratch = identity::resource(&id, "checkpoint_scratch");
        let function = |part: &str| identity::resource(&id, &format!("checkpoint_{part}"));
        if part == "load" {
            return Ok(format!(
                "scoreboard players set #busy {lock} 0\ndata remove storage {scratch} operation"
            )
            .into_commands());
        }
        if part == "restore_owner" {
            let mut actions = self.ready(true, false);
            if let Some(callback) = before {
                actions.extend([runtime::isolated(callback(EntityContext::default()))]);
            }
            actions.extend([format!(
                "$execute as @a[nbt={{UUID:$(owner)}},limit=1] at @s run function {}",
                function("restore_commit")
            )]);
            return Ok(actions);
        }
        if part == "restore_commit" {
            let mut actions = self.ready(true, false);
            actions.extend([self.restore.clone()]);
            actions.extend([format!("scoreboard players set #accepted {result} 1")]);
            if let Some(callback) = after {
                actions.extend([runtime::isolated(callback(EntityContext::default()))]);
            }
            return Ok(actions);
        }
        let mut actions =
            "execute unless entity @s[type=minecraft:player] run return 0".into_commands();
        actions.extend([
            format!("execute if score #busy {lock} matches 1 run return 0"),
            format!("scoreboard players set #busy {lock} 1"),
            format!("scoreboard players set #accepted {result} 0"),
            format!("data remove storage {scratch} operation"),
        ]);
        let mut operation = self.ready(part == "restore", part == "clear");
        match part {
            "save" => operation.extend([
                self.save.clone(),
                format!("scoreboard players set #accepted {result} 1").into_commands(),
            ]),
            "clear" => operation.extend([
                self.clear.clone(),
                format!("scoreboard players set #accepted {result} 1").into_commands(),
            ]),
            "restore" => operation.extend([
                format!("data modify storage {scratch} operation.owner set from entity @s UUID"),
                format!(
                    "function {} with storage {scratch} operation",
                    function("restore_owner")
                ),
            ]),
            _ => unreachable!(),
        }
        actions.extend([
            runtime::isolated(operation),
            format!("data remove storage {scratch} operation").into_commands(),
            format!("scoreboard players set #busy {lock} 0").into_commands(),
            Cmd::ReturnRun(Box::new(Cmd::Raw(format!(
                "scoreboard players get #accepted {result}"
            ))))
            .into_commands(),
        ]);
        Ok(actions)
    }
}
impl IntoDatapack for Checkpoint {
    fn into_datapack(self) -> DatapackRegistration {
        register_checkpoint(&self.id.clone(), move |part| self.build(part))
    }
}

fn register_checkpoint(
    id: &ResourceLocation,
    build: impl Fn(&str) -> CommandResult<Actions> + 'static,
) -> DatapackRegistration {
    let build = Arc::new(build);
    let mut registration = DatapackRegistration::new().lifecycle(LifecycleContribution::load(
        format!("function {}", identity::resource(id, "checkpoint_load")),
    ));
    for part in [
        "load",
        "save",
        "restore",
        "clear",
        "restore_owner",
        "restore_commit",
    ] {
        let build = build.clone();
        registration = registration.component(runtime::Function::new(
            id,
            &format!("checkpoint_{part}"),
            move || build(part),
        ));
    }
    registration
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checkpoints_require_bounded_explicit_registered_field_pairs() {
        let valid = EntityFlag::new("checkpoint_test", "saved", "valid", false);
        let checkpoint = Checkpoint::new("checkpoint_test:snapshot".parse().unwrap(), valid);
        assert_eq!(checkpoint.schemas().unwrap_err().field, "fields");
        let live = EntityScore::new("checkpoint_test", "live", "value", 0, None);
        let saved = EntityScore::new("checkpoint_test", "saved", "value", 0, None);
        assert!(
            checkpoint
                .clone()
                .field(live, saved)
                .schemas()
                .unwrap_err()
                .to_string()
                .contains("registered canonical State")
        );
        let mut oversized = checkpoint;
        for _ in 0..65 {
            oversized = oversized.field(live, saved);
        }
        assert_eq!(oversized.schemas().unwrap_err().field, "fields");
    }
}
