use sand::prelude::*;

fn main() {
    // #368: player-only capabilities remain unavailable on entity targets.
    let _ = Target::entities().gamemode_typed(GameMode::Survival);
}
