use sand::prelude::*;
use sand::events::OnJoinEvent;
#[function("callee", context = player)]
pub fn callee(player: EntityContext<PlayerKind>) {
    player.inventory().offhand().clear().unwrap();
}
#[function("caller", context = player)]
fn caller(player: EntityContext<PlayerKind>) {
    player.call(callee);
}
fn main() {
    // Compile the EntityContext::call catalog example in its real block context.
    #[function("reward", context = player)]
    fn reward() {}
    let player = EntityContext::<PlayerKind>::default();
    let _actions = player.call(reward);
    let _: fn() -> Actions = callee;
    let _: FunctionId = cmd::function_id(caller);
    let _target = Target::from(Event::<OnJoinEvent>::context().player()).gamemode(GameMode::Creative);
}
