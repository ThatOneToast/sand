use sand_core::sand_state;
use sand_core::state::{Cooldown, GameState, ScoreVar, Ticks, TypedGameState};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Lobby = 0,
    Playing = 1,
}

impl TypedGameState for Phase {
    fn to_score(self) -> i32 {
        self as i32
    }

    fn from_score(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Lobby),
            1 => Some(Self::Playing),
            _ => None,
        }
    }
}

sand_state! {
    static MANA: ScoreVar<i32> = ScoreVar::new("mana") =>
        MANA.lifecycle().default(100);
    static PHASE: GameState<Phase> = GameState::with_default_score("phase", 0) =>
        PHASE.lifecycle();
    static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60)) =>
        DASH.lifecycle().default(0).auto_tick();
}

fn main() {
    // No load/tick function and no manual registry drain are required.
    let _typed_api_still_works = (
        MANA.set("@s", 50),
        PHASE.of("@s").set(Phase::Playing),
        DASH.start("@s"),
    );
    println!(
        "{}",
        sand_core::try_export_components_json("automatic_state").unwrap()
    );
}
