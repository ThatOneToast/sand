//! Player join handling with player-scoped derived State.

use sand::prelude::*;

#[derive(State)]
#[state(namespace = "my_pack", scope = player)]
struct Visits {
    #[state(default = 0)]
    count: Score,
}

#[on_event]
pub fn on_player_join(event: Event<sand::events::OnJoinEvent>) {
    let _ = event;
    Visits::on(EntityContext::<PlayerKind>::default())
        .count
        .add(1);
    cmd::tellraw(
        Target::self_(),
        Text::new("Welcome back. Your visit counter was updated.").gold(),
    );
}
