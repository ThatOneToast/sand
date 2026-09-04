use sand::prelude::*;

#[derive(State)]
#[state(namespace = "cf_missing_input", scope = living)]
#[allow(dead_code)]
struct Combat {
    maximum: Score,
}

#[derive(State)]
#[state(namespace = "cf_missing_input", scope = living)]
#[allow(dead_code)]
struct Unrelated {
    value: Score,
}

#[entity_archetype]
fn missing_curve_input() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("cf_missing_input:seeker".parse().unwrap())
        .components::<Combat>()
        .derive(Combat::maximum, StatCurve::state(Unrelated::value))
}

#[test]
fn unattached_curve_input_is_actionable() {
    let message = sand_core::try_export_components_json("cf_missing_input")
        .unwrap_err()
        .to_string();
    assert!(message.contains("cf_missing_input:seeker"));
    assert!(message.contains("input"));
    assert!(message.contains("cf_missing_input:unrelated"));
    assert!(message.contains("value"));
}
