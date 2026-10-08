//! User function names do not establish ownership of a State initializer.
use sand::prelude::*;

#[function("entry:__sand_lifecycle_init")]
pub fn user_initializer() {
    cmd::say("user function");
}

#[function("entry:player", context = player)]
pub fn player() {
    cmd::say("player body");
}

#[test]
fn player_without_state_does_not_call_a_user_named_initializer() {
    for _ in 0..2 {
        let json = sand::advanced::try_export_components_json("entry", "26.2").unwrap();
        let records: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        let body = |path: &str| {
            records
                .iter()
                .find(|record| record["dir"] == "function" && record["path"] == path)
                .unwrap()["content"]
                .as_str()
                .unwrap()
        };
        assert_eq!(body("player"), "say player body");
        assert_eq!(body("__sand_lifecycle_init"), "say user function");
    }
}
