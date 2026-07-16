use sand_core::events::{SandEvent, SandEventDispatch};

macro_rules! event {
    ($name:ident) => {
        struct $name;
        impl SandEvent for $name {
            fn dispatch() -> impl Into<SandEventDispatch> {
                SandEventDispatch::tick().as_players()
            }
        }
    };
}

event!(A); event!(B); event!(C); event!(D); event!(E);
event!(F); event!(G); event!(H); event!(I);

fn main() {
    let _ = SandEventDispatch::after_any::<(A, B, C, D, E, F, G, H, I)>();
}
