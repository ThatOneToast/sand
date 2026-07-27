use sand::prelude::*;

fn main() {
    // #200: `gamemode` is a player-only vanilla selector argument, so the
    // typed filter is deliberately exposed only on `PlayerTarget<A>`. An
    // entity target must be narrowed to a player target first.
    let _ = EntityTargets::all().gamemode_typed(GameMode::Survival);
}
