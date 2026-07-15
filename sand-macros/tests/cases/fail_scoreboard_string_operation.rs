use sand_core::cmd::{ScoreOp, scoreboard_players_operation};

fn main() {
    // Use `ScoreHolder` and `ObjectiveName`; unrestricted strings could hide a
    // multi-holder operation source until Minecraft reload.
    let _ = scoreboard_players_operation("@s", "mana", ScoreOp::Set, "@a", "other");
}
