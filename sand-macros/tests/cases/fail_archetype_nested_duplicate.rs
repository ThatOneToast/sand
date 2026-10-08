use sand::prelude::*;
#[derive(State)]
#[state(namespace = "test", scope = living)]
struct Health { value: Score }
#[derive(StateBundle)]
struct Nested { first: Health, second: Health }
#[derive(Archetype)]
#[archetype(id = "test:bad", entity = Zombie)]
struct Bad { nested: Nested }
fn main() {}
