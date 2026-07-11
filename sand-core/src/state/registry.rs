//! Automatic declarations and the legacy manual lifecycle registry.
//!
//! [`StateLifecycle`] descriptors submitted at link time are rebuilt on every
//! export. The manual registration and drain APIs remain available as an escape
//! hatch and for compatibility.
//!
//! # Pattern — manual typed state registration
//!
//! Call `.register()` on any typed state variable to enroll it in the lifecycle
//! registry. Then call [`define_registered_state`] once in your `load` function
//! to emit all the `scoreboard objectives add …` commands.
//!
//! ```rust,ignore
//! use sand_core::state::{ScoreVar, Flag, Cooldown, Ticks};
//! use sand_core::state::{define_registered_state, register_load_objective};
//!
//! static MANA:  ScoreVar<i32> = ScoreVar::new("mana");
//! static ALIVE: Flag          = Flag::new("alive");
//! static DASH:  Cooldown      = Cooldown::new("dash", Ticks::new(60));
//!
//! fn load() -> Vec<String> {
//!     MANA.register();
//!     ALIVE.register();
//!     DASH.register();
//!     define_registered_state() // emits all three objectives, sorted
//! }
//! ```
//!
//! Manual `.define()` continues to work. Choose one approach per objective:
//! - Use `.register()` + [`define_registered_state`] for lifecycle-managed output.
//! - Use `.define()` directly when you want a standalone command string.
//!
//! Note: `.define()` returns a command string independent of the registry.
//! If you mix both approaches for the same objective, you must take care not to
//! include both the manual command string and the lifecycle drain in the same
//! load function — the registry deduplicates only among `.register()` calls,
//! not between `.define()` strings and `.register()` output.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use super::score::objective_name;

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

/// One automatically exported typed-state lifecycle declaration.
///
/// This is the non-macro model behind [`crate::sand_state!`]. Submit a
/// descriptor with [`inventory::submit!`](crate::inventory::submit) when a
/// project wants declaration-time discovery without using the convenience
/// macro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateLifecycle {
    objective: &'static str,
    criterion: &'static str,
    default: Option<i32>,
    auto_tick: bool,
}

impl StateLifecycle {
    /// Declare a dummy scoreboard-backed state objective.
    pub const fn score(objective: &'static str) -> Self {
        Self {
            objective,
            criterion: "dummy",
            default: None,
            auto_tick: false,
        }
    }

    /// Override the vanilla scoreboard criterion.
    pub const fn criterion(mut self, criterion: &'static str) -> Self {
        self.criterion = criterion;
        self
    }

    /// Initialize players that do not yet have a score, without overwriting
    /// existing progress.
    pub const fn default(mut self, value: i32) -> Self {
        self.default = Some(value);
        self
    }

    /// Opt this state into per-player countdown ticking.
    pub const fn auto_tick(mut self) -> Self {
        self.auto_tick = true;
        self
    }
}

/// Link-time descriptor for an automatically managed state declaration.
pub struct StateDescriptor {
    pub lifecycle: StateLifecycle,
}

impl StateDescriptor {
    pub const fn new(lifecycle: StateLifecycle) -> Self {
        Self { lifecycle }
    }
}

inventory::collect!(StateDescriptor);

/// Deterministic automatic lifecycle output built afresh for each export.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct AutomaticLifecycle {
    pub load_commands: Vec<String>,
    pub init_commands: Vec<String>,
    pub tick_commands: Vec<String>,
}

/// Resolve link-time state declarations without mutating the manual registry.
///
/// Identical declarations deduplicate. Any criterion, default, or auto-tick
/// disagreement for the same resolved objective is returned as a contextual
/// error instead of silently choosing one definition.
pub(crate) fn automatic_lifecycle() -> Result<AutomaticLifecycle, String> {
    automatic_lifecycle_from(
        inventory::iter::<StateDescriptor>().map(|descriptor| descriptor.lifecycle.clone()),
    )
}

fn automatic_lifecycle_from(
    declarations: impl IntoIterator<Item = StateLifecycle>,
) -> Result<AutomaticLifecycle, String> {
    let mut states: BTreeMap<String, (&'static str, Option<i32>, bool)> = BTreeMap::new();
    let mut declarations: Vec<_> = declarations.into_iter().collect();
    declarations.sort_by_key(|declaration| {
        (
            objective_name(declaration.objective),
            declaration.criterion,
            declaration.default,
            declaration.auto_tick,
        )
    });

    for declaration in declarations {
        let objective = objective_name(declaration.objective);
        let definition = (
            declaration.criterion,
            declaration.default,
            declaration.auto_tick,
        );
        match states.get(&objective) {
            Some(existing) if existing == &definition => {}
            Some(existing) => {
                return Err(format!(
                    "conflicting automatic state `{objective}`: first declaration has criterion `{}`, default {:?}, auto_tick {}; conflicting declaration has criterion `{}`, default {:?}, auto_tick {}",
                    existing.0, existing.1, existing.2, definition.0, definition.1, definition.2
                ));
            }
            None => {
                states.insert(objective, definition);
            }
        }
    }

    let mut output = AutomaticLifecycle::default();
    for (objective, (criterion, default, auto_tick)) in states {
        output
            .load_commands
            .push(format!("scoreboard objectives add {objective} {criterion}"));
        if let Some(default) = default {
            output.init_commands.push(format!(
                "execute unless score @s {objective} matches -2147483648.. run scoreboard players set @s {objective} {default}"
            ));
        }
        if auto_tick {
            output.tick_commands.push(format!(
                "execute if score @s {objective} matches 1.. run scoreboard players remove @s {objective} 1"
            ));
        }
    }
    Ok(output)
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

/// Emit and drain all `scoreboard objectives add …` commands for every state
/// variable that was registered via `.register()`.
///
/// Call this **once** at the start of your datapack's `load` function to
/// produce the objective definitions for all typed state variables enrolled in
/// the lifecycle registry.
///
/// Commands are sorted by objective name for deterministic output.
/// This drains the registry — subsequent calls return an empty `Vec` until new
/// objectives are registered.
///
/// # Interoperability with manual `.define()`
///
/// `.define()` returns a command string without interacting with this registry.
/// Registry deduplication applies only among `.register()` calls. Do not
/// include both a manual `.define()` command string and the output of
/// [`define_registered_state`] for the same objective in the same load
/// function, or you will emit duplicate scoreboard definitions.
///
/// # Example
///
/// ```rust,ignore
/// fn load() -> Vec<String> {
///     MANA.register();
///     ALIVE.register();
///     define_registered_state()
/// }
/// ```
pub fn define_registered_state() -> Vec<String> {
    drain_load_commands()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// A test-only serialization lock shared across any test that touches the global
/// lifecycle registries.  Lives at module level (not inside `mod tests`) so that
/// component-level tests in `sand_core::component` can import and use it too.
///
/// Uses the same poison-recovery pattern so a `#[should_panic]` test does not
/// permanently block subsequent tests.
#[cfg(test)]
pub(crate) fn registry_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static TEST_MUTEX: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    let m = TEST_MUTEX.get_or_init(|| std::sync::Mutex::new(()));
    m.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use std::sync::MutexGuard;

    use super::*;

    /// Delegate to the module-level lock so both this module and component tests
    /// serialize against the same mutex.
    fn test_lock() -> MutexGuard<'static, ()> {
        super::registry_test_lock()
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

    // ── define_registered_state tests ─────────────────────────────────────────

    #[test]
    fn define_registered_state_drains_load_registry() {
        let _lock = test_lock();
        drain_all();
        register_load_objective("mana", "dummy");
        register_load_objective("alive", "dummy");
        let cmds = define_registered_state();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "scoreboard objectives add alive dummy");
        assert_eq!(cmds[1], "scoreboard objectives add mana dummy");
    }

    #[test]
    fn define_registered_state_deduplicates_repeated_registration() {
        let _lock = test_lock();
        drain_all();
        register_load_objective("dash", "dummy");
        register_load_objective("dash", "dummy");
        register_load_objective("dash", "dummy");
        let cmds = define_registered_state();
        assert_eq!(cmds, vec!["scoreboard objectives add dash dummy"]);
    }

    #[test]
    fn define_registered_state_after_drain_returns_empty() {
        let _lock = test_lock();
        drain_all();
        register_load_objective("blink", "dummy");
        define_registered_state(); // first call drains
        let second = define_registered_state();
        assert!(
            second.is_empty(),
            "expected empty after drain, got: {second:?}"
        );
    }

    #[test]
    fn define_registered_state_with_no_registrations_returns_empty() {
        let _lock = test_lock();
        drain_all();
        let cmds = define_registered_state();
        assert!(
            cmds.is_empty(),
            "expected empty when nothing registered, got: {cmds:?}"
        );
    }

    // ── typed state .register() tests ─────────────────────────────────────────

    #[test]
    fn score_var_register_enrolls_in_lifecycle() {
        use crate::state::ScoreVar;
        let _lock = test_lock();
        drain_all();
        static MANA: ScoreVar<i32> = ScoreVar::new("mana");
        MANA.register();
        let cmds = define_registered_state();
        assert_eq!(cmds, vec!["scoreboard objectives add mana dummy"]);
    }

    #[test]
    fn flag_register_enrolls_in_lifecycle() {
        use crate::state::Flag;
        let _lock = test_lock();
        drain_all();
        static ALIVE: Flag = Flag::new("alive");
        ALIVE.register();
        let cmds = define_registered_state();
        assert_eq!(cmds, vec!["scoreboard objectives add alive dummy"]);
    }

    #[test]
    fn timer_register_enrolls_in_lifecycle() {
        use crate::state::{Ticks, Timer};
        let _lock = test_lock();
        drain_all();
        static BLINK: Timer = Timer::new("blink_cd", Ticks::new(60));
        BLINK.register();
        let cmds = define_registered_state();
        assert_eq!(cmds, vec!["scoreboard objectives add blink_cd dummy"]);
    }

    #[test]
    fn cooldown_register_enrolls_in_lifecycle() {
        use crate::state::{Cooldown, Ticks};
        let _lock = test_lock();
        drain_all();
        static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
        DASH.register();
        let cmds = define_registered_state();
        assert_eq!(cmds, vec!["scoreboard objectives add dash dummy"]);
    }

    #[test]
    fn multiple_typed_state_register_sorted_deduped() {
        use crate::state::{Cooldown, Flag, ScoreVar, Ticks, Timer};
        let _lock = test_lock();
        drain_all();
        static MANA: ScoreVar<i32> = ScoreVar::new("mana");
        static ALIVE: Flag = Flag::new("alive");
        static BLINK: Timer = Timer::new("blink_cd", Ticks::new(20));
        static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));

        DASH.register();
        MANA.register();
        ALIVE.register();
        BLINK.register();
        // register MANA a second time — must still produce one entry
        MANA.register();

        let cmds = define_registered_state();
        assert_eq!(cmds.len(), 4, "expected 4 unique objectives, got: {cmds:?}");
        assert_eq!(cmds[0], "scoreboard objectives add alive dummy");
        assert_eq!(cmds[1], "scoreboard objectives add blink_cd dummy");
        assert_eq!(cmds[2], "scoreboard objectives add dash dummy");
        assert_eq!(cmds[3], "scoreboard objectives add mana dummy");
    }

    #[test]
    fn repeated_lifecycle_registration_paths_dedupe() {
        use crate::state::ScoreVar;
        let _lock = test_lock();
        drain_all();
        static SCORE: ScoreVar<i32> = ScoreVar::new("points");

        // Both calls write to the lifecycle registry and deduplicate.
        SCORE.register();
        register_load_objective("points", "dummy");

        let cmds = define_registered_state();
        assert_eq!(
            cmds,
            vec!["scoreboard objectives add points dummy"],
            "identical lifecycle registrations must produce exactly one entry"
        );
    }

    #[test]
    fn game_state_register_enrolls_in_lifecycle() {
        use crate::state::{GameState, TypedGameState};

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum BossPhase {
            Idle = 0,
            Enraged = 1,
        }

        impl TypedGameState for BossPhase {
            fn to_score(self) -> i32 {
                self as i32
            }

            fn from_score(n: i32) -> Option<Self> {
                match n {
                    0 => Some(Self::Idle),
                    1 => Some(Self::Enraged),
                    _ => None,
                }
            }
        }

        let _lock = test_lock();
        drain_all();
        static PHASE: GameState<BossPhase> = GameState::new("boss_phase");
        PHASE.register();
        let cmds = define_registered_state();
        assert_eq!(cmds, vec!["scoreboard objectives add boss_phase dummy"]);
    }

    #[test]
    fn manual_define_returns_string_independent_of_registry() {
        use crate::state::ScoreVar;
        let _lock = test_lock();
        drain_all();
        static SCORE: ScoreVar<i32> = ScoreVar::new("pts");

        // .define() returns a command string without touching the registry
        let cmd = SCORE.define();
        assert_eq!(cmd, "scoreboard objectives add pts dummy");

        // Registry is still empty — .define() did not register anything
        let registry_cmds = define_registered_state();
        assert!(
            registry_cmds.is_empty(),
            "define() must not side-effect the registry: {registry_cmds:?}"
        );

        // Now register explicitly
        SCORE.register();
        let registry_cmds = define_registered_state();
        assert_eq!(registry_cmds, vec!["scoreboard objectives add pts dummy"]);

        // Both cmd and registry_cmds contain the same command — the user must
        // choose one path; mixing both in a load function would duplicate output.
        assert_eq!(cmd, registry_cmds[0]);
    }

    #[test]
    fn automatic_declarations_sort_dedupe_and_generate_player_safe_commands() {
        let output = automatic_lifecycle_from([
            StateLifecycle::score("z_timer").default(0).auto_tick(),
            StateLifecycle::score("alpha").default(100),
            StateLifecycle::score("z_timer").default(0).auto_tick(),
        ])
        .unwrap();

        assert_eq!(
            output.load_commands,
            vec![
                "scoreboard objectives add alpha dummy",
                "scoreboard objectives add z_timer dummy",
            ]
        );
        assert_eq!(
            output.init_commands,
            vec![
                "execute unless score @s alpha matches -2147483648.. run scoreboard players set @s alpha 100",
                "execute unless score @s z_timer matches -2147483648.. run scoreboard players set @s z_timer 0",
            ]
        );
        assert_eq!(
            output.tick_commands,
            vec![
                "execute if score @s z_timer matches 1.. run scoreboard players remove @s z_timer 1"
            ]
        );
    }

    #[test]
    fn automatic_conflicts_report_all_lifecycle_options() {
        let err = automatic_lifecycle_from([
            StateLifecycle::score("mana").default(100),
            StateLifecycle::score("mana").default(0),
        ])
        .unwrap_err();
        assert!(err.contains("conflicting automatic state `mana`"));
        assert!(err.contains("default Some(100)"));
        assert!(err.contains("default Some(0)"));
    }
}
