mod feature;
use sand::prelude::*;
use sand_ability_linking_consumer as schema;
fn main() {
    // Force real construction/linkage, then discard without registration.
    let _ = std::hint::black_box(feature::maintenance());
    let _ = std::hint::black_box(
        feature::ability(EntityContext::<PlayerKind>::default())
            .accept(Condition::all([]), cmd::say("unused")),
    );
    schema::export();
}
