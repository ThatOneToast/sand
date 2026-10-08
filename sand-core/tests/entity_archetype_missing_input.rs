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

#[derive(Archetype)]
#[archetype(id = "cf_missing_input:seeker", entity = ZombieKind, configure = Self::configure)]
#[allow(dead_code)]
struct MissingCurveInput {
    combat: Combat,
}

impl MissingCurveInput {
    fn configure(archetype: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        archetype.derive(Combat::maximum, StatCurve::state(Unrelated::value))
    }
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
