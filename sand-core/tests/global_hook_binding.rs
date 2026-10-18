//! Rust callbacks retain their static global context beside owned State inputs.
use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "hook_test", name = "hooked", scope = global, version = 2, migrate(from = 1, to = 2))]
struct Hooked {
    value: Score,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "hook_test", name = "owned", scope = global, version = 1)]
struct Owned {
    #[state(default = 5, min = 0, max = 10)]
    value: Score,
}

fn marker(phase: &str, holder: &'static str) -> Vec<String> {
    const EXPECTED: sand_core::GlobalStateHolder =
        sand_core::GlobalStateHolder::new("hook_test:hooked");
    assert_eq!(holder, EXPECTED.as_str());
    vec![format!("say {phase} {holder}")]
}

#[state_lifecycle]
impl StateLifecycle for Hooked {
    fn initialize(ctx: StateInit) -> Vec<String> {
        marker("initialize", ctx.holder())
    }
    fn reconcile(ctx: StateReconcile) -> Vec<String> {
        marker("reconcile", ctx.holder())
    }
    fn tick(ctx: StateTick) -> Vec<String> {
        marker("tick", ctx.holder())
    }
    fn migrate(ctx: StateMigrate) -> Vec<String> {
        assert_eq!((ctx.from(), ctx.to()), (1, 2));
        marker("migrate", ctx.holder())
    }
}

#[test]
fn global_callbacks_and_owned_globals_share_deterministic_lifecycle_lowering() {
    let first = sand::advanced::try_export_pack_json("hook_test", "26.2").unwrap();
    assert_eq!(
        first,
        sand::advanced::try_export_pack_json("hook_test", "26.2").unwrap()
    );
    let pack: serde_json::Value = serde_json::from_str(&first).unwrap();
    let bodies = pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| record["dir"] == "function")
        .map(|record| record["content"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let holder = sand_core::GlobalStateHolder::new("hook_test:hooked");
    for phase in ["initialize", "reconcile", "tick", "migrate"] {
        assert!(
            bodies.contains(&format!("run say {phase} {}", holder.as_str())),
            "{phase}"
        );
    }
    let owned = sand_core::GlobalStateHolder::new("hook_test:owned");
    assert!(bodies.contains(&format!("scoreboard players set {}", owned.as_str())));
    assert!(!bodies.contains("@s"));
}
