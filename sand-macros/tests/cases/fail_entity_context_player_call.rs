use sand::prelude::*;
#[function("callee", context = player)]
fn callee() {}
fn main() {
    let entity = EntityContext::<AnyEntity>::default();
    let _ = entity.call(callee);
}
