use sand::prelude::*;

#[derive(State)]
#[state(namespace = "demo", scope = entity)]
struct EntityState;

#[derive(StateQuery)]
#[query(scope = player)]
struct PlayersOnly {
    state: EntityState,
}

fn main() {}
