use sand::prelude::*;

#[derive(EntityState)]
#[entity_state(namespace = "RPG!", name = "bad", version = 1)]
struct Bad {
    value: EntityScore<i32>,
}

fn main() {}
