//! Basic typed Sand functions.

use sand_core::prelude::*;
use sand_macros::{datapack_component, function};

static TICK_COUNT: ScoreVar<i32> = ScoreVar::new("tick_count");

#[datapack_component(Load)]
pub fn load() {
    TICK_COUNT.define();
    cmd::tellraw(Target::players(), Text::new("Datapack loaded").green());
}

#[datapack_component(Tick)]
pub fn tick() {
    TICK_COUNT.add(Target::players(), 1);
}

#[function]
pub fn greet() {
    cmd::tellraw(
        Target::players(),
        Text::new("Hello from Sand").gold().bold(true),
    );
    Actionbar::show(
        Target::players(),
        Text::new("Typed commands, typed output").aqua(),
    );
}
