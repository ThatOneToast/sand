use sand_core::prelude::*;
use sand_macros::{datapack_component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

#[datapack_component(Load)]
pub fn load() {
    MANA.define();
}

#[function]
pub fn reward() {
    MANA.add(Target::self_(), 10);
    cmd::tellraw(Target::self_(), Text::new("+10 mana").aqua());
}

fn main() {
    let _ = load();
    let _ = reward();
}
