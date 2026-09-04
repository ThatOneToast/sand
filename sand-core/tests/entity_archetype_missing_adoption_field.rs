use sand::prelude::*;

#[derive(State)]
#[state(namespace = "cf_missing_adoption", scope = living)]
#[allow(dead_code)]
struct Attached {
    value: Score,
}

#[derive(State)]
#[state(namespace = "cf_missing_adoption", scope = living)]
#[allow(dead_code)]
struct Unrelated {
    value: Score,
}

#[entity_archetype]
fn missing_adoption_field() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("cf_missing_adoption:seeker".parse().unwrap())
        .components::<Attached>()
        .adopt(
            Adoption::natural()
                .where_state(Unrelated::value.matches(1..).expect("valid score range")),
        )
}

#[test]
fn unattached_adoption_field_uses_component_membership() {
    let message = sand_core::try_export_components_json("cf_missing_adoption")
        .unwrap_err()
        .to_string();
    assert!(message.contains("cf_missing_adoption:seeker"));
    assert!(message.contains("adoption predicate"));
    assert!(message.contains("cf_missing_adoption:unrelated"));
    assert!(message.contains("value"));
}
