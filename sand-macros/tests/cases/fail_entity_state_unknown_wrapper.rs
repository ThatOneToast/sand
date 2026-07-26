use sand::prelude::*;

#[derive(EntityState)]
#[entity_state(namespace = "rpg", name = "bad", version = 1)]
struct Bad {
    value: String,
}

fn main() {}
