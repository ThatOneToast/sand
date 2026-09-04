//! Immutable component lifecycle descriptors emitted by `#[derive(State)]`.
//!
//! Descriptors are collected at link time and rebuilt into deterministic,
//! export-scoped load/init/tick work for every export. The descriptor is
//! schema-oriented because presence, version, and ownership belong to a
//! component rather than to any one field.

use std::collections::BTreeMap;

use super::score::objective_name;
use crate::entity::StateFieldDescriptor;
use crate::entity::state::{StateDataFieldDescriptor, numeric_scratch_name};

/// Runtime ownership scope for a derived state component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateScope {
    /// State owned by players observed online.
    Player,
    /// State explicitly attached to arbitrary loaded entities.
    Entity,
    /// State explicitly attached to loaded living entities.
    Living,
    /// Singleton state stored on a deterministic fake score holder.
    Global(&'static str),
}

/// Compiler descriptor for one scoreboard-backed component field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateLifecycleDescriptor {
    /// Resolved objective name.
    pub objective: &'static str,
    /// Vanilla scoreboard criterion.
    pub criterion: &'static str,
    /// Optional objective display name.
    pub display_name: Option<&'static str>,
    /// Field metadata shared with the generated typed handle.
    pub field: StateFieldDescriptor,
    /// Whether this timer/cooldown decrements once per lifecycle tick.
    pub auto_tick: bool,
}

impl StateLifecycleDescriptor {
    /// Construct lifecycle metadata for one field.
    pub const fn new(objective: &'static str, field: StateFieldDescriptor) -> Self {
        Self {
            objective,
            criterion: "dummy",
            display_name: None,
            field,
            auto_tick: false,
        }
    }

    /// Override the vanilla scoreboard criterion.
    pub const fn criterion(mut self, criterion: &'static str) -> Self {
        self.criterion = criterion;
        self
    }

    /// Set the objective display name.
    pub const fn display_name(mut self, display_name: &'static str) -> Self {
        self.display_name = Some(display_name);
        self
    }

    /// Enable countdown ticking.
    pub const fn auto_tick(mut self) -> Self {
        self.auto_tick = true;
        self
    }
}

/// Link-time descriptor for one independently attachable State component.
#[derive(Debug, Clone, Copy)]
pub struct StateDescriptor {
    /// Stable logical `namespace:name` identity.
    pub id: &'static str,
    /// Current schema version. Zero is reserved for absence.
    pub version: u32,
    /// Owner scope.
    pub scope: StateScope,
    /// Presence/version objective.
    pub presence_objective: &'static str,
    /// Player-only explicit-detachment suppression objective.
    pub suppression_objective: &'static str,
    /// Fields in declaration order.
    pub fields: &'static [StateLifecycleDescriptor],
    /// Ordered component-version transitions.
    pub migrations: &'static [StateMigrationDescriptor],
    /// Component-owned typed storage paths.
    pub data_fields: &'static [StateDataFieldDescriptor],
}

/// One contiguous component presence/version transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateMigrationDescriptor {
    pub from: u32,
    pub to: u32,
}

impl StateMigrationDescriptor {
    pub const fn new(from: u32, to: u32) -> Self {
        Self { from, to }
    }
}

impl StateDescriptor {
    /// Construct an immutable component descriptor.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        id: &'static str,
        version: u32,
        scope: StateScope,
        presence_objective: &'static str,
        suppression_objective: &'static str,
        fields: &'static [StateLifecycleDescriptor],
        migrations: &'static [StateMigrationDescriptor],
        data_fields: &'static [StateDataFieldDescriptor],
    ) -> Self {
        Self {
            id,
            version,
            scope,
            presence_objective,
            suppression_objective,
            fields,
            migrations,
            data_fields,
        }
    }
}

inventory::collect!(StateDescriptor);

/// Link-time callbacks emitted by `#[state_lifecycle]`.
#[doc(hidden)]
pub struct StateHookDescriptor {
    /// Resolve the State schema implemented by the hook owner.
    pub schema: fn() -> crate::entity::StateSchema,
    /// Custom load-time provisioning.
    pub provision: fn() -> Vec<String>,
    /// Custom initialization.
    pub initialize: fn(&'static str) -> Vec<String>,
    /// Custom tick behavior.
    pub tick: fn(&'static str) -> Vec<String>,
    /// Custom reconciliation.
    pub reconcile: fn(&'static str) -> Vec<String>,
    /// Custom cleanup.
    pub cleanup: fn(&'static str) -> Vec<String>,
    /// Custom version-transition behavior.
    pub migrate: fn(&'static str, u32, u32) -> Vec<String>,
}

inventory::collect!(StateHookDescriptor);

pub(crate) fn initialize_hook_commands<S: crate::entity::EntityState>(
    holder: &'static str,
) -> Vec<String> {
    let id = S::schema().id();
    inventory::iter::<StateHookDescriptor>
        .into_iter()
        .find(|hook| (hook.schema)().id() == id)
        .map_or_else(Vec::new, |hook| {
            let mut commands = (hook.initialize)(holder);
            commands.extend((hook.reconcile)(holder));
            commands
        })
}

pub(crate) fn cleanup_hook_commands<S: crate::entity::EntityState>(
    holder: &'static str,
) -> Vec<String> {
    let id = S::schema().id();
    inventory::iter::<StateHookDescriptor>
        .into_iter()
        .find(|hook| (hook.schema)().id() == id)
        .map_or_else(Vec::new, |hook| (hook.cleanup)(holder))
}

pub(crate) fn migration_steps<S: crate::entity::EntityState>() -> Vec<(u32, u32)> {
    let id = S::schema().id();
    inventory::iter::<StateDescriptor>
        .into_iter()
        .find(|descriptor| descriptor.id == id)
        .map_or_else(Vec::new, |descriptor| {
            descriptor
                .migrations
                .iter()
                .map(|step| (step.from, step.to))
                .collect()
        })
}

pub(crate) fn migrate_hook_commands<S: crate::entity::EntityState>(
    holder: &'static str,
    from: u32,
    to: u32,
) -> Vec<String> {
    let id = S::schema().id();
    inventory::iter::<StateHookDescriptor>
        .into_iter()
        .find(|hook| (hook.schema)().id() == id)
        .map_or_else(Vec::new, |hook| (hook.migrate)(holder, from, to))
}

/// Deterministic automatic lifecycle output built afresh for each export.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct AutomaticLifecycle {
    pub load_commands: Vec<String>,
    pub provision_commands: Vec<String>,
    pub player_init_commands: Vec<String>,
    pub player_tick_commands: Vec<String>,
    pub entity_tick_commands: Vec<String>,
    pub global_init_commands: Vec<String>,
    pub global_tick_commands: Vec<String>,
}

/// Resolve all link-time component declarations.
pub(crate) fn automatic_lifecycle() -> Result<AutomaticLifecycle, String> {
    automatic_lifecycle_from(inventory::iter::<StateDescriptor>.into_iter().copied())
}

fn automatic_lifecycle_from(
    declarations: impl IntoIterator<Item = StateDescriptor>,
) -> Result<AutomaticLifecycle, String> {
    let mut components = BTreeMap::<&'static str, StateDescriptor>::new();
    for declaration in declarations {
        match components.get(declaration.id) {
            Some(existing) if descriptor_eq(existing, &declaration) => {}
            Some(_) => {
                return Err(format!(
                    "conflicting State component declarations for `{}`",
                    declaration.id
                ));
            }
            None => {
                components.insert(declaration.id, declaration);
            }
        }
    }

    let mut output = AutomaticLifecycle::default();
    let mut hooks = BTreeMap::<String, &StateHookDescriptor>::new();
    for hook in inventory::iter::<StateHookDescriptor> {
        let id = (hook.schema)().id();
        if hooks.insert(id.clone(), hook).is_some() {
            return Err(format!(
                "multiple #[state_lifecycle] implementations registered for `{id}`"
            ));
        }
    }
    let mut objectives = BTreeMap::<String, (String, String)>::new();
    for component in components.values() {
        let hook = hooks.get(component.id).copied();
        if let Some(hook) = hook {
            output.provision_commands.extend((hook.provision)());
        }
        let presence = objective_name(component.presence_objective);
        let suppression = objective_name(component.suppression_objective);
        claim_objective(
            &mut objectives,
            &presence,
            "dummy",
            &format!("{}::presence", component.id),
        )?;
        output
            .load_commands
            .push(format!("scoreboard objectives add {} dummy", presence));
        if matches!(component.scope, StateScope::Player) {
            claim_objective(
                &mut objectives,
                &suppression,
                "dummy",
                &format!("{}::suppression", component.id),
            )?;
            output
                .load_commands
                .push(format!("scoreboard objectives add {} dummy", suppression));
        }

        if component.fields.iter().any(|field| {
            matches!(
                field.field.kind,
                crate::entity::StateFieldKind::Score
                    | crate::entity::StateFieldKind::Fixed(_)
                    | crate::entity::StateFieldKind::Version
                    | crate::entity::StateFieldKind::Dirty
            )
        }) {
            let (namespace, schema) = component
                .id
                .split_once(':')
                .expect("State component IDs contain a namespace");
            let numeric_scratch = numeric_scratch_name(namespace, schema);
            claim_objective(
                &mut objectives,
                &numeric_scratch,
                "dummy",
                &format!("{}::numeric_scratch", component.id),
            )?;
            output
                .load_commands
                .push(format!("scoreboard objectives add {numeric_scratch} dummy"));
        }

        for field in component.fields {
            let objective = objective_name(field.objective);
            claim_objective(
                &mut objectives,
                &objective,
                field.criterion,
                &format!("{}::{}", component.id, field.field.name),
            )?;
            let display = field
                .display_name
                .map(|name| {
                    format!(
                        " {}",
                        serde_json::to_string(name).expect("string serializes")
                    )
                })
                .unwrap_or_default();
            output.load_commands.push(format!(
                "scoreboard objectives add {} {}{}",
                objective, field.criterion, display
            ));
            if matches!(component.scope, StateScope::Entity | StateScope::Living) {
                let dirty = objective_name(&format!("{}.dirty", field.objective));
                claim_objective(
                    &mut objectives,
                    &dirty,
                    "dummy",
                    &format!("{}::{}::dirty", component.id, field.field.name),
                )?;
                output
                    .load_commands
                    .push(format!("scoreboard objectives add {dirty} dummy"));
            }
        }
        if matches!(component.scope, StateScope::Entity | StateScope::Living) {
            let reconcile_dirty = component_reconcile_dirty(component.id);
            claim_objective(
                &mut objectives,
                &reconcile_dirty,
                "dummy",
                &format!("{}::reconcile_dirty", component.id),
            )?;
            output
                .load_commands
                .push(format!("scoreboard objectives add {reconcile_dirty} dummy"));
        }
        for field in component.data_fields.iter().filter(|field| field.keyed) {
            output.provision_commands.push(format!(
                "execute unless data storage {} owners run data modify storage {} owners set value []",
                field.storage, field.storage
            ));
        }

        match component.scope {
            StateScope::Player => emit_player(component, hook, &mut output),
            StateScope::Entity | StateScope::Living => emit_entity(component, hook, &mut output),
            StateScope::Global(holder) => emit_global(component, holder, hook, &mut output),
        }
    }
    output.load_commands.sort();
    output.load_commands.dedup();
    output.provision_commands.sort();
    output.provision_commands.dedup();
    Ok(output)
}

fn emit_player(
    component: &StateDescriptor,
    hook: Option<&StateHookDescriptor>,
    output: &mut AutomaticLifecycle,
) {
    let presence = objective_name(component.presence_objective);
    let suppression = objective_name(component.suppression_objective);
    let guard = format!("unless score @s {} matches 1..", suppression);
    for field in component.fields {
        let objective = objective_name(field.objective);
        output.player_init_commands.push(format!(
            "execute {guard} unless score @s {} matches -2147483648.. run scoreboard players set @s {} {}",
            objective, objective, field.field.default
        ));
        if field.auto_tick {
            output.player_tick_commands.push(format!(
                "execute if score @s {} matches 1.. if score @s {} matches 1.. run scoreboard players remove @s {} 1",
                presence, objective, objective
            ));
        }
    }
    for field in component.data_fields {
        output.player_init_commands.extend(
            crate::entity::state::state_data_initialize_commands(*field)
                .into_iter()
                .map(|command| format!("execute {guard} run {command}")),
        );
    }
    if let Some(hook) = hook {
        for command in (hook.initialize)("@s") {
            output.player_init_commands.push(format!(
                "execute {guard} unless score @s {presence} matches 1.. run {command}"
            ));
        }
        for command in (hook.reconcile)("@s") {
            output.player_init_commands.push(format!(
                "execute {guard} unless score @s {presence} matches 1.. run {command}"
            ));
        }
        for command in (hook.tick)("@s") {
            output.player_tick_commands.push(format!(
                "execute if score @s {presence} matches {} run {command}",
                component.version
            ));
        }
    }
    for migration in component.migrations {
        if let Some(hook) = hook {
            for command in (hook.migrate)("@s", migration.from, migration.to) {
                output.player_init_commands.push(format!(
                    "execute {guard} if score @s {presence} matches {} run {command}",
                    migration.from
                ));
            }
        }
        output.player_init_commands.push(format!(
            "execute {guard} if score @s {presence} matches {} run scoreboard players set @s {presence} {}",
            migration.from, migration.to
        ));
    }
    output.player_init_commands.push(format!(
        "execute {guard} unless score @s {presence} matches 1.. run scoreboard players set @s {presence} {}",
        component.version
    ));
}

fn emit_entity(
    component: &StateDescriptor,
    hook: Option<&StateHookDescriptor>,
    output: &mut AutomaticLifecycle,
) {
    let presence = objective_name(component.presence_objective);
    let reconcile_dirty = component_reconcile_dirty(component.id);
    let mut body = Vec::new();
    for field in component.fields.iter().filter(|field| field.auto_tick) {
        let objective = objective_name(field.objective);
        let dirty = objective_name(&format!("{}.dirty", field.objective));
        body.push(format!(
            "execute if score @s {objective} matches 1.. run scoreboard players set @s {dirty} 1"
        ));
        body.push(format!(
            "execute if score @s {objective} matches 1.. run scoreboard players set @s {reconcile_dirty} 1"
        ));
        body.push(format!(
            "execute if score @s {objective} matches 1.. run scoreboard players remove @s {objective} 1"
        ));
    }
    if let Some(hook) = hook {
        body.extend((hook.tick)("@s"));
        let reconcile = (hook.reconcile)("@s");
        if !reconcile.is_empty() {
            let mut reconcile_body = reconcile;
            reconcile_body.push(format!("scoreboard players reset @s {reconcile_dirty}"));
            let reconcile_path =
                crate::function::register_dyn_fn_dedup("sand/state_reconcile", reconcile_body);
            body.push(format!(
                "execute if score @s {reconcile_dirty} matches 1.. run function __sand_local:{reconcile_path}"
            ));
        }
    }
    if !body.is_empty() {
        let path = crate::function::register_dyn_fn_dedup("sand/state_tick", body);
        output.entity_tick_commands.push(format!(
            "execute as @e[scores={{{presence}={}}}] at @s run function __sand_local:{path}",
            component.version
        ));
    }
}

fn component_reconcile_dirty(id: &str) -> String {
    objective_name(&format!("{id}.reconcile_dirty"))
}

fn emit_global(
    component: &StateDescriptor,
    holder: &'static str,
    hook: Option<&StateHookDescriptor>,
    output: &mut AutomaticLifecycle,
) {
    let presence = objective_name(component.presence_objective);
    for field in component.fields {
        let objective = objective_name(field.objective);
        output.global_init_commands.push(format!(
            "execute unless score {holder} {} matches -2147483648.. run scoreboard players set {holder} {} {}",
            objective, objective, field.field.default
        ));
        if field.auto_tick {
            output.global_tick_commands.push(format!(
                "execute if score {holder} {presence} matches {} if score {holder} {} matches 1.. run scoreboard players remove {holder} {} 1",
                component.version, objective, objective
            ));
        }
    }
    for field in component.data_fields {
        output.global_init_commands.push(format!(
            "execute unless data storage {} {} run data modify storage {} {} set value {}",
            field.storage, field.path, field.storage, field.path, field.default_snbt
        ));
    }
    if let Some(hook) = hook {
        for command in (hook.initialize)(holder) {
            output.global_init_commands.push(format!(
                "execute unless score {holder} {presence} matches 1.. run {command}"
            ));
        }
        for command in (hook.reconcile)(holder) {
            output.global_init_commands.push(format!(
                "execute unless score {holder} {presence} matches 1.. run {command}"
            ));
        }
        for command in (hook.tick)(holder) {
            output.global_tick_commands.push(format!(
                "execute if score {holder} {presence} matches {} run {command}",
                component.version
            ));
        }
    }
    for migration in component.migrations {
        if let Some(hook) = hook {
            for command in (hook.migrate)(holder, migration.from, migration.to) {
                output.global_init_commands.push(format!(
                    "execute if score {holder} {presence} matches {} run {command}",
                    migration.from
                ));
            }
        }
        output.global_init_commands.push(format!(
            "execute if score {holder} {presence} matches {} run scoreboard players set {holder} {presence} {}",
            migration.from, migration.to
        ));
    }
    output.global_init_commands.push(format!(
        "execute unless score {holder} {presence} matches 1.. run scoreboard players set {holder} {presence} {}",
        component.version
    ));
}

fn claim_objective(
    objectives: &mut BTreeMap<String, (String, String)>,
    objective: &str,
    criterion: &str,
    owner: &str,
) -> Result<(), String> {
    match objectives.get(objective) {
        Some((existing_criterion, existing_owner))
            if existing_criterion != criterion || existing_owner != owner =>
        {
            Err(format!(
                "generated objective `{objective}` for `{owner}` conflicts with `{existing_owner}`"
            ))
        }
        _ => {
            objectives.insert(
                objective.to_owned(),
                (criterion.to_owned(), owner.to_owned()),
            );
            Ok(())
        }
    }
}

fn descriptor_eq(left: &StateDescriptor, right: &StateDescriptor) -> bool {
    left.id == right.id
        && left.version == right.version
        && left.scope == right.scope
        && left.presence_objective == right.presence_objective
        && left.suppression_objective == right.suppression_objective
        && left.fields == right.fields
        && left.migrations == right.migrations
        && left.data_fields == right.data_fields
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::StateFieldKind;

    const TIMER: StateFieldDescriptor =
        StateFieldDescriptor::new("timer", StateFieldKind::Timer, 0, Some((0, i32::MAX)));
    const FIELDS: &[StateLifecycleDescriptor] =
        &[StateLifecycleDescriptor::new("timer_obj", TIMER).auto_tick()];

    #[test]
    fn player_component_initializes_idempotently_and_publishes_presence_last() {
        let output = automatic_lifecycle_from([StateDescriptor::new(
            "demo:player",
            2,
            StateScope::Player,
            "presence",
            "suppressed",
            FIELDS,
            &[],
            &[],
        )])
        .unwrap();
        assert!(
            output
                .load_commands
                .contains(&"scoreboard objectives add presence dummy".into())
        );
        assert!(
            output
                .player_init_commands
                .last()
                .unwrap()
                .ends_with("presence 2")
        );
        assert!(
            output
                .player_init_commands
                .iter()
                .all(|line| line.contains("unless score @s suppressed matches 1.."))
        );
    }

    #[test]
    fn numeric_components_provision_one_internal_scale_scratch() {
        const SCORE: StateFieldDescriptor =
            StateFieldDescriptor::new("health", StateFieldKind::Score, 20, None);
        const NUMERIC_FIELDS: &[StateLifecycleDescriptor] =
            &[StateLifecycleDescriptor::new("health_obj", SCORE)];
        let output = automatic_lifecycle_from([StateDescriptor::new(
            "demo:stats",
            1,
            StateScope::Entity,
            "presence",
            "unused",
            NUMERIC_FIELDS,
            &[],
            &[],
        )])
        .unwrap();
        let scratch = numeric_scratch_name("demo", "stats");
        assert!(
            output
                .load_commands
                .contains(&format!("scoreboard objectives add {scratch} dummy"))
        );
        assert_eq!(
            output
                .load_commands
                .iter()
                .filter(|command| command.contains(&scratch))
                .count(),
            1
        );
    }

    #[test]
    fn version_and_dirty_only_components_provision_numeric_scratch() {
        const VERSION: StateFieldDescriptor =
            StateFieldDescriptor::new("version", StateFieldKind::Version, 1, Some((0, 10)));
        const DIRTY: StateFieldDescriptor =
            StateFieldDescriptor::new("dirty", StateFieldKind::Dirty, 0, Some((0, 1)));
        const INTERNAL_NUMERIC_FIELDS: &[StateLifecycleDescriptor] = &[
            StateLifecycleDescriptor::new("version_obj", VERSION),
            StateLifecycleDescriptor::new("dirty_obj", DIRTY),
        ];
        let output = automatic_lifecycle_from([StateDescriptor::new(
            "demo:internal_only",
            1,
            StateScope::Entity,
            "presence",
            "unused",
            INTERNAL_NUMERIC_FIELDS,
            &[],
            &[],
        )])
        .unwrap();
        let scratch = numeric_scratch_name("demo", "internal_only");
        let expected = format!("scoreboard objectives add {scratch} dummy");
        assert_eq!(
            output
                .load_commands
                .iter()
                .filter(|command| *command == &expected)
                .count(),
            1
        );
    }

    #[test]
    fn entity_auto_tick_is_presence_constrained() {
        let output = automatic_lifecycle_from([StateDescriptor::new(
            "demo:mob",
            1,
            StateScope::Entity,
            "presence",
            "unused",
            FIELDS,
            &[],
            &[],
        )])
        .unwrap();
        assert_eq!(output.entity_tick_commands.len(), 1);
        assert!(output.entity_tick_commands[0].contains("@e[scores={presence=1}]"));
        let callbacks = crate::function::drain_dyn_fns();
        assert_eq!(callbacks.len(), 1);
        assert_eq!(
            callbacks[0].1,
            vec![
                "execute if score @s timer_obj matches 1.. run scoreboard players set @s timer_obj.dirty 1".to_owned(),
                format!(
                    "execute if score @s timer_obj matches 1.. run scoreboard players set @s {} 1",
                    component_reconcile_dirty("demo:mob")
                ),
                "execute if score @s timer_obj matches 1.. run scoreboard players remove @s timer_obj 1".to_owned()
            ]
        );
    }

    #[test]
    fn component_output_is_registration_order_independent() {
        const PLAYER_FIELDS: &[StateLifecycleDescriptor] =
            &[StateLifecycleDescriptor::new("player_timer", TIMER).auto_tick()];
        let player = StateDescriptor::new(
            "demo:player",
            1,
            StateScope::Player,
            "player_presence",
            "player_suppressed",
            PLAYER_FIELDS,
            &[],
            &[],
        );
        let entity = StateDescriptor::new(
            "demo:entity",
            1,
            StateScope::Entity,
            "entity_presence",
            "unused",
            FIELDS,
            &[],
            &[],
        );
        assert_eq!(
            automatic_lifecycle_from([player, entity]).unwrap(),
            automatic_lifecycle_from([entity, player]).unwrap()
        );
    }

    #[test]
    fn objective_collision_names_both_logical_owners() {
        const OTHER_TIMER: StateFieldDescriptor =
            StateFieldDescriptor::new("other_timer", StateFieldKind::Timer, 0, Some((0, i32::MAX)));
        const OTHER_FIELDS: &[StateLifecycleDescriptor] =
            &[StateLifecycleDescriptor::new("timer_obj", OTHER_TIMER).auto_tick()];
        let error = automatic_lifecycle_from([
            StateDescriptor::new(
                "demo:first",
                1,
                StateScope::Entity,
                "first_presence",
                "unused",
                FIELDS,
                &[],
                &[],
            ),
            StateDescriptor::new(
                "demo:second",
                1,
                StateScope::Entity,
                "second_presence",
                "unused",
                OTHER_FIELDS,
                &[],
                &[],
            ),
        ])
        .unwrap_err();
        assert!(error.contains("timer_obj"));
        assert!(error.contains("demo:first::timer"));
        assert!(error.contains("demo:second::other_timer"));
    }
}
