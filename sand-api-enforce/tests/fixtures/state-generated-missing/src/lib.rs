use sand::State;

#[derive(State)]
#[state(namespace = "fixture", scope = player)]
#[allow(dead_code)]
pub struct PlayerState {
    mana: sand::__private::EntityScore<i32>,
}

const _: sand::__private::EntityScore<i32> = PlayerState::mana;
const _: fn(sand::__private::EntityContext<sand::__private::PlayerKind>) -> PlayerStateBound =
    PlayerState::on;

#[doc(hidden)]
pub mod __private {
    include!(concat!(env!("OUT_DIR"), "/api_enforcement.rs"));
}
