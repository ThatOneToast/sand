use sand::prelude::*;

#[derive(State)]
#[state(namespace = "cf_missing_target", scope = living)]
#[allow(dead_code)]
struct Progression {
    level: Score,
}

#[derive(State)]
#[state(namespace = "cf_missing_target", scope = living)]
#[allow(dead_code)]
struct Unrelated {
    value: Score,
}

#[entity_archetype]
fn missing_target() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("cf_missing_target:seeker".parse().unwrap())
        .components::<Progression>()
        .derive(Unrelated::value, StatCurve::state(Progression::level))
}

#[test]
fn unattached_derivation_target_is_actionable() {
    let message = sand_core::try_export_components_json("cf_missing_target")
        .unwrap_err()
        .to_string();
    assert!(message.contains("cf_missing_target:seeker"));
    assert!(message.contains("derivation"));
    assert!(message.contains("cf_missing_target:unrelated"));
    assert!(message.contains("value"));
}
