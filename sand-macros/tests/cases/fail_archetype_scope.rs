use sand::prelude::*;
#[derive(State)]
#[state(namespace = "test", scope = player)]
struct Mana { value: Score }
#[derive(Archetype)]
#[archetype(id = "test:bad", entity = Zombie)]
struct Bad { mana: Mana }
fn main() {}
