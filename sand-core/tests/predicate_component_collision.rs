use sand::prelude::*;

fn definition() -> Predicate {
    Predicate::new(
        PredicateId::custom(ResourceLocation::new("__sand_local", "same").unwrap()),
        PredicateRoot::random_chance(0.5),
    )
}

#[datapack_component]
fn first_owner() -> Predicate {
    let retained = definition();
    let _reference = Condition::predicate(&retained);
    retained
}

#[datapack_component]
fn second_owner() -> Predicate {
    definition()
}

#[test]
fn references_do_not_bypass_registration_collision_validation() {
    let error = sand_core::try_export_components("predicate_collision")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("predicate_collision:predicate/same.json"),
        "{error}"
    );
    assert!(error.contains("first_owner"), "{error}");
    assert!(error.contains("second_owner"), "{error}");
}
