use sand::prelude::*;

struct Pulse;

impl SandEvent for Pulse {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

struct Systems;

#[system]
impl Systems {
    #[tick]
    fn invalid_tick(_query: String) {
        cmd::say("invalid tick query");
    }

    #[event(Pulse)]
    fn invalid_event(_event: Pulse, _query: String) {
        cmd::say("invalid event query");
    }
}

fn main() {}
