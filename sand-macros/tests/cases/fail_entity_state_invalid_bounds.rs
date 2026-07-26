use sand::prelude::*;

#[derive(EntityState)]
#[entity_state(namespace = "rpg", name = "bad", version = 1)]
struct Bad {
    #[state(min = 10, max = 1)]
    value: EntityScore<i32>,
}

fn main() {}
