use sand::prelude::*;

#[derive(State)]
#[state(namespace = "test", scope = player)]
struct Bad {
    #[state(auto_tick)]
    value: EntityScore<i32>,
}

fn main() {}
