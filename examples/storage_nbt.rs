//! Canonical NBT roots, paths, and typed references.

use sand::prelude::*;

#[function]
pub fn initialize_storage() {
    let mana = Nbt::storage(ResourceLocation::new("example", "data").unwrap())
        .typed_path::<i32>("players.self.mana");
    mana.set(100);
    mana.field("regen").set(true);
}

#[function]
pub fn show_storage_state() {
    let mana = Nbt::storage(ResourceLocation::new("example", "data").unwrap())
        .path("players.self.mana");
    TypedExecute::as_players()
        .when(mana.exists())
        .run(Actionbar::show(
            Target::self_(),
            Text::new("Storage ready").green(),
        ));
}
