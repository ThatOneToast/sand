use sand::prelude::*;

#[derive(State)]
#[state(namespace = "demo", scope = player)]
struct PlayerState;

#[derive(State)]
#[state(namespace = "demo", scope = living)]
struct LivingState;

#[derive(StateBundle)]
struct InvalidBundle {
    player: PlayerState,
    living: LivingState,
}

fn main() {}
