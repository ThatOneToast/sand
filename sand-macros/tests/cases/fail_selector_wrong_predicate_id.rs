use sand::prelude::*;

fn main() {
    let item = ItemId::minecraft("stone").unwrap();
    let _ = Target::entities().predicate(item);
}
