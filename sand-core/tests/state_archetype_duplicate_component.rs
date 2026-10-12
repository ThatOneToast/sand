#[path = "support/actions.rs"]
mod actions;
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "duplicate_component", scope = living)]
#[allow(dead_code)]
struct PrimaryState {
    value: Score,
}

#[derive(StateBundle)]
#[allow(dead_code)]
struct RepeatsPrimary {
    primary: PrimaryState,
}

#[derive(State)]
#[state(namespace = "duplicate_component", scope = living)]
#[allow(dead_code)]
struct SharedState {
    shared: Score,
}

#[derive(State)]
#[state(namespace = "duplicate_component", scope = living)]
#[allow(dead_code)]
struct FirstOnly {
    value: Score,
}

#[derive(State)]
#[state(namespace = "duplicate_component", scope = living)]
#[allow(dead_code)]
struct SecondOnly {
    value: Score,
}

#[derive(Archetype)]
#[archetype(id = "duplicate_component:duplicate_composition", entity = ZombieKind, configure = Self::configure)]
#[allow(dead_code)]
struct DuplicateComposition {
    repeats_primary: RepeatsPrimary,
}

impl DuplicateComposition {
    fn configure(archetype: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        archetype
    }
}

#[derive(Archetype)]
#[archetype(id = "duplicate_component:first", entity = ZombieKind, configure = Self::configure)]
#[allow(dead_code)]
struct FirstSharedArchetype {
    shared_state: SharedState,
    first_only: FirstOnly,
}

impl FirstSharedArchetype {
    fn configure(archetype: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        archetype
    }
}

#[derive(Archetype)]
#[archetype(id = "duplicate_component:second", entity = ZombieKind, configure = Self::configure)]
#[allow(dead_code)]
struct SecondSharedArchetype {
    shared_state: SharedState,
    second_only: SecondOnly,
}

impl SecondSharedArchetype {
    fn configure(archetype: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        archetype
    }
}

#[test]
fn nested_component_composition_exports_deterministically() {
    let first = sand_core::try_export_components_json("duplicate_component").unwrap();
    let second = sand_core::try_export_components_json("duplicate_component").unwrap();
    assert_eq!(first, second);
    assert_eq!(
        DuplicateComposition::summon(Vec3::here())
            .iter()
            .filter(|command| command.contains("summon minecraft:zombie"))
            .count(),
        1
    );
}

#[test]
fn shared_components_remain_reusable_and_cleanup_is_composition_scoped() {
    let json = sand_core::try_export_components_json("duplicate_component").unwrap();
    let records: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    let functions = |suffix: &str| {
        records
            .iter()
            .filter(|record| {
                record["path"]
                    .as_str()
                    .is_some_and(|path| path.ends_with(suffix))
            })
            .filter_map(|record| record["content"].as_str())
            .collect::<Vec<_>>()
            .join("\n")
    };

    let provisions = functions("/provision");
    assert_eq!(
        provisions.matches(&SharedState::shared.objective()).count(),
        4
    );

    let first_cleanup = records
        .iter()
        .find(|record| {
            record["path"]
                .as_str()
                .is_some_and(|path| path.ends_with("/cleanup"))
                && record["content"]
                    .as_str()
                    .is_some_and(|content| content.contains(&FirstOnly::value.objective()))
        })
        .and_then(|record| record["content"].as_str())
        .expect("first archetype cleanup");
    assert!(first_cleanup.contains(&SharedState::shared.objective()));
    assert!(first_cleanup.contains(&FirstOnly::value.objective()));
    assert!(!first_cleanup.contains(&SecondOnly::value.objective()));
    let component_dirty = actions::emitted(SharedState::shared.bind().set(1))
        .into_iter()
        .last()
        .and_then(|command| command.split_whitespace().nth(4).map(str::to_owned))
        .expect("shared component dirty objective");
    assert!(first_cleanup.lines().any(|line| {
        line.contains("unless entity @s[tag=")
            && line.ends_with(&format!("scoreboard players reset @s {component_dirty}"))
    }));
    assert!(
        !first_cleanup
            .lines()
            .any(|line| line == format!("scoreboard players reset @s {component_dirty}"))
    );
}
