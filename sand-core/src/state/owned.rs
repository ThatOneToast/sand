//! Owned adaptation of portable State through canonical lifecycle lowering.
//!
//! Strings are borrowed only while canonical lifecycle generation runs. No
//! descriptor is interned, leaked or registered in link-time inventory.

use std::collections::BTreeMap;

use super::GlobalStateHolder;
use super::registry::{AutomaticLifecycle, automatic_lifecycle_with_hooks};
use super::registry::{StateDescriptor, StateLifecycleDescriptor, StateScope};
use crate::entity::state::StateDataFieldDescriptor;
use crate::entity::{StateFieldDescriptor, StateFieldKind};

use crate::compiler::program::model::{ScoreField, State};

impl TryFrom<StateDescriptor<'_>> for State {
    type Error = String;

    fn try_from(descriptor: StateDescriptor<'_>) -> Result<Self, Self::Error> {
        let scope = match descriptor.scope {
            StateScope::Player => "player",
            StateScope::Entity => "entity",
            StateScope::Global(holder)
                if holder == GlobalStateHolder::new(descriptor.id).as_str() =>
            {
                "global"
            }
            _ => {
                return Err(format!(
                    "State `{}` has an unsupported scope or noncanonical global holder",
                    descriptor.id
                ));
            }
        };
        if descriptor.version != 1
            || !descriptor.data_fields.is_empty()
            || !descriptor.migrations.is_empty()
        {
            return Err(format!(
                "State `{}` is outside the player/entity/global score v1 slice",
                descriptor.id
            ));
        }
        for field in descriptor.fields {
            if field.field.kind != StateFieldKind::Score
                || field.criterion != "dummy"
                || field.display_name.is_some()
                || field.objective != format!("{}.{}", descriptor.id, field.field.name)
            {
                return Err(format!(
                    "State field `{}.{}` is outside the player/entity/global score v1 slice",
                    descriptor.id, field.field.name
                ));
            }
        }
        if descriptor.presence_objective != format!("{}.presence", descriptor.id)
            || descriptor.suppression_objective != format!("{}.suppressed", descriptor.id)
        {
            return Err(format!(
                "State `{}` has noncanonical ownership objectives",
                descriptor.id
            ));
        }
        Ok(Self {
            data_fields: Vec::new(),
            id: descriptor
                .id
                .parse()
                .map_err(|error| format!("invalid State ID: {error}"))?,
            scope: scope.into(),
            revision: 1,
            fields: descriptor
                .fields
                .iter()
                .map(|field| ScoreField {
                    name: field.field.name.to_owned(),
                    default: field.field.default,
                    bounds: field.field.bounds,
                    timer: field.auto_tick,
                })
                .collect(),
        })
    }
}

/// Resolve the scoreboard owner from the same identity used by lifecycle lowering.
pub(crate) fn holder(state: &State) -> sand_commands::ScoreHolder {
    match state.scope.as_str() {
        "player" | "entity" => sand_commands::ScoreHolder::self_(),
        "global" => {
            sand_commands::ScoreHolder::fake(GlobalStateHolder::new(&state.id.to_string()).as_str())
        }
        _ => unreachable!("owned State scopes are validated before lowering"),
    }
}

/// A canonical player score and its owning State metadata, retained by
/// runtime systems without interning any of the portable input.
#[derive(Debug, Clone)]
pub(crate) struct PlayerScore {
    pub(crate) objective: sand_commands::ObjectiveName,
    pub(crate) presence: sand_commands::ObjectiveName,
    pub(crate) revision: u32,
    pub(crate) default: i32,
    pub(crate) bounds: Option<(i32, i32)>,
    pub(crate) timer: bool,
}

impl PlayerScore {
    pub(crate) fn from_state(state: &State, name: &str) -> Option<Self> {
        let field = state.fields.iter().find(|field| field.name == name)?;
        (state.scope == "player").then(|| Self {
            objective: sand_commands::ObjectiveName::logical(format!("{}.{name}", state.id)),
            presence: sand_commands::ObjectiveName::logical(format!("{}.presence", state.id)),
            revision: state.revision,
            default: field.default,
            bounds: field.bounds,
            timer: field.timer,
        })
    }

    pub(crate) fn matches(
        &self,
        range: crate::condition::ScoreRange,
    ) -> crate::condition::Condition {
        use crate::condition::{Condition, ScoreRange};
        Condition::all([
            Condition::score(
                "@s".into(),
                self.presence.to_string(),
                ScoreRange::Eq(self.revision as i32),
            ),
            self.value_matches(range),
        ])
    }

    pub(crate) fn value_matches(
        &self,
        range: crate::condition::ScoreRange,
    ) -> crate::condition::Condition {
        crate::condition::Condition::score("@s".into(), self.objective.to_string(), range)
    }

    pub(crate) fn remove(&self, amount: i32) -> crate::ir::Actions {
        use crate::{IntoCommands, ir::Cmd};
        let objective = &self.objective;
        let mut actions =
            Cmd::Raw(format!("scoreboard players remove @s {objective} {amount}")).into_commands();
        if let Some((_, max)) = self.bounds
            && max < i32::MAX
        {
            actions.extend([Cmd::Raw(format!(
                "execute if score @s {objective} matches {}.. run scoreboard players set @s {objective} {max}",
                max + 1,
            ))]);
        }
        actions
    }

    pub(crate) fn set(&self, value: i32) -> crate::ir::Cmd {
        crate::ir::Cmd::Raw(format!(
            "scoreboard players set @s {} {value}",
            self.objective
        ))
    }
}

/// Borrow one portable field through the same canonical storage identity used
/// by derived Rust State components.
pub(crate) fn with_data_field<R>(
    state: &State,
    name: &str,
    lower: impl FnOnce(StateDataFieldDescriptor<'_>) -> R,
) -> R {
    let field = state
        .data_fields
        .iter()
        .find(|field| field.name == name)
        .expect("validated storage field");
    let (storage, path, default) = data_identity(state, field);
    lower(if state.scope == "global" {
        StateDataFieldDescriptor::new(&storage, &path, &default)
    } else {
        StateDataFieldDescriptor::keyed(&storage, &path, &default)
    })
}

fn data_identity(
    state: &State,
    field: &crate::compiler::program::model::DataField,
) -> (String, String, String) {
    let id = state.id.to_string();
    let (namespace, component) = id.split_once(':').expect("validated State identity");
    let component = serde_json::to_string(component).expect("component key serializes");
    (
        format!("{namespace}:state"),
        format!("components.{component}.{}", field.name),
        field.default.to_string(),
    )
}

/// Lower explicit owned inputs through the same lifecycle implementation used
/// by Rust descriptors, without collecting or executing Rust hooks.
pub(crate) fn lifecycle(states: &[State]) -> Result<AutomaticLifecycle, String> {
    with_descriptors(states, |descriptors| {
        automatic_lifecycle_with_hooks(descriptors, &BTreeMap::new())
    })
}

/// Borrow owned State data for one canonical lowering pass. The callback cannot
/// retain these descriptors beyond their owned backing storage.
pub(crate) fn with_descriptors<R>(
    states: &[State],
    lower: impl FnOnce(Vec<StateDescriptor<'_>>) -> R,
) -> R {
    let identities: Vec<_> = states
        .iter()
        .map(|state| {
            (
                state.id.to_string(),
                GlobalStateHolder::new(&state.id.to_string()),
                format!("{}.presence", state.id),
                format!("{}.suppressed", state.id),
                state
                    .fields
                    .iter()
                    .map(|field| format!("{}.{}", state.id, field.name))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let fields: Vec<Vec<_>> = states
        .iter()
        .zip(&identities)
        .map(|(state, (_, _, _, _, objectives))| {
            state
                .fields
                .iter()
                .zip(objectives)
                .map(|(field, objective)| {
                    let descriptor = StateLifecycleDescriptor::new(
                        objective,
                        StateFieldDescriptor::new(
                            &field.name,
                            StateFieldKind::Score,
                            field.default,
                            field.bounds,
                        ),
                    );
                    if field.timer {
                        descriptor.auto_tick()
                    } else {
                        descriptor
                    }
                })
                .collect()
        })
        .collect();
    let data_storage: Vec<_> = states
        .iter()
        .map(|state| {
            state
                .data_fields
                .iter()
                .map(|field| data_identity(state, field))
                .collect::<Vec<_>>()
        })
        .collect();
    let data_fields: Vec<Vec<_>> = states
        .iter()
        .zip(&data_storage)
        .map(|(state, fields)| {
            fields
                .iter()
                .map(|(storage, path, default)| {
                    if state.scope == "global" {
                        StateDataFieldDescriptor::new(storage, path, default)
                    } else {
                        StateDataFieldDescriptor::keyed(storage, path, default)
                    }
                })
                .collect()
        })
        .collect();
    let descriptors = states
        .iter()
        .zip(&identities)
        .zip(&fields)
        .zip(&data_fields)
        .map(
            |(((state, (id, holder, presence, suppression, _)), fields), data_fields)| {
                StateDescriptor::new(
                    id,
                    1,
                    match state.scope.as_str() {
                        "player" => StateScope::Player,
                        "entity" => StateScope::Entity,
                        "global" => StateScope::Global(holder.as_str()),
                        _ => unreachable!("owned State scopes are validated before lowering"),
                    },
                    presence,
                    suppression,
                    fields,
                    &[],
                    data_fields,
                )
            },
        );
    lower(descriptors.collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_data_defaults_share_canonical_storage_lifecycle_without_static_allocations() {
        for scope in ["player", "entity", "global"] {
            let state = State {
                id: "demo:notes".parse().unwrap(),
                scope: scope.into(),
                revision: 1,
                fields: Vec::new(),
                data_fields: vec![crate::compiler::program::model::DataField {
                    name: "text".into(),
                    default: sand_commands::NbtValue::String("$(uuid)".into()),
                }],
            };
            let storage = String::from("demo:state");
            let path = String::from("components.\"notes\".text");
            let default = String::from("\"$(uuid)\"");
            let field = if scope == "global" {
                StateDataFieldDescriptor::new(&storage, &path, &default)
            } else {
                StateDataFieldDescriptor::keyed(&storage, &path, &default)
            };
            let holder = GlobalStateHolder::new("demo:notes");
            let fields = [field];
            let descriptor = StateDescriptor::new(
                "demo:notes",
                1,
                match scope {
                    "player" => StateScope::Player,
                    "entity" => StateScope::Entity,
                    _ => StateScope::Global(holder.as_str()),
                },
                "demo:notes.presence",
                "demo:notes.suppressed",
                &[],
                &[],
                &fields,
            );
            let expected = automatic_lifecycle_with_hooks([descriptor], &BTreeMap::new()).unwrap();
            assert_eq!(lifecycle(&[state]).unwrap(), expected);
        }
    }

    #[test]
    fn owned_state_matches_static_lifecycle_without_static_allocations() {
        let holder = GlobalStateHolder::new("demo:counter");
        for scope in [StateScope::Player, StateScope::Global(holder.as_str())] {
            for default in [i32::MIN, -20, 0, 100, i32::MAX] {
                for bounds in [None, Some((0, 100)), Some((i32::MIN, i32::MAX))] {
                    let fields = [StateLifecycleDescriptor::new(
                        "demo:counter.value",
                        StateFieldDescriptor::new("value", StateFieldKind::Score, default, bounds),
                    )];
                    let descriptor = StateDescriptor::new(
                        "demo:counter",
                        1,
                        scope,
                        "demo:counter.presence",
                        "demo:counter.suppressed",
                        &fields,
                        &[],
                        &[],
                    );
                    let expected =
                        automatic_lifecycle_with_hooks([descriptor], &BTreeMap::new()).unwrap();
                    let owned = State::try_from(descriptor).unwrap();
                    assert_eq!(owned.fields[0].default, default);
                    assert_eq!(owned.fields[0].bounds, bounds);
                    assert_eq!(lifecycle(&[owned]).unwrap(), expected);
                }
            }
        }
    }

    #[test]
    fn multi_field_player_helpers_are_owned_and_do_not_pollute_pending_resources() {
        let state = State {
            data_fields: Vec::new(),
            id: "demo:pair".parse().unwrap(),
            scope: "player".into(),
            revision: 1,
            fields: vec![
                ScoreField {
                    name: "first".into(),
                    default: -7,
                    bounds: None,
                    timer: false,
                },
                ScoreField {
                    name: "second".into(),
                    default: 42,
                    bounds: None,
                    timer: false,
                },
            ],
        };
        assert!(crate::function::drain_dyn_fns().is_empty());
        crate::function::register_dyn_fn_dedup("test/pending", vec!["say pending".to_owned()]);
        let first = lifecycle(std::slice::from_ref(&state)).unwrap();
        assert_eq!(first, lifecycle(&[state]).unwrap());
        assert_eq!(first.player_init_helpers.len(), 1);
        let helper = &first.player_init_helpers[0];
        assert_eq!(helper.owner, "demo:pair");
        assert_eq!(helper.commands.len(), 3);
        assert!(helper.commands[0].ends_with(" -7"));
        assert!(helper.commands[1].ends_with(" 42"));
        assert!(helper.commands[2].ends_with(" 1"));
        let pending = crate::function::drain_dyn_fns();
        assert_eq!(pending.len(), 1);
        assert!(pending[0].0.starts_with("test/pending/"));
    }

    #[test]
    fn owned_counter_provisions_and_initializes_canonical_objectives() {
        let output = lifecycle(&[State {
            data_fields: Vec::new(),
            id: "demo:counter".parse().unwrap(),
            scope: "player".into(),
            revision: 1,
            fields: vec![ScoreField {
                name: "value".to_owned(),
                default: 0,
                bounds: None,
                timer: false,
            }],
        }])
        .unwrap();
        assert_eq!(
            output.load_commands.join("\n"),
            include_str!(
                "../../../tests/fixtures/program/golden/data/demo/function/__sand_lifecycle_load.mcfunction"
            )
        );
        assert_eq!(
            output.player_init_commands.join("\n"),
            include_str!(
                "../../../tests/fixtures/program/golden/data/demo/function/__sand_lifecycle_init.mcfunction"
            )
        );
        assert!(output.player_tick_commands.is_empty());
        assert!(output.global_init_commands.is_empty());
        assert!(output.entity_tick_commands.is_empty());
    }

    #[test]
    fn owned_and_rust_only_states_share_objective_collision_checks() {
        let owned = [State {
            data_fields: Vec::new(),
            id: "demo:counter".parse().unwrap(),
            scope: "player".into(),
            revision: 1,
            fields: vec![ScoreField {
                name: "value".to_owned(),
                default: 0,
                bounds: None,
                timer: false,
            }],
        }];
        const CONFLICTING_FIELDS: &[StateLifecycleDescriptor] = &[StateLifecycleDescriptor::new(
            "demo:counter.value",
            StateFieldDescriptor::new("other", StateFieldKind::Score, 5, None),
        )];
        let rust_only = StateDescriptor::new(
            "demo:other",
            1,
            StateScope::Player,
            "demo:other.presence",
            "demo:other.suppressed",
            CONFLICTING_FIELDS,
            &[],
            &[],
        );
        let error = with_descriptors(&owned, |mut descriptors| {
            descriptors.push(rust_only);
            automatic_lifecycle_with_hooks(descriptors, &BTreeMap::new())
        })
        .unwrap_err();
        assert!(error.contains("demo:counter::value"), "{error}");
        assert!(error.contains("demo:other::other"), "{error}");
    }

    #[test]
    fn unsupported_state_is_not_silently_adapted() {
        let descriptor = StateDescriptor::new(
            "demo:counter",
            2,
            StateScope::Player,
            "demo:counter.presence",
            "demo:counter.suppressed",
            &[],
            &[],
            &[],
        );
        assert!(State::try_from(descriptor).is_err());
    }
}
