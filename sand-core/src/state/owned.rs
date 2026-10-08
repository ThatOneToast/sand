//! Owned adaptation of the supported player-score State slice.
//!
//! Strings are borrowed only while canonical lifecycle generation runs. No
//! descriptor is interned, leaked or registered in link-time inventory.

use std::collections::BTreeMap;

use super::registry::{AutomaticLifecycle, automatic_lifecycle_with_hooks};
use super::registry::{StateDescriptor, StateLifecycleDescriptor, StateScope};
use crate::entity::{StateFieldDescriptor, StateFieldKind};

use crate::compiler::program::model::{ScoreField, State};

impl TryFrom<StateDescriptor<'_>> for State {
    type Error = String;

    fn try_from(descriptor: StateDescriptor<'_>) -> Result<Self, Self::Error> {
        if descriptor.scope != StateScope::Player
            || descriptor.version != 1
            || !descriptor.data_fields.is_empty()
            || !descriptor.migrations.is_empty()
        {
            return Err(format!(
                "State `{}` is outside the player-score v1 slice",
                descriptor.id
            ));
        }
        for field in descriptor.fields {
            if field.field.kind != StateFieldKind::Score
                || field.field.default != 0
                || field.field.bounds.is_some()
                || field.criterion != "dummy"
                || field.display_name.is_some()
                || field.auto_tick
                || field.objective != format!("{}.{}", descriptor.id, field.field.name)
            {
                return Err(format!(
                    "State field `{}.{}` is outside the player-score v1 slice",
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
            scope: "player".into(),
            revision: 1,
            fields: descriptor
                .fields
                .iter()
                .map(|field| ScoreField {
                    name: field.field.name.to_owned(),
                    default: 0,
                })
                .collect(),
        })
    }
}

/// Lower explicit owned inputs through the same lifecycle implementation used
/// by Rust descriptors, without collecting or executing Rust hooks.
pub(crate) fn player_lifecycle(states: &[State]) -> Result<AutomaticLifecycle, String> {
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
        .map(|(state, (_, _, _, objectives))| {
            state
                .fields
                .iter()
                .zip(objectives)
                .map(|(field, objective)| {
                    StateLifecycleDescriptor::new(
                        objective,
                        StateFieldDescriptor::new(&field.name, StateFieldKind::Score, 0, None),
                    )
                })
                .collect()
        })
        .collect();
    let descriptors = states.iter().zip(&identities).zip(&fields).map(
        |((_state, (id, presence, suppression, _)), fields)| {
            StateDescriptor::new(
                id,
                1,
                StateScope::Player,
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
        let fields = [StateLifecycleDescriptor::new(
            "demo:counter.value",
            StateFieldDescriptor::new("value", StateFieldKind::Score, 0, None),
        )];
        let descriptor = StateDescriptor::new(
            "demo:counter",
            1,
            StateScope::Player,
            "demo:counter.presence",
            "demo:counter.suppressed",
            &fields,
            &[],
            &[],
        );
        let expected = automatic_lifecycle_with_hooks([descriptor], &BTreeMap::new()).unwrap();
        let owned = State::try_from(descriptor).unwrap();
        assert_eq!(player_lifecycle(&[owned]).unwrap(), expected);
    }

    #[test]
    fn owned_counter_provisions_and_initializes_canonical_objectives() {
        let output = player_lifecycle(&[State {
            id: "demo:counter".parse().unwrap(),
            scope: "player".into(),
            revision: 1,
            fields: vec![ScoreField {
                name: "value".to_owned(),
                default: 0,
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
