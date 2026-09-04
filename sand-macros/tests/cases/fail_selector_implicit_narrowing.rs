use sand::prelude::*;

fn main() {
    // A many-target value never narrows merely because a single-target
    // command consumes it. Use `.nearest()` or `.limit(1)` explicitly.
    let _ = cmd::damage(Target::entities(), 1.0);
}
