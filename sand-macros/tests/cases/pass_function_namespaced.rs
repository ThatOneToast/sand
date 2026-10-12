mod actions_support;
use sand_macros::function;
use sand_core::mcfunction;

#[function("other_pack:api/run")]
fn run() {
    mcfunction! {
        "say namespaced";
    }
}

fn main() {
    let cmds = actions_support::emitted(run());
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], "say namespaced");
}
