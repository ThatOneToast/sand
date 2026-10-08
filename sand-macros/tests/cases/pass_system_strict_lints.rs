#![forbid(dead_code, unused_must_use, unused_variables)]
#![deny(unfulfilled_lint_expectations)]

use sand::prelude::*;

#[derive(State)]
#[state(namespace = "strict_lints", scope = player)]
struct Health;

struct Pulse;

impl SandEvent for Pulse {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

#[on_event]
fn strict_event(event @ _: Pulse) {
    cmd::say("strict event");
}

#[system]
fn free(query: Health) {
    query.each(|_health| sand::command::Actions::default());
}

struct Systems;

#[system]
impl Systems {
    #[tick]
    fn tick(query: Health) {
        query.each(|_health| sand::command::Actions::default());
    }

    #[event(Pulse)]
    #[expect(unused_mut)]
    fn event(_event: Pulse, mut query: Health) {
        query.current(|_health| sand::command::Actions::default());
    }
}

fn main() {}
