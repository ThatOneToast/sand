use sand::prelude::*;

struct Pulse;
struct Systems;

trait SystemTypes {
    type Event;
}

impl SystemTypes for Systems {
    type Event = Pulse;
}

#[system]
impl Systems {
    #[event(<Self as SystemTypes>::Event)]
    fn invalid(_event: <Self as SystemTypes>::Event) {}
}

fn main() {}
