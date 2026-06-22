//! A compact typed spell-system example.

use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static FIREBALL: Cooldown = Cooldown::new("fireball", Ticks::seconds(5));
static PLAYER_DATA: StorageVar<i32> = StorageVar::new("spells:data", "player.mana");

#[component(Load)]
pub fn load_spells() {
    mcfunction! {
        MANA.define();
        FIREBALL.define();
        MANA.set(Selector::all_players(), 100);
        PLAYER_DATA.set_int(100);
    }
}

#[component(Tick)]
pub fn tick_spells() {
    mcfunction! {
        FIREBALL.tick(Selector::all_players());
    }
}

#[function]
pub fn cast_fireball() {
    mcfunction! {
        TypedExecute::as_players_at_self()
            .when(all![MANA.of("@s").gte(20), FIREBALL.ready("@s")])
            .run(cmd::function(ResourceLocation::new("spells", "fireball/do_cast").unwrap()));
    }
}

#[function]
pub fn show_spell_hint() {
    mcfunction! {
        TypedExecute::as_players()
            .when(any![FIREBALL.ready("@s"), PLAYER_DATA.exists()])
            .run(Actionbar::show(Selector::self_(), Text::new("Fireball ready").gold()));
    }
}
