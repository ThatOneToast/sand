use sand::entity::{Adoption, EntityArchetype};
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "rpg", scope = entity, name = "zombie", version = 1)]
struct ZombieState {
    #[state(default = 1, min = 1, max = 100)]
    level: EntityScore<i32>,
}

#[derive(Archetype)]
#[archetype(id = "rpg:zombie", entity = ZombieKind, configure = Self::configure)]
#[allow(dead_code)]
struct Zombie {
    zombie_state: ZombieState,
}

impl Zombie {
    fn configure(archetype: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
    archetype
        .adopt(Adoption::natural_and_external())
    }
}

fn main() {
    assert!(Zombie::summon(Vec3::here())[0].contains("summon minecraft:zombie"));
}
