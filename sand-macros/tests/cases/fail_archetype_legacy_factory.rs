use sand::prelude::*;
#[sand::entity_archetype]
fn legacy() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("test:legacy".parse().unwrap())
}
fn main() {}
