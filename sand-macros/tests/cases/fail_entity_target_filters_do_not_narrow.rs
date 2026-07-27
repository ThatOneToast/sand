use sand::prelude::*;

fn needs_one(_target: SingleEntity) {}

fn main() {
    // #200: forwarded typed filters preserve the cardinality marker `A` —
    // they never narrow `Many` to `One`. Narrowing still has to go through
    // `EntityTargets::limit(1)` / `EntityTargets::nearest()`.
    needs_one(EntityTargets::all().tag("elite").within_blocks(8.0));
}
