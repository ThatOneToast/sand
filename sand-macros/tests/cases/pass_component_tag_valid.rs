mod actions_support;
use sand_macros::datapack_component;
use sand_core::mcfunction;

#[datapack_component(Tag = "my_lib:on_death")]
fn handle_death() {
    mcfunction! {
        "say death handler";
    }
}

fn main() {
    let cmds = actions_support::emitted(handle_death());
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], "say death handler");
}
