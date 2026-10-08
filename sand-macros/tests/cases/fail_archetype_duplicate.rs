use sand::prelude::*;
#[derive(State)]
#[state(namespace = "test", scope = living)]
struct Health { value: Score }
#[derive(StateBundle)]
struct Nested { health: Health }
#[derive(Archetype)]
#[archetype(id = "test:bad", entity = Zombie)]
struct Bad { direct: Health, nested: Nested }
fn main() {}
