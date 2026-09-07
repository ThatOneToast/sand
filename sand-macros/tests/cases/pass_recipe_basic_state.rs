// Canonical recipe: derived player State wired through component functions.
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "recipe", scope = player)]
#[allow(dead_code)]
struct PlayerState {
    #[state(default = 100, min = 0, max = 100)]
    mana: Score,
    #[state(auto_tick)]
    cast_cooldown: Cooldown,
    silenced: Flag,
}

fn player_state() -> PlayerStateBound {
    PlayerState::on(EntityContext::<PlayerKind>::default())
}

#[datapack_component(Load)]
pub fn load() {
    cmd::say("recipe state loaded");
}

#[datapack_component(Tick)]
pub fn tick() {
    cmd::say("recipe state tick");
}

#[function]
pub fn cast_bolt() {
    let player = player_state();
    player.mana.subtract(20);
    player.cast_cooldown.start(Ticks::seconds(3));
    cmd::tellraw(Target::self_(), Text::new("Bolt cast!").aqua());
}

fn main() {
    let load_cmds = load();
    assert_eq!(load_cmds, ["say recipe state loaded"]);

    let tick_cmds = tick();
    assert!(!tick_cmds.is_empty(), "tick should emit at least one command");

    let cast_cmds = cast_bolt();
    assert!(
        cast_cmds
            .iter()
            .any(|c| c.starts_with("scoreboard players remove @s ") && c.ends_with(" 20")),
        "expected mana remove; got: {cast_cmds:?}"
    );
    assert!(
        cast_cmds.iter().any(|c| c.contains("tellraw")),
        "expected tellraw; got: {cast_cmds:?}"
    );

    // Descriptors registered via inventory linkme
    let mut found_load = false;
    let mut found_cast = false;
    for d in inventory::iter::<sand_core::FunctionDescriptor>() {
        if d.path == "load" {
            found_load = true;
        }
        if d.path == "cast_bolt" {
            found_cast = true;
        }
    }
    assert!(found_load, "load descriptor not registered");
    assert!(found_cast, "cast_bolt descriptor not registered");
}
