use sand::prelude::*;
#[function("wrong", context = player)]
fn wrong(a: EntityContext<PlayerKind>, b: EntityContext<PlayerKind>) {
    let _ = (a, b);
}
fn main() {}
