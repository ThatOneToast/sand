use sand::prelude::*;

#[derive(State)]
#[state(namespace = "cf_cycle", scope = living)]
#[allow(dead_code)]
struct Progression {
    level: Score,
}

#[derive(State)]
#[state(namespace = "cf_cycle", scope = living)]
#[allow(dead_code)]
struct Scaling {
    power: Score,
}

#[entity_archetype]
fn cross_component_cycle() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("cf_cycle:cycle".parse().unwrap())
        .components::<Progression>()
        .components::<Scaling>()
        .derive(Progression::level, StatCurve::state(Scaling::power))
        .derive(Scaling::power, StatCurve::state(Progression::level))
}

#[test]
fn cross_component_cycles_are_rejected() {
    let error =
        sand_core::try_export_components_json("cf_cycle").expect_err("cycle should fail export");
    let message = error.to_string();
    assert!(message.contains("cycle"));
    assert!(message.contains(&Progression::level.objective()));
    assert!(message.contains(&Scaling::power.objective()));
}
