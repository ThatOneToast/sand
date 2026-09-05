use sand::prelude::*;

#[derive(State)]
#[state(namespace = "diagnostics", scope = entity)]
struct Health;

struct Pulse;

struct Systems;

#[system]
impl Systems {
    #[event(Pulse)]
    fn invalid(_event: Pulse, _query: Health, _extra: Health) {}
}

fn main() {}
