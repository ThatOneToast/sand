use sand::prelude::*;

#[derive(State)]
#[state(namespace = "direct", scope = global)]
struct GameState {
    wave: Score,
}

#[system]
fn wrong(query: GameState) {
    query.each(|_| Vec::new());
}

fn main() {}
