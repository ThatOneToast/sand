use sand::prelude::*;

#[derive(State)]
#[state(namespace = "quickstart", scope = player)]
#[allow(dead_code)]
struct PlayerState {
    #[state(default = 0, min = 0, max = 100)]
    mana: Score,
}

#[datapack_component(Load)]
pub fn load() {
    cmd::say("quickstart loaded");
}

#[function]
pub fn reward() {
    PlayerState::on(EntityContext::<PlayerKind>::default())
        .mana
        .add(10);
    cmd::tellraw(Target::self_(), Text::new("+10 mana").aqua());
}

fn main() {
    let _ = load();
    let _ = reward();
}
