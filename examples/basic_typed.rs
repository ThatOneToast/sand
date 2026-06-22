//! Basic typed Sand functions.

use sand_core::prelude::*;
use sand_macros::{component, function};

static TICK_COUNT: ScoreVar<i32> = ScoreVar::new("tick_count");

#[component(Load)]
pub fn load() {
    TICK_COUNT.define();
    cmd::tellraw(Selector::all_players(), Text::new("Datapack loaded").green());
}

#[component(Tick)]
pub fn tick() {
    TICK_COUNT.add(Selector::all_players(), 1);
}

#[function]
pub fn greet() {
    cmd::tellraw(
        Selector::all_players(),
        Text::new("Hello from Sand").gold().bold(true),
    );
    Actionbar::show(
        Selector::all_players(),
        Text::new("Typed commands, typed output").aqua(),
    );
}
