// Canonical recipe: ScoreVar + Cooldown + Flag wired through
// #[component(Load)], #[component(Tick)], and #[function].
use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static CAST_CD: Cooldown = Cooldown::new("cast_cd", Ticks::seconds(3));
static SILENCED: Flag = Flag::new("silenced");

#[component(Load)]
pub fn load() {
    MANA.define();
    CAST_CD.define();
    SILENCED.define();
    MANA.set(Selector::all_players(), 100);
    SILENCED.disable(Selector::all_players());
}

#[component(Tick)]
pub fn tick() {
    CAST_CD.tick(Selector::all_players());
}

#[function]
pub fn cast_bolt() {
    MANA.remove(Selector::self_(), 20);
    CAST_CD.start(Selector::self_());
    cmd::tellraw(Selector::self_(), Text::new("Bolt cast!").aqua());
}

fn main() {
    let load_cmds = load();
    assert!(
        load_cmds
            .iter()
            .any(|c| c.contains("scoreboard objectives add") && c.contains("mana")),
        "expected mana objective creation; got: {load_cmds:?}"
    );
    assert!(
        load_cmds
            .iter()
            .any(|c| c.contains("scoreboard objectives add") && c.contains("cast_cd")),
        "expected cast_cd objective creation; got: {load_cmds:?}"
    );
    assert!(
        load_cmds
            .iter()
            .any(|c| c.contains("scoreboard players set") && c.contains("mana")),
        "expected mana initialisation; got: {load_cmds:?}"
    );

    let tick_cmds = tick();
    assert!(!tick_cmds.is_empty(), "tick should emit at least one command");

    let cast_cmds = cast_bolt();
    assert!(
        cast_cmds
            .iter()
            .any(|c| c.contains("scoreboard players remove") && c.contains("mana")),
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
