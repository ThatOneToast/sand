use sand::entity::{Adoption, EntityArchetype};
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "rpg", scope = entity, name = "zombie", version = 1)]
struct ZombieState {
    #[state(default = 1, min = 1, max = 100)]
    level: EntityScore<i32>,
}

#[entity_archetype]
fn zombie() -> EntityArchetype<ZombieKind, ZombieState> {
    EntityArchetype::new(ResourceLocation::new("rpg", "zombie").unwrap())
        .adopt(Adoption::natural_and_external())
}

fn main() {
    assert_eq!(zombie().id().to_string(), "rpg:zombie");
}
