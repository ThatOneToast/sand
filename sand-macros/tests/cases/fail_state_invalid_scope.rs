use sand::prelude::*;

#[derive(State)]
#[state(namespace = "test", scope = players)]
struct Bad {
    value: EntityScore<i32>,
}

fn main() {}
