use sand::prelude::*;

mod hidden {
    use super::*;

    #[derive(State)]
    #[state(namespace = "test", scope = player)]
    struct PrivateState {
        score: EntityScore<i32>,
    }
}

fn main() {
    let _: hidden::PrivateStateBound;
}
