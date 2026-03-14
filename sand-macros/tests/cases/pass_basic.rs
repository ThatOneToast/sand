use sand_core::mcfunction;
use sand_macros::function;

#[function]
fn hello_world() {
    mcfunction! {
        "say hello world";
    }
}

fn main() {
    // Function is callable and returns commands.
    let cmds = hello_world();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], "say hello world");

    // Descriptor is registered — we can iterate inventory.
    let mut found = false;
    for d in inventory::iter::<sand_core::FunctionDescriptor>() {
        if d.path == "hello_world" {
            let commands = (d.make)();
            assert_eq!(commands[0], "say hello world");
            found = true;
        }
    }
    assert!(found, "hello_world descriptor not found in inventory");
}
