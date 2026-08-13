//! Regression tests: event bodies that call `then_all` / `unless(...).then_all`
//! / `if_(...).then_all(...).else_all(...)` must register branch functions that
//! survive into the export records.
//!
//! The historic bug: `drain_dyn_fns()` was called immediately after
//! `FunctionDescriptor` iteration, *before* event `make()` bodies ran.
//! Branches registered by events were silently dropped.

use sand_core::execute_when::{if_, unless, when};
use sand_core::state::{Flag, ScoreVar};

static MANA: ScoreVar<i32> = ScoreVar::new("evt_test_mana");
static CASTING: Flag = Flag::new("evt_test_casting");
static BONUS: Flag = Flag::new("evt_test_bonus");

/// Simulate what an `#[on_event]` body does when the export pipeline calls
/// `(desc.make)()` — after the early drain in the old code had already run.
fn simulate_event_body_when() -> Vec<String> {
    let mut cmds = Vec::new();
    cmds.extend(when(MANA.of("@s").gte(25)).then_all(["say mana ok", "say branch works"]));
    cmds
}

fn simulate_event_body_unless() -> Vec<String> {
    let mut cmds = Vec::new();
    cmds.extend(unless(CASTING.of("@s").is_true()).then_all(["say not casting", "say can start"]));
    cmds
}

fn simulate_event_body_if_else() -> Vec<String> {
    let mut cmds = Vec::new();
    cmds.extend(
        if_(BONUS.of("@s").is_true())
            .then_all(["say has bonus"])
            .else_all(["say no bonus"]),
    );
    cmds
}

/// Drain the registry and return all registered branches.
fn drain_branches() -> Vec<(String, Vec<String>)> {
    sand_core::drain_dyn_fns()
}

use std::sync::{Mutex, OnceLock};

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn when_then_all_branch_survives_after_event_make() {
    let _guard = test_lock();
    let _ = drain_branches(); // clean slate

    let event_cmds = simulate_event_body_when();

    // The branch must be in the registry (waiting to be drained by the
    // export pipeline's late drain_dynamic_functions_into call).
    let branches = drain_branches();
    assert!(
        !branches.is_empty(),
        "when().then_all() branch should be registered after event make()"
    );
    assert!(
        branches
            .iter()
            .any(|(path, cmds)| path.contains("sand/branches/")
                && cmds.contains(&"say mana ok".to_string())),
        "branch not found: {branches:?}"
    );

    // The event body must reference the branch.
    assert_eq!(event_cmds.len(), 1);
    assert!(
        event_cmds[0].contains("function __sand_local:sand/branches/"),
        "event cmd should reference branch: {}",
        event_cmds[0]
    );
}

#[test]
fn unless_then_all_branch_survives_after_event_make() {
    let _guard = test_lock();
    let _ = drain_branches();

    let event_cmds = simulate_event_body_unless();

    let branches = drain_branches();
    assert!(
        !branches.is_empty(),
        "unless().then_all() branch should be registered"
    );
    assert!(
        branches
            .iter()
            .any(|(path, cmds)| path.contains("sand/branches/")
                && cmds.contains(&"say not casting".to_string())),
        "branch not found: {branches:?}"
    );
    assert_eq!(event_cmds.len(), 1);
    assert!(
        event_cmds[0].contains("execute unless score @s evt_test_casting"),
        "event cmd polarity: {}",
        event_cmds[0]
    );
    assert!(
        event_cmds[0].contains("function __sand_local:sand/branches/"),
        "event cmd should reference branch: {}",
        event_cmds[0]
    );
}

#[test]
fn if_else_branches_survive_after_event_make() {
    let _guard = test_lock();
    let _ = drain_branches();

    let event_cmds = simulate_event_body_if_else();

    let branches = drain_branches();
    assert_eq!(
        branches.len(),
        4,
        "if_/else_all should register success, failure, success-wrapper, and dispatcher functions: {branches:?}"
    );
    assert!(
        branches
            .iter()
            .any(|(_, cmds)| cmds.contains(&"say has bonus".to_string())),
        "then-branch not found: {branches:?}"
    );
    assert!(
        branches
            .iter()
            .any(|(_, cmds)| cmds.contains(&"say no bonus".to_string())),
        "else-branch not found: {branches:?}"
    );

    let dispatcher = branches
        .iter()
        .find(|(_, cmds)| {
            cmds.first()
                .is_some_and(|command| command.starts_with("scoreboard players set #sand_if_"))
        })
        .expect("single-decision dispatcher not found");
    assert!(
        dispatcher
            .1
            .first()
            .is_some_and(|command| command.starts_with("scoreboard players set #sand_if_")),
        "dispatcher initializes one decision score: {dispatcher:?}"
    );
    assert!(
        dispatcher
            .1
            .last()
            .is_some_and(|command| command.contains("matches 0 run function")),
        "dispatcher failure arm uses the snapshotted decision: {dispatcher:?}"
    );

    // The event calls the dispatcher once; it cannot re-test after either arm.
    assert_eq!(event_cmds.len(), 1);
    assert!(
        event_cmds[0].starts_with("function __sand_local:sand/branches/"),
        "event should call one dispatcher: {}",
        event_cmds[0]
    );
}

#[test]
fn branches_reference_correct_paths() {
    let _guard = test_lock();
    let _ = drain_branches();

    let event_cmds = simulate_event_body_when();
    let branches = drain_branches();

    // Extract the branch path from the event command:
    // "execute if score @s evt_test_mana matches 25.. run function __sand_local:sand/branches/XXXX"
    let branch_ref = event_cmds[0]
        .split("function ")
        .nth(1)
        .expect("no function ref in event cmd");
    // Strip "__sand_local:" prefix to get the path
    let branch_path = branch_ref
        .strip_prefix("__sand_local:")
        .unwrap_or(branch_ref);

    assert!(
        branches.iter().any(|(path, _)| path == branch_path),
        "referenced branch path '{branch_path}' not found in registered branches: {branches:?}"
    );
}
