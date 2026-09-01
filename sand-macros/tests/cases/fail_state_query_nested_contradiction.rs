use sand::prelude::*;

#[derive(State)]
#[state(namespace = "demo", scope = living)]
struct Attack;

#[derive(State)]
#[state(namespace = "demo", scope = living)]
struct Defense;

#[derive(StateBundle)]
struct Combat {
    attack: Attack,
    defense: Defense,
}

#[derive(StateQuery)]
#[query(scope = living)]
struct Impossible {
    #[require]
    combat: Combat,
    #[without]
    attack: Attack,
}

fn main() {}
