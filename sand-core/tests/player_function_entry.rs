//! Player entry initialization must run after automatic lifecycle assembly.
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "entry", name = "counter", scope = player, version = 1)]
pub struct Counter {
    #[state(default = 0)]
    pub value: Score,
}

#[function("entry:increment", context = player)]
pub fn increment() {
    Counter::on(EntityContext::<PlayerKind>::default())
        .value
        .add(1);
}

#[function("entry:tick")]
pub fn tick() {
    Execute::new()
        .as_(Target::players())
        .run(cmd::function(increment));
}

#[test]
fn player_entries_initialize_state_before_the_body_on_every_export() {
    for _ in 0..2 {
        let json = sand::advanced::try_export_components_json("entry", "26.2").unwrap();
        let records: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        let content = |path: &str| {
            records
                .iter()
                .find(|record| record["dir"] == "function" && record["path"] == path)
                .unwrap()["content"]
                .as_str()
                .unwrap()
        };
        let player_lines: Vec<_> = content("increment").lines().collect();
        assert_eq!(player_lines.len(), 2);
        assert_eq!(player_lines[0], "function entry:__sand_lifecycle_init");
        assert!(player_lines[1].starts_with("scoreboard players add @s "));
        assert_eq!(
            content("tick"),
            "execute as @a run function entry:increment"
        );
        assert!(content("__sand_lifecycle_init").contains("scoreboard players"));
    }
}
