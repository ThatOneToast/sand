use sand_macros::{function, run_fn};
use sand_core::cmd::{self, Execute, Selector};

#[function]
fn my_fn() {
    Execute::new()
        .as_(Selector::all_players())
        .run(run_fn!("test:valid_path" {
            cmd::say("from inline fn");
        }));
}

fn main() {
    let cmds = my_fn();
    assert_eq!(cmds.len(), 1);
    assert!(cmds[0].starts_with("execute as @a run function test:valid_path"));
}
