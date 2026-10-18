//! Owned adaptation of the supported player/global score State slice.
//!
//! Strings are borrowed only while canonical lifecycle generation runs. No
//! descriptor is interned, leaked or registered in link-time inventory.

use std::collections::BTreeMap;

use super::GlobalStateHolder;
use super::registry::{AutomaticLifecycle, automatic_lifecycle_with_hooks};
use super::registry::{StateDescriptor, StateLifecycleDescriptor, StateScope};
use crate::entity::{StateFieldDescriptor, StateFieldKind};

use crate::compiler::program::model::{ScoreField, State};

impl TryFrom<StateDescriptor<'_>> for State {
    type Error = String;

    fn try_from(descriptor: StateDescriptor<'_>) -> Result<Self, Self::Error> {
        let scope = match descriptor.scope {
            StateScope::Player => "player",
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
                "State `{}` is outside the player/global score v1 slice",
                descriptor.id
            ));
        }
        for field in descriptor.fields {
            if field.field.kind != StateFieldKind::Score
                || field.criterion != "dummy"
                || field.display_name.is_some()
                || field.auto_tick
                || field.objective != format!("{}.{}", descriptor.id, field.field.name)
            {
                return Err(format!(
                    "State field `{}.{}` is outside the player/global score v1 slice",
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
                })
                .collect(),
        })
    }
}

/// Resolve the scoreboard owner from the same identity used by lifecycle lowering.
pub(crate) fn holder(state: &State) -> sand_commands::ScoreHolder {
    match state.scope.as_str() {
        "player" => sand_commands::ScoreHolder::self_(),
        "global" => {
            sand_commands::ScoreHolder::fake(GlobalStateHolder::new(&state.id.to_string()).as_str())
        }
        _ => unreachable!("owned State scopes are validated before lowering"),
    }
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
                    StateLifecycleDescriptor::new(
                        objective,
                        StateFieldDescriptor::new(
                            &field.name,
                            StateFieldKind::Score,
                            field.default,
                            field.bounds,
                        ),
                    )
                })
                .collect()
        })
        .collect();
    let descriptors = states.iter().zip(&identities).zip(&fields).map(
        |((state, (id, holder, presence, suppression, _)), fields)| {
            StateDescriptor::new(
                id,
                1,
                match state.scope.as_str() {
                    "player" => StateScope::Player,
                    "global" => StateScope::Global(holder.as_str()),
                    _ => unreachable!("owned State scopes are validated before lowering"),
                },
                presence,
                suppression,
                fields,
                &[],
                &[],
            )
        },
    );
    lower(descriptors.collect())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn owned_counter_provisions_and_initializes_canonical_objectives() {
        let output = lifecycle(&[State {
            id: "demo:counter".parse().unwrap(),
            scope: "player".into(),
            revision: 1,
            fields: vec![ScoreField {
                name: "value".to_owned(),
                default: 0,
                bounds: None,
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
            id: "demo:counter".parse().unwrap(),
            scope: "player".into(),
            revision: 1,
            fields: vec![ScoreField {
                name: "value".to_owned(),
                default: 0,
                bounds: None,
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
