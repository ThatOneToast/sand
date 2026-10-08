mod actions_support;
use sand_macros::function;

// A bare &str expression should work as a raw command string.
#[function]
fn my_func() {
    "say hello";
}

fn main() {
    let cmds = actions_support::emitted(my_func());
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], "say hello");
}
