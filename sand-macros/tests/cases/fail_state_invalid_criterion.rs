use sand::prelude::*;

#[derive(State)]
#[state(namespace = "test", scope = player)]
struct InvalidCriterionState {
    #[state(criterion = "not a criterion")]
    value: EntityScore<i32>,
}

fn main() {}
