//! Runtime event handle API for enabling/disabling/resetting events.

use crate::condition::{Condition, ScoreRange};
use std::marker::PhantomData;
use std::sync::OnceLock;

/// Runtime handle for enabling, disabling, and resetting an advancement-backed event.
///
/// The generic parameter `E` is a marker that binds the handle to a specific
/// event type. No trait bound is required — the objective name is derived lazily
/// from `E`'s fully-qualified Rust type name via [`std::any::type_name`].
///
/// # Example
///
/// ```rust,ignore
/// use sand_core::event::handle::EventHandle;
///
/// static GOLDEN_APPLE: EventHandle<AteGoldenAppleEvent> = EventHandle::new();
///
/// #[component(Load)]
/// pub fn load() {
///     GOLDEN_APPLE.define();
/// }
///
/// #[event]
/// pub fn on_death(_: OnDeath) {
///     GOLDEN_APPLE.disable("@s");
/// }
/// ```
///
/// # Objective naming
///
/// The scoreboard objective is `__ev_<8-hex-chars>` where the hash input is
/// the fully-qualified Rust type name of `E` (e.g.
/// `arcane_pack::events::AteGoldenAppleEvent`).  This is stable within a
/// compilation but may change if the type is moved to a different module.
/// When migrating from [`RawEventHandle`], issue a one-time scoreboard rename
/// in your load function.
pub struct EventHandle<E> {
    /// Lazily-initialised objective name (computed from `type_name::<E>()`).
    objective: OnceLock<String>,
    /// Variance: `fn() -> E` keeps the handle `Sync` even for non-`Sync` `E`,
    /// since no `E` value is ever stored.
    _marker: PhantomData<fn() -> E>,
}

impl<E> EventHandle<E> {
    /// Create a typed event handle bound to event type `E`.
    ///
    /// The scoreboard objective name is derived from the Rust type name of `E`
    /// the first time any method on this handle is called — no string required.
    pub const fn new() -> Self {
        Self {
            objective: OnceLock::new(),
            _marker: PhantomData,
        }
    }

    /// `scoreboard objectives add <obj> dummy` — register the objective.
    ///
    /// Call this in your `#[component(Load)]` function.
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    /// Build a [`Condition`] that checks whether this event is enabled for `@s`.
    ///
    /// Inject into your event's `guard()` implementation to honour the handle.
    ///
    /// ```rust,ignore
    /// fn guard() -> Option<Condition> {
    ///     Some(GOLDEN_APPLE.condition().and(MANA.of("@s").lt(100)))
    /// }
    /// ```
    pub fn condition(&self) -> Condition {
        Condition::Score {
            selector: "@s".into(),
            objective: self.objective_name().to_string(),
            range: ScoreRange::Eq(1),
        }
    }

    /// Command to enable this event for the given selector.
    pub fn enable(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {selector} {} 1",
            self.objective_name()
        )
    }

    /// Command to disable this event for the given selector.
    pub fn disable(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {selector} {} 0",
            self.objective_name()
        )
    }

    /// `advancement revoke <selector> only <advancement_id>` — re-arm the trigger.
    ///
    /// Call this to allow the advancement to fire again after it has been granted.
    pub fn reset(&self, advancement_id: &str, selector: impl std::fmt::Display) -> String {
        format!("advancement revoke {selector} only {advancement_id}")
    }

    /// `advancement grant <selector> only <advancement_id>` — manually fire the trigger.
    pub fn grant(&self, advancement_id: &str, selector: impl std::fmt::Display) -> String {
        format!("advancement grant {selector} only {advancement_id}")
    }

    pub(crate) fn objective_name(&self) -> &str {
        self.objective.get_or_init(|| {
            let h = stable_hash(std::any::type_name::<E>());
            format!("__ev_{h}")
        })
    }
}

impl<E> Default for EventHandle<E> {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: EventHandle<E> never stores an E value; the OnceLock<String> is
// inherently Sync.  The PhantomData<fn() -> E> is Sync regardless of E.
unsafe impl<E> Sync for EventHandle<E> {}

// ── Backward-compat: stringly-typed handle ────────────────────────────────────

/// Stringly-typed event handle — prefer [`EventHandle<E>`] for new code.
///
/// Accepts an explicit string key used to derive the objective name.  Useful
/// when the event type isn't in scope or when migrating existing packs.
///
/// ```rust,ignore
/// static MY_HANDLE: RawEventHandle = RawEventHandle::new("my_pack:on_kill");
/// ```
pub struct RawEventHandle {
    event_key: &'static str,
    objective: OnceLock<String>,
}

impl RawEventHandle {
    pub const fn new(event_key: &'static str) -> Self {
        Self {
            event_key,
            objective: OnceLock::new(),
        }
    }

    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    pub fn condition(&self) -> Condition {
        Condition::Score {
            selector: "@s".into(),
            objective: self.objective_name().to_string(),
            range: ScoreRange::Eq(1),
        }
    }

    pub fn enable(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {selector} {} 1",
            self.objective_name()
        )
    }

    pub fn disable(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {selector} {} 0",
            self.objective_name()
        )
    }

    pub fn reset(&self, advancement_id: &str, selector: impl std::fmt::Display) -> String {
        format!("advancement revoke {selector} only {advancement_id}")
    }

    pub fn grant(&self, advancement_id: &str, selector: impl std::fmt::Display) -> String {
        format!("advancement grant {selector} only {advancement_id}")
    }

    fn objective_name(&self) -> &str {
        self.objective.get_or_init(|| {
            let h = stable_hash(self.event_key);
            format!("__ev_{h}")
        })
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Stable FNV-1a 64-bit hash, first 8 hex chars.
fn stable_hash(s: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let prime: u64 = 0x0000_0100_0000_01b3;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(prime);
    }
    format!("{hash:016x}")[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeEventA;
    struct FakeEventB;

    #[test]
    fn typed_handle_define_emits_objective() {
        let handle: EventHandle<FakeEventA> = EventHandle::new();
        let cmd = handle.define();
        assert!(cmd.starts_with("scoreboard objectives add __ev_"), "{cmd}");
    }

    #[test]
    fn typed_handle_enable_disable() {
        let handle: EventHandle<FakeEventA> = EventHandle::new();
        let enable = handle.enable("@s");
        let disable = handle.disable("@s");
        assert!(enable.contains("@s") && enable.ends_with("1"), "{enable}");
        assert!(disable.contains("@s") && disable.ends_with("0"), "{disable}");
        let obj = handle.objective_name().to_string();
        assert!(enable.contains(&obj));
        assert!(disable.contains(&obj));
    }

    #[test]
    fn typed_handle_condition() {
        let handle: EventHandle<FakeEventA> = EventHandle::new();
        let cond = handle.condition();
        let cmd_str = format!("{cond:?}");
        assert!(cmd_str.contains("__ev_"), "{cmd_str}");
    }

    #[test]
    fn raw_handle_backward_compat() {
        let raw = RawEventHandle::new("arcane:on_ate_golden_apple");
        let define = raw.define();
        assert!(define.starts_with("scoreboard objectives add __ev_"), "{define}");
    }

    #[test]
    fn different_event_types_get_different_objectives() {
        let h1: EventHandle<FakeEventA> = EventHandle::new();
        let h2: EventHandle<FakeEventB> = EventHandle::new();
        assert_ne!(h1.objective_name(), h2.objective_name());
    }
}
