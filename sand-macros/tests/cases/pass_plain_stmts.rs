use sand_macros::function;
use sand_core::cmd::{self, Selector};

// Plain expression statements — no mcfunction! needed.
// Each semicolon-terminated expression becomes a command.
// let bindings work for local variables.
#[function]
fn greet() {
    let target = Selector::all_players();
    cmd::say("Welcome to the server!");
    cmd::kill(target);
}

fn main() {
    let cmds = greet();
    assert_eq!(cmds.len(), 2);
    assert!(cmds[0].contains("say"));
    assert!(cmds[1].contains("kill"));
}
