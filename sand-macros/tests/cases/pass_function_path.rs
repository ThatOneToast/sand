use sand_macros::function;
use sand_core::mcfunction;

#[function("foo/bar")]
fn bar() {
    mcfunction! {
        "say hello from foo/bar";
    }
}

fn main() {
    let cmds = bar();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], "say hello from foo/bar");
}
