//! Basic typed Sand functions and derived State.

use sand::prelude::*;

#[derive(State)]
#[state(namespace = "example", scope = global)]
struct PackState {
    #[state(default = 0)]
    ticks: Score,
}

#[function]
pub fn tick() {
    PackState::global().ticks.add(1);
}

#[function]
pub fn greet() {
    cmd::tellraw(
        Target::players(),
        Text::new("Hello from Sand").gold().bold(true),
    );
}
