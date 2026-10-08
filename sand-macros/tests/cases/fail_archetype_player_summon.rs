use sand::prelude::*;
#[derive(Archetype)]
#[archetype(id = "test:hero", entity = Player)]
struct Hero;
fn main() { let _ = Hero::summon(Vec3::here()); }
