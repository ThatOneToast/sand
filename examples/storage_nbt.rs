//! Typed storage and NBT paths.

use sand_core::prelude::*;
use sand_macros::{component, function};

static PLAYER_MANA: StorageVar<i32> = StorageVar::new("example:data", "players.self.mana");

#[component(Load)]
pub fn load_storage() {
    mcfunction! {
        PLAYER_MANA.set_int(100);
        PLAYER_MANA.as_path().key("regen").set_bool(true);
    }
}

#[function]
pub fn show_storage_state() {
    mcfunction! {
        TypedExecute::as_players()
            .when(PLAYER_MANA.exists())
            .run(Actionbar::show(Selector::self_(), Text::new("Storage ready").green()));
    }
}
