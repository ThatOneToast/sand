use sand_core::events::{SandEvent, SandEventDispatch};

struct Parent;
impl SandEvent for Parent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

struct NotAnEvent;

fn main() {
    let _ = SandEventDispatch::after_all::<(Parent, NotAnEvent)>();
}
