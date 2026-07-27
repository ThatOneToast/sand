use sand::prelude::*;

fn main() {
    // #200: a player target is already `type=player`, so `type=` filters are
    // deliberately not forwarded onto `PlayerTarget<A>` — exposing them would
    // let callers build contradictory selectors like `@a[type=zombie]`.
    let _ = PlayerTargets::all().entity_type("minecraft:zombie");
}
