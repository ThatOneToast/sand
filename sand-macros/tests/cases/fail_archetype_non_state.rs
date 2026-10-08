use sand::prelude::*;
#[derive(Archetype)]
#[archetype(id = "test:bad", entity = Zombie)]
struct Bad { value: i32 }
fn main() {}
