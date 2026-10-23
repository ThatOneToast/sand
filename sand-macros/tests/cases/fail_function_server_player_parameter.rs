use sand::prelude::*;
#[function("wrong", context = server)]
fn wrong(player: EntityContext<PlayerKind>) {
    let _ = player;
}
fn main() {}
