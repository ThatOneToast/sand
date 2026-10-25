use sand::prelude::*;

fn main() {
    EntityContext::<AnyEntity>::default().equipment().mainhand();
    EntityContext::<MarkerKind>::default().equipment().boots();
    EntityContext::<ZombieKind>::default().inventory().mainhand();
}
