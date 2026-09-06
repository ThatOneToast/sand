use sand::prelude::*;

#[derive(State)]
#[state(namespace = "diagnostics", scope = entity)]
struct Health;

#[system(tick, every = 0)]
fn invalid_cadence(query: Health) {
    query.each(|_| Vec::new());
}

fn main() {}
