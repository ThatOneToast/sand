//! Runtime event handle API for enabling/disabling/resetting events.

use crate::condition::{Condition, ScoreRange};
use std::sync::OnceLock;

/// Runtime handle for enabling, disabling, and resetting an event.
///
/// Each handle owns a per-player scoreboard objective (`__ev_<hash>`) that
/// defaults to `1` (enabled). Setting it to `0` prevents the event handler
/// from running for that player.
///
/// # Example
///
/// ```rust,ignore
/// use sand_core::event::handle::EventHandle;
///
/// static MY_EVENT: EventHandle = EventHandle::new("my_pack:on_kill");
///
/// #[function]
/// pub fn disable_event() {
///     cmd::call(|| Vec::new());
///     // At runtime:
///     let _cmd = MY_EVENT.disable("@s");
/// }
/// ```
pub struct EventHandle {
    event_path: &'static str,
    objective: OnceLock<String>,
}

impl EventHandle {
    /// Create a new event handle for the given event path.
    ///
    /// The `event_path` should match the path used in `#[event]`.
    pub const fn new(event_path: &'static str) -> Self {
        Self {
            event_path,
            objective: OnceLock::new(),
        }
    }

    /// `scoreboard objectives add <obj> dummy` — register the objective.
    ///
    /// Call this in your `load` function.
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    /// Build a [`Condition`] that checks whether this event is enabled for `@s`.
    ///
    /// ```rust,ignore
    /// TypedExecute::as_players()
    ///     .when(MY_EVENT.condition())
    ///     .run(cmd::say("Event is enabled!"));
    /// ```
    pub fn condition(&self) -> Condition {
        Condition::Score {
            selector: "@s".into(),
            objective: self.objective_name().to_string(),
            range: ScoreRange::Eq(1),
        }
    }

    /// Command to enable this event for the given selector.
    pub fn enable(&self, selector: &str) -> String {
        format!(
            "scoreboard players set {sel} {} 1",
            self.objective_name(),
            sel = selector
        )
    }

    /// Command to disable this event for the given selector.
    pub fn disable(&self, selector: &str) -> String {
        format!(
            "scoreboard players set {sel} {} 0",
            self.objective_name(),
            sel = selector
        )
    }

    /// Command to re-arm an advancement-backed event.
    ///
    /// Revokes the advancement so the trigger can grant it again.
    pub fn reset(&self, advancement_id: &str, selector: &str) -> String {
        format!(
            "advancement revoke {sel} only {id}",
            sel = selector,
            id = advancement_id
        )
    }

    /// Command to manually grant the advancement (normally done by the trigger).
    pub fn grant(&self, advancement_id: &str, selector: &str) -> String {
        format!(
            "advancement grant {sel} only {id}",
            sel = selector,
            id = advancement_id
        )
    }

    fn objective_name(&self) -> &str {
        self.objective.get_or_init(|| {
            let h = stable_hash(self.event_path);
            format!("__ev_{h}")
        })
    }
}

/// Stable FNV-1a 64-bit hash, first 8 hex chars.
fn stable_hash(s: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    let prime: u64 = 0x100000001b3;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(prime);
    }
    format!("{hash:016x}")[..8].to_string()
}
