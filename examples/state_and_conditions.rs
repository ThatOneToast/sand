//! Typed state and nested conditions.

use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static CASTING: Flag = Flag::new("casting");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

#[component(Load)]
pub fn load_state() {
    mcfunction! {
        MANA.define();
        CASTING.define();
        DASH.define();
        MANA.set(Selector::all_players(), 100);
        CASTING.disable(Selector::all_players());
    }
}

#[component(Tick)]
pub fn tick_state() {
    mcfunction! {
        DASH.tick(Selector::all_players());
    }
}

#[function]
pub fn try_dash() {
    mcfunction! {
        TypedExecute::as_players_at_self()
            .when(all![
                MANA.of("@s").gte(25),
                CASTING.of("@s").is_false(),
                any![DASH.ready("@s"), Condition::predicate("example:dash_override")],
            ])
            .run(Actionbar::show(Selector::self_(), Text::new("Dash ready").aqua()));
    }
}
