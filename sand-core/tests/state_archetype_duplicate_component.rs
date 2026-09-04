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

#[entity_archetype]
fn duplicate_composition() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new(
        ResourceLocation::new("duplicate_component", "duplicate_composition").unwrap(),
    )
    .components::<PrimaryState>()
    .components::<RepeatsPrimary>()
}

#[entity_archetype]
fn first_shared_archetype() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("duplicate_component:first".parse().unwrap())
        .components::<SharedState>()
        .components::<FirstOnly>()
}

#[entity_archetype]
fn second_shared_archetype() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("duplicate_component:second".parse().unwrap())
        .components::<SharedState>()
        .components::<SecondOnly>()
}

#[test]
fn duplicate_component_and_bundle_flatten_deterministically() {
    let first = sand_core::try_export_components_json("duplicate_component").unwrap();
    let second = sand_core::try_export_components_json("duplicate_component").unwrap();
    assert_eq!(first, second);
    assert_eq!(
        duplicate_composition()
            .summon()
            .iter()
            .filter(|command| command.contains("execute summon"))
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
}
