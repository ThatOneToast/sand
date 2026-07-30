use sand::prelude::*;

#[derive(State)]
#[state(namespace = "rpg", scope = entity, name = "bad", version = 1)]
struct Bad(EntityScore<i32>);

fn main() {}
