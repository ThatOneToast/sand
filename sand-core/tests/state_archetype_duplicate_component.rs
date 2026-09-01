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

#[entity_archetype]
fn invalid_composition() -> EntityArchetype<ZombieKind, PrimaryState> {
    EntityArchetype::new(
        ResourceLocation::new("duplicate_component", "invalid_composition").unwrap(),
    )
    .components::<RepeatsPrimary>()
}

#[test]
fn duplicate_primary_component_is_an_actionable_export_error() {
    let error = sand_core::try_export_components_json("duplicate_component")
        .expect_err("repeating the primary component must not duplicate lifecycle work");
    let message = error.to_string();
    assert!(
        message.contains("repeats primary State component"),
        "{message}"
    );
    assert!(
        message.contains("duplicate_component:primary_state"),
        "{message}"
    );
}
