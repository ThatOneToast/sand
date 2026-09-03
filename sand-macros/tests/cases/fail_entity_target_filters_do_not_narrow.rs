use sand::prelude::*;

fn main() {
    // #368: filters preserve `Many`; only explicit narrowing can satisfy a
    // command such as `damage` that requires one entity.
    let _ = cmd::damage(
        Target::entities().tag("elite").within_blocks(8.0),
        1.0,
    );
}
