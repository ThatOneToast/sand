mod actions_support;
use sand_macros::{function, run_fn};
use sand_core::cmd::{self, Execute, ExecuteExt, Target};

#[function]
fn my_fn() {
    Execute::new()
        .as_(Target::all_players())
        .run_actions(run_fn!("test:valid_path" {
            cmd::say("from inline fn");
        }));
}

fn main() {
    let cmds = actions_support::emitted(my_fn());
    assert_eq!(cmds.len(), 1);
    assert!(cmds[0].starts_with("execute as @a run function test:valid_path"));
}
