//! Shared dirty generations must survive either composition's acknowledgement.
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "dirty_shared", scope = living)]
pub struct Progression {
    #[state(default = 1)]
    pub level: Score,
}

#[derive(Archetype)]
#[archetype(id = "dirty_shared:first", entity = Zombie, configure = Self::configure)]
pub struct First {
    pub progression: Progression,
}
impl First {
    fn configure(base: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        base.adopt(Adoption::external())
            .attribute(AttributeBinding::new(
                sand_components::AttributeType::AttackDamage,
                NumericPropertySource::state(Progression::level),
            ))
            .on(
                EntityTransition::threshold(Progression::level, 10, ThresholdDirection::Rising),
                EntityAction::Run("dirty_shared:callback".parse().unwrap()),
            )
    }
}

#[derive(Archetype)]
#[archetype(id = "dirty_shared:second", entity = Zombie, configure = Self::configure)]
pub struct Second {
    pub progression: Progression,
}
impl Second {
    fn configure(base: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        base.adopt(Adoption::external())
            .attribute(AttributeBinding::new(
                sand_components::AttributeType::Armor,
                NumericPropertySource::state(Progression::level),
            ))
    }
}

#[function("dirty_shared:callback")]
fn callback() {
    let entity = First::on(EntityContext::<ZombieKind>::default());
    entity.progression.level.set(11);
}

fn export(namespace: &str) -> String {
    sand_core::try_export_components_json(namespace).unwrap()
}

#[test]
fn bookkeeping_exports_are_deterministic_under_concurrency() {
    let expected = export("dirty_shared");
    for worker in (0..4).map(|_| std::thread::spawn(|| export("dirty_shared"))) {
        assert_eq!(worker.join().unwrap(), expected);
    }
}

#[test]
#[ignore = "external live-server fixture export"]
fn dump_dirty_bookkeeping_pack() {
    std::fs::write(
        std::env::var("SAND_DIRTY_DUMP_PATH").expect("dump path"),
        serde_json::to_string(&serde_json::json!({
            "records": serde_json::from_str::<serde_json::Value>(&export("dirty_shared")).unwrap(),
            "level": Progression::level.objective(),
            "dirty": Progression::level.dirty_objective(),
        }))
        .unwrap(),
    )
    .unwrap();
}
