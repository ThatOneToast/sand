//! Native configuration must preserve declaration ownership and use composed State.
use sand::prelude::*;
use std::cell::Cell;
thread_local! { static MODE: Cell<u8> = const { Cell::new(0) }; }
#[derive(State)]
#[state(namespace = "decl", scope = living)]
pub struct Combat {
    pub health: Score,
}
#[derive(State)]
#[state(namespace = "decl", scope = living)]
pub struct Other {
    pub health: Score,
}
#[derive(Archetype)]
#[archetype(id = "decl:seeker", entity = Zombie, configure = Self::configure)]
pub struct Seeker {
    pub combat: Combat,
}
impl Seeker {
    fn configure(base: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        match MODE.get() {
            1 => base.health(HealthBinding::new(Other::health)),
            2 => EntityArchetype::new("decl:changed".parse().unwrap()).components::<Combat>(),
            3 => base.components::<Other>(),
            4 => base.derive(Combat::health, StatCurve::state(Other::health)),
            _ => base,
        }
    }
}
#[test]
fn configuration_uses_canonical_membership_and_retains_the_declaration() {
    let expected = sand::advanced::try_export_components_json("decl", "26.2").unwrap();
    for (mode, diagnostic) in [
        (1, "SAND-ENTITY-COMPONENT"),
        (2, "SAND-ARCHETYPE-DECLARATION"),
        (3, "SAND-ARCHETYPE-DECLARATION"),
        (4, "SAND-ENTITY-COMPONENT"),
    ] {
        MODE.set(mode);
        let error = sand::advanced::try_export_components_json("decl", "26.2")
            .unwrap_err()
            .to_string();
        assert!(error.contains(diagnostic), "{error}");
        assert!(error.contains("decl:seeker"), "{error}");
        MODE.set(0);
        assert_eq!(
            sand::advanced::try_export_components_json("decl", "26.2").unwrap(),
            expected
        );
    }
}
