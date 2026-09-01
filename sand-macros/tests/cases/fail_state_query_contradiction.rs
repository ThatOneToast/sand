use sand::prelude::*;

#[derive(State)]
#[state(namespace = "demo", scope = entity)]
struct Attached;

#[derive(StateQuery)]
struct Impossible {
    #[require]
    required: Attached,
    #[without]
    forbidden: Attached,
}

fn main() {}
