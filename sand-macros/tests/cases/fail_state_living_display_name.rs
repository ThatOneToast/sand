use sand::prelude::*;

#[derive(State)]
#[state(namespace = "test", scope = living)]
struct LivingState {
    #[state(display_name = "Visible name")]
    score: EntityScore<i32>,
}

fn main() {}
