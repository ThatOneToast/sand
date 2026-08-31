use sand::prelude::*;

#[derive(State)]
#[state(namespace = "demo", scope = entity)]
struct InvalidEntityData {
    #[state(default = 0)]
    payload: Data<i32>,
}

fn main() {}
