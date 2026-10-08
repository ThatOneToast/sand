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

#[derive(Archetype)]
#[archetype(id = "cf_cycle:cycle", entity = ZombieKind, configure = Self::configure)]
#[allow(dead_code)]
struct CrossComponentCycle {
    progression: Progression,
    scaling: Scaling,
}

impl CrossComponentCycle {
    fn configure(archetype: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        archetype
            .derive(Progression::level, StatCurve::state(Scaling::power))
            .derive(Scaling::power, StatCurve::state(Progression::level))
    }
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
