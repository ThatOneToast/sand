use sand::prelude::*;

#[derive(State)]
#[state(namespace = "rpg", scope = entity, name = "bad", version = 1)]
struct Bad {
    #[state(min = 10, max = 1)]
    value: EntityScore<i32>,
}

fn main() {}
