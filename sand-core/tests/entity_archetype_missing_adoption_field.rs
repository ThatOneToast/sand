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

#[derive(Archetype)]
#[archetype(id = "cf_missing_adoption:seeker", entity = ZombieKind, configure = Self::configure)]
#[allow(dead_code)]
struct MissingAdoptionField {
    attached: Attached,
}

impl MissingAdoptionField {
    fn configure(archetype: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        archetype.adopt(
            Adoption::natural()
                .where_state(Unrelated::value.matches(1..).expect("valid score range")),
        )
    }
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
