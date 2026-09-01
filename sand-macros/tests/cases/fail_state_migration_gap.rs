use sand::prelude::*;

#[derive(State)]
#[state(
    namespace = "demo",
    scope = entity,
    version = 4,
    migrate(from = 1, to = 2),
    migrate(from = 3, to = 4)
)]
struct BrokenMigrations;

fn main() {}
