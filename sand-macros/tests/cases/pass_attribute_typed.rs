use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));
static SETTINGS: StorageVar<i32> = StorageVar::new("example:data", "settings.mana");

fn helper_commands() -> Vec<String> {
    vec![cmd::say("helper command").to_string()]
}

#[component(Load)]
pub fn load() {
    MANA.define();
    DASH.define();
    SETTINGS.set_int(100);
    helper_commands();
}

#[component(Tick)]
pub fn tick() {
    DASH.tick(Selector::all_players());
    TypedExecute::as_players()
        .when(all![
            MANA.of("@s").gte(25),
            any![DASH.ready("@s"), SETTINGS.exists()],
        ])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua().bold(true),
        ));
}

#[function("example:cast_dash")]
pub fn cast_dash() {
    MANA.remove(Selector::self_(), 25);
    DASH.start(Selector::self_());
    cmd::tellraw(Selector::self_(), Text::new("Dash cast").green());
    cmd::raw("function other_pack:api/after_dash");
}

fn main() {
    assert!(load().iter().any(|cmd| cmd.contains("scoreboard objectives add")));
    assert!(tick().iter().any(|cmd| cmd.contains("title @s actionbar")));
    assert!(cast_dash()
        .iter()
        .any(|cmd| cmd == "function other_pack:api/after_dash"));
}
