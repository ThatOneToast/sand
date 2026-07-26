use sand::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EntityStateEnum)]
enum Phase {
    Dormant = -1,
    Active,
    Enraged = 7,
}

#[derive(EntityState)]
#[entity_state(namespace = "rpg", name = "mob/zombie", version = 3)]
struct MobState {
    #[state(default = 10, min = 1, max = 100)]
    level: EntityScore<i32>,
    #[state(default = true)]
    sick: EntityFlag,
    #[state(default = "Phase::Dormant")]
    phase: EntityEnum<Phase>,
    automatic_phase: EntityEnum<Phase>,
    #[state(default = 20)]
    age: EntityTimer,
    #[state(default = 0)]
    ability: EntityCooldown,
    #[state(default = 3, kind = "version")]
    schema_version: EntityScore<i32>,
    #[state(kind = "dirty")]
    stats_dirty: EntityScore<i32>,
}

fn main() {
    let schema = MobState::schema();
    assert_eq!(schema.id(), "rpg:mob/zombie");
    assert_eq!(schema.version, 3);
    assert_eq!(schema.fields, MobState::FIELDS);
    assert_eq!(MobState::level.descriptor().bounds, Some((1, 100)));
    assert_eq!(MobState::phase.descriptor().default, -1);
    assert_eq!(MobState::automatic_phase.descriptor().default, -1);
    assert_eq!(Phase::Active.encode(), 0);
    assert_eq!(Phase::Enraged.encode(), 7);
}
