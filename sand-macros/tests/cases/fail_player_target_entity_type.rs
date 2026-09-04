use sand::prelude::*;

fn main() {
    // #368: a player target is already type=player, so entity-type filters
    // remain unavailable and contradictory selectors cannot be built.
    let _ = Target::players().entity_type("minecraft:zombie");
}
