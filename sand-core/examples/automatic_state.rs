use sand::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EntityStateEnum)]
enum Phase {
    Lobby = 0,
    Playing = 1,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "demo", scope = player)]
struct PlayerState {
    #[state(default = 100, min = 0, max = 100)]
    mana: EntityScore<i32>,
    #[state(default = "Phase::Lobby")]
    phase: EntityEnum<Phase>,
    #[state(auto_tick)]
    dash: EntityCooldown,
}

fn main() {
    let state = PlayerState::on(EntityContext::<PlayerKind>::default());
    let _commands = (
        state.mana.set(50),
        state.phase.set(Phase::Playing),
        state.dash.start(Ticks::new(60)),
    );
    println!(
        "{}",
        sand_core::try_export_components_json("automatic_state").unwrap()
    );
}
