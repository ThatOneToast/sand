use sand::prelude::*;
#[function("wrong", context = player)]
fn wrong(player: EntityContext<AnyEntity>) {
    let _ = player;
}
fn main() {}
