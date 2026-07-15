use sand_core::prelude::*;

fn needs_one(_target: SingleEntity) {}

fn main() {
    // Selector cardinality is not guaranteed by the unrestricted type. Use
    // `SingleEntity::try_from(selector)` so validation cannot be skipped.
    needs_one(Selector::all_entities().into());
}
