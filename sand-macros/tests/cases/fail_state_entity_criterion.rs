use sand::prelude::*;

#[derive(State)]
#[state(namespace = "test", scope = entity)]
struct EntityState {
    #[state(criterion = "playerKillCount")]
    score: EntityScore<i32>,
}

fn main() {}
