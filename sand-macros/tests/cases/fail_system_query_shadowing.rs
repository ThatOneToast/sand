use sand::prelude::*;

#[derive(State)]
#[state(namespace = "diagnostics", scope = player)]
struct Mana;

#[system]
fn shadowed(query: Mana) {
    query.each(|_mana| {
        let query = Target::players();
        vec![cmd::kill(query).to_string()]
    });
}

fn main() {}
