//! #368: one canonical `Target` preserves category and cardinality while
//! remaining directly consumable by command APIs.

use sand::prelude::*;

fn main() {
    let entities = Target::entities()
        .entity_type("minecraft:zombie")
        .without_tag("done")
        .within_blocks(16.0)
        .scores_raw("threat=5..")
        .nbt_raw("{Silent:1b}")
        .predicate_raw("pack:is_burning");
    let _ = cmd::kill().targets(entities);

    let one = Target::entities().tag("elite").nearest();
    let _ = cmd::damage(one, 5.0);

    let players = Target::players()
        .tag("ready")
        .gamemode_typed(GameMode::Adventure)
        .not_gamemode_typed(GameMode::Spectator)
        .level("10..30");
    let _ = cmd::tellraw(players, Text::new("Ready"));

    let nearest_player = Target::players().tag("ready").nearest();
    let _ = cmd::damage(nearest_player, 1.0);
}
