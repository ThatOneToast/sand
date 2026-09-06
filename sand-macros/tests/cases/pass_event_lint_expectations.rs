#![deny(unfulfilled_lint_expectations)]

use sand::prelude::*;

struct Pulse;

impl SandEvent for Pulse {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

#[on_event]
#[expect(unused_variables)]
fn expected_unused(event: Pulse) {
    cmd::say("expected unused event binding");
}

#[on_event]
#[expect(unused_mut)]
fn expected_mut(mut event: Pulse) {
    cmd::say("expected mutable event binding");
}

fn main() {}
