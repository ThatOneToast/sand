use sand_core::prelude::*;

fn main() {
    let _ = cmd::damage(EntityTargets::nearby(5.0), 4.0);
}
