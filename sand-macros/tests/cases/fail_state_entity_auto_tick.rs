use sand::prelude::*;

#[derive(State)]
#[state(namespace = "test", scope = entity)]
struct EntityState {
    #[state(auto_tick)]
    timer: EntityTimer,
}

fn main() {}
