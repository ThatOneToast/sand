use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

#[component(Load)]
pub fn load() {
    MANA.define();
}

#[function]
pub fn reward() {
    MANA.add(Selector::self_(), 10);
    cmd::tellraw(Selector::self_(), Text::new("+10 mana").aqua());
}

fn main() {
    let _ = load();
    let _ = reward();
}
