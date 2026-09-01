use sand::prelude::*;

#[derive(StateBundle)]
struct Recursive {
    recursive: Recursive,
}

fn main() {}
