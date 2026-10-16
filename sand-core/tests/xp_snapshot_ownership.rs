//! Cached typed XP expressions resolve against each export's detector storage.
use sand::events::PlayerLevelUpEvent;
use sand::prelude::*;
use std::collections::BTreeSet;
use std::sync::OnceLock;

fn body() -> Actions {
    static BODY: OnceLock<Actions> = OnceLock::new();
    BODY.get_or_init(|| {
        let player = Target::current_player();
        when(
            PlayerLevelUpEvent::current_level(player.clone())
                .gte(1)
                .and(PlayerLevelUpEvent::previous_level(player.clone()).gte(0))
                .and(PlayerLevelUpEvent::level_delta(player).gte(1)),
        )
        .then_all(["say owned XP snapshot"])
    })
    .clone()
}
sand_core::inventory::submit! {
    sand_core::EventDescriptor { owner: module_path!(), path: "level", id_override: None, make: body,
        dispatch: sand_core::EventDispatch::XpLevelUp }
}

fn verify(namespace: &str) -> String {
    let export = sand_core::try_export_components_json(namespace).unwrap();
    let records: Vec<serde_json::Value> = serde_json::from_str(&export).unwrap();
    let check = records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == "__sand_xp_check")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    let lines = check.lines().collect::<Vec<_>>();
    let current = lines[0].split_whitespace().nth(5).unwrap();
    let previous = lines.last().unwrap().split_whitespace().nth(4).unwrap();
    let delta = lines[3].split_whitespace().nth(4).unwrap();
    let expected = [current.to_string(), previous.to_string(), delta.to_string()]
        .into_iter()
        .collect::<BTreeSet<_>>();
    assert_eq!(expected.len(), 3);
    let actual = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .flat_map(|r| r["content"].as_str().unwrap().lines())
        .filter_map(|line| {
            line.split_once("run scoreboard players get @s ")
                .map(|(_, objective)| objective.to_string())
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual, expected,
        "snapshot helpers must read the exporting pack's own detector"
    );
    export
}

#[test]
fn cached_snapshots_remain_owned_across_repeated_and_concurrent_exports() {
    let _ = body();
    let a = verify("xp_a");
    let b = verify("xp_b");
    assert_ne!(a, b);
    for worker in ["xp_a", "xp_b", "xp_a", "xp_b"]
        .map(|namespace| std::thread::spawn(move || (namespace, verify(namespace))))
    {
        let (namespace, export) = worker.join().unwrap();
        assert_eq!(
            export,
            if namespace == "xp_a" { &a } else { &b }.to_string()
        );
    }
}
