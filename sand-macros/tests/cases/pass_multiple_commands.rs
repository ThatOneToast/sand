use sand_core::mcfunction;
use sand_macros::function;

#[function]
fn tick() {
    mcfunction! {
        "scoreboard players add @a playtime 1";
        r#"execute as @a[scores={playtime=200}] run tellraw @s {"text":"You've been here 10 seconds!","color":"aqua"}"#;
        "scoreboard players set @a[scores={playtime=200}] playtime 0";
    }
}

fn main() {
    let cmds = tick();
    assert_eq!(cmds.len(), 3);
    assert!(cmds[0].contains("scoreboard"));
}
