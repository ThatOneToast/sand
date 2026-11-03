//! Authored lifecycle commands must retain their original return scope.
use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "init_hook", scope = player)]
struct Hooked {
    first: Score,
    second: Score,
}

#[state_lifecycle]
impl StateLifecycle for Hooked {
    fn initialize(_ctx: StateInit) -> Vec<String> {
        vec![cmd::return_cmd(7)]
    }

    fn reconcile(_ctx: StateReconcile) -> Vec<String> {
        vec![cmd::say("reconcile").to_string()]
    }
}

#[function("entry", context = player)]
fn entry() {
    Hooked::on(EntityContext::<PlayerKind>::default())
        .first
        .add(1);
}

#[test]
fn authored_return_keeps_initializer_scope_and_presence_publication_order() {
    let records: Vec<serde_json::Value> =
        serde_json::from_str(&sand_core::try_export_components_json("hookpack").unwrap()).unwrap();
    let initializer = records
        .iter()
        .find(|record| record["path"] == "__sand_lifecycle_init")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    assert_eq!(initializer.lines().count(), 5);
    assert!(
        initializer
            .lines()
            .nth(2)
            .unwrap()
            .ends_with("run return 7")
    );
    assert!(
        initializer
            .lines()
            .nth(3)
            .unwrap()
            .ends_with("run say reconcile")
    );
    assert!(
        initializer
            .lines()
            .last()
            .unwrap()
            .contains("run scoreboard players set @s")
    );
    assert!(!initializer.contains("run function"));
    assert!(!records.iter().any(|record| {
        record["path"]
            .as_str()
            .unwrap()
            .starts_with("sand/state_init/")
    }));
}
