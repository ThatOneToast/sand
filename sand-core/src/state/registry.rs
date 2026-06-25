//! Global opt-in lifecycle registry for load and tick wiring.
//!
//! Users explicitly register objectives and tick handlers; Sand collects them
//! and emits the appropriate commands when building the datapack.
//!
//! # Pattern
//!
//! ```rust,ignore
//! use sand_core::state::{ScoreVar, register_load_objective};
//!
//! static MANA: ScoreVar<i32> = ScoreVar::new("mana");
//!
//! fn load() -> Vec<String> {
//!     register_load_objective(MANA.objective_name(), "dummy");
//!     drain_load_commands() // called once by the export pipeline
//! }
//! ```

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

// ── Internal storage types ────────────────────────────────────────────────────

/// An objective registered for definition on datapack load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadEntry {
    pub objective: String,
    pub criterion: String,
}

/// A named tick handler, deduplicated by `id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickEntry {
    pub id: String,
    pub commands: Vec<String>,
}

// ── Registry accessors ────────────────────────────────────────────────────────

fn load_registry() -> &'static Mutex<BTreeMap<String, String>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn tick_registry() -> &'static Mutex<BTreeMap<String, Vec<String>>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, Vec<String>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Register a scoreboard objective to be defined on datapack load.
///
/// - Same `objective` + same `criterion` → no-op (idempotent).
/// - Same `objective` + different `criterion` → panics with a clear message.
pub fn register_load_objective(objective: impl Into<String>, criterion: impl Into<String>) {
    let objective = objective.into();
    let criterion = criterion.into();

    let mut registry = load_registry().lock().unwrap_or_else(|e| e.into_inner());

    match registry.get(&objective) {
        Some(existing) if existing == &criterion => {
            // Identical registration — no-op.
        }
        Some(existing) => {
            panic!(
                "conflicting Sand load objective `{objective}`: already registered with \
                 criterion `{existing}`, attempted to register with `{criterion}`"
            );
        }
        None => {
            registry.insert(objective, criterion);
        }
    }
}

/// Register a named tick handler.
///
/// - Same `id` + same `commands` → no-op (idempotent).
/// - Same `id` + different `commands` → panics with a clear message.
pub fn register_tick_handler(id: impl Into<String>, commands: Vec<String>) {
    let id = id.into();

    let mut registry = tick_registry().lock().unwrap_or_else(|e| e.into_inner());

    match registry.get(&id) {
        Some(existing) if existing == &commands => {
            // Identical registration — no-op.
        }
        Some(_) => {
            panic!(
                "conflicting Sand tick handler `{id}`: already registered with different commands"
            );
        }
        None => {
            registry.insert(id, commands);
        }
    }
}

/// Return and drain all registered load objective commands.
///
/// Returns `scoreboard objectives add <obj> <criterion>` for each registered
/// objective, **sorted by objective name** for deterministic output.
///
/// Drains the registry — subsequent calls return an empty `Vec` until new
/// objectives are registered.
pub fn drain_load_commands() -> Vec<String> {
    let mut registry = load_registry().lock().unwrap_or_else(|e| e.into_inner());
    let entries = std::mem::take(&mut *registry);
    // BTreeMap iteration is already sorted by key (objective name).
    entries
        .into_iter()
        .map(|(objective, criterion)| format!("scoreboard objectives add {objective} {criterion}"))
        .collect()
}

/// Return and drain all registered tick handler commands.
///
/// Returns commands for every registered tick handler, **sorted by handler id**
/// for deterministic output. Commands within a single handler appear in
/// registration order.
///
/// Drains the registry — subsequent calls return an empty `Vec` until new
/// handlers are registered.
pub fn drain_tick_commands() -> Vec<String> {
    let mut registry = tick_registry().lock().unwrap_or_else(|e| e.into_inner());
    let entries = std::mem::take(&mut *registry);
    // BTreeMap iteration is already sorted by key (handler id).
    entries.into_values().flatten().collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::MutexGuard;

    use super::*;

    /// A test-only serialization lock that ensures registry tests never run in
    /// parallel with each other. This prevents one test's mutations from
    /// polluting another test's assertions in the shared global registries.
    fn test_lock() -> MutexGuard<'static, ()> {
        static TEST_MUTEX: OnceLock<std::sync::Mutex<()>> = OnceLock::new();
        let m = TEST_MUTEX.get_or_init(|| std::sync::Mutex::new(()));
        // Recover from poison so a panicking `#[should_panic]` test does not
        // permanently block subsequent tests.
        m.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Helper: drain both registries to avoid test pollution.
    ///
    /// Recovers from a poisoned mutex (which can happen when a `#[should_panic]`
    /// test panics while holding the lock) so that subsequent tests are not
    /// contaminated by a globally-poisoned state.
    fn drain_all() {
        // Recover from poison by taking the inner value and clearing it.
        {
            let mut guard = match load_registry().lock() {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
            guard.clear();
        }
        {
            let mut guard = match tick_registry().lock() {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
            guard.clear();
        }
    }

    #[test]
    fn register_same_objective_twice_is_noop() {
        let _lock = test_lock();
        drain_all();
        register_load_objective("alpha", "dummy");
        register_load_objective("alpha", "dummy"); // identical — must not panic
        let cmds = drain_load_commands();
        assert_eq!(cmds, vec!["scoreboard objectives add alpha dummy"]);
    }

    #[test]
    #[should_panic(expected = "conflicting Sand load objective")]
    fn register_conflicting_objective_panics() {
        let _lock = test_lock();
        drain_all();
        register_load_objective("conflict_obj", "dummy");
        register_load_objective("conflict_obj", "playerKillCount");
    }

    #[test]
    fn drain_load_commands_sorted_by_objective() {
        let _lock = test_lock();
        drain_all();
        register_load_objective("zeta", "dummy");
        register_load_objective("alpha", "dummy");
        register_load_objective("mana", "dummy");
        let cmds = drain_load_commands();
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], "scoreboard objectives add alpha dummy");
        assert_eq!(cmds[1], "scoreboard objectives add mana dummy");
        assert_eq!(cmds[2], "scoreboard objectives add zeta dummy");
    }

    #[test]
    fn drain_load_commands_after_drain_is_empty() {
        let _lock = test_lock();
        drain_all();
        register_load_objective("temp", "dummy");
        drain_load_commands(); // first drain
        let second = drain_load_commands();
        assert!(
            second.is_empty(),
            "expected empty after drain, got: {second:?}"
        );
    }

    #[test]
    fn multiple_objectives_all_appear() {
        let _lock = test_lock();
        drain_all();
        register_load_objective("foo", "dummy");
        register_load_objective("bar", "playerKillCount");
        let cmds = drain_load_commands();
        assert_eq!(cmds.len(), 2);
        assert!(cmds.iter().any(|c| c.contains("foo")));
        assert!(cmds.iter().any(|c| c.contains("bar")));
    }

    #[test]
    fn register_same_tick_handler_twice_is_noop() {
        let _lock = test_lock();
        drain_all();
        let cmds = vec!["scoreboard players remove @a cooldown 1".to_string()];
        register_tick_handler("cooldown/dash", cmds.clone());
        register_tick_handler("cooldown/dash", cmds); // identical — must not panic
        let out = drain_tick_commands();
        assert_eq!(out.len(), 1);
    }

    #[test]
    #[should_panic(expected = "conflicting Sand tick handler")]
    fn register_conflicting_tick_handler_panics() {
        let _lock = test_lock();
        drain_all();
        register_tick_handler("handler/x", vec!["say first".to_string()]);
        register_tick_handler("handler/x", vec!["say second".to_string()]);
    }

    #[test]
    fn drain_tick_commands_sorted_by_handler_id() {
        let _lock = test_lock();
        drain_all();
        register_tick_handler("z_handler", vec!["say z".to_string()]);
        register_tick_handler("a_handler", vec!["say a".to_string()]);
        let cmds = drain_tick_commands();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "say a");
        assert_eq!(cmds[1], "say z");
    }

    #[test]
    fn drain_tick_commands_after_drain_is_empty() {
        let _lock = test_lock();
        drain_all();
        register_tick_handler("tmp_handler", vec!["say hi".to_string()]);
        drain_tick_commands();
        let second = drain_tick_commands();
        assert!(
            second.is_empty(),
            "expected empty after drain, got: {second:?}"
        );
    }
}
