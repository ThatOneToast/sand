use sand::prelude::*;

struct Pulse;
struct Systems;

#[system]
impl Systems {
    #[event(Pulse)]
    fn invalid(_event: Pulse) -> Actions {
        Actions::default()
    }
}

fn main() {}
