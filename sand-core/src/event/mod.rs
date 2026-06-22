//! Typed event model — strongly-typed advancement-backed event framework.
//!
//! # Core types
//!
//! | Type | Purpose |
//! |---|---|
//! | [`AdvancementEvent`] | Trait for events backed by an advancement trigger |
//! | [`Event`] | Zero-cost handler context passed to `#[event]` handlers |
//! | [`EventId`] | Controls how the advancement ID is determined |
//! | [`EventReset`] | Controls re-arming after firing |
//! | [`EventVisibility`] | Controls toast/chat visibility |
//! | [`IntoEventAdvancement`] | Extension: builds the full advancement from an event |

pub mod handle;
pub mod trigger;
pub mod vanilla;

use crate::AdvancementTrigger;
use std::marker::PhantomData;

// ── Configuration enums ─────────────────────────────────────────────────────

/// Controls how the advancement's resource-location ID is determined.
#[derive(Clone, Debug)]
pub enum EventId {
    /// Auto-generate from the event handler function path.
    Auto,
    /// Use an explicit `namespace:path` resource location.
    Explicit(&'static str),
}

impl EventId {
    /// Resolve to a full `namespace:path` string.
    pub fn resolve(&self, namespace: &str, path: &str) -> String {
        match self {
            EventId::Auto => format!("{namespace}:{path}"),
            EventId::Explicit(s) => s.to_string(),
        }
    }
}

/// Controls whether the event re-arms itself after firing.
/// Controls when a fired advancement-backed event re-arms itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventReset {
    /// Revoke the advancement immediately after firing so it can trigger again
    /// on the next game tick when the condition is met.  This is the default
    /// for most repeating events (e.g. consuming an item on any occasion).
    AfterFire,
    /// Fire once per player, ever — the advancement is never revoked.
    /// Use for permanent progression milestones (e.g. first join).
    OncePerPlayer,
    /// No automatic revocation.  The pack is responsible for revoking the
    /// advancement manually (e.g. via `EventHandle::revoke()`), typically as
    /// part of a session lifecycle or a cool-down system.
    Manual,

    // ── Backward-compatible aliases ──────────────────────────────────────────
    /// Alias for [`AfterFire`](EventReset::AfterFire).
    Auto,
    /// Alias for [`AfterFire`](EventReset::AfterFire).
    Revoke,
    /// Alias for [`OncePerPlayer`](EventReset::OncePerPlayer).
    Once,
}

impl EventReset {
    /// Whether the export pipeline should prepend an `advancement revoke` line.
    pub fn should_revoke(&self) -> bool {
        match self {
            EventReset::AfterFire | EventReset::Auto | EventReset::Revoke => true,
            EventReset::OncePerPlayer | EventReset::Once | EventReset::Manual => false,
        }
    }
}

/// Controls the advancement toast and chat message visibility.
#[derive(Clone, Debug)]
pub enum EventVisibility {
    /// No toast, no chat message — fully silent.
    Hidden,
    /// Show an advancement toast only.
    Toast,
    /// Show both toast and chat message.
    Chat,
}

// ── AdvancementEvent trait ──────────────────────────────────────────────────

/// Marker trait for events backed by a Minecraft advancement trigger.
///
/// Implement this on your event type to define how it fires, how its
/// advancement ID is derived, whether it re-arms, and any typed guard
/// condition. Handle the event with `#[event] fn handler(event: Event<T>)`.
///
/// # Example
///
/// ```rust,ignore
/// use sand_core::event::{AdvancementEvent, EventId, EventReset, EventVisibility};
/// use sand_core::AdvancementTrigger;
///
/// pub struct DrankHoney;
///
/// impl AdvancementEvent for DrankHoney {
///     type Trigger = AdvancementTrigger;
///     fn trigger() -> Self::Trigger {
///         AdvancementTrigger::ConsumeItem { item: None }
///     }
///     fn id() -> EventId { EventId::Auto }
///     fn reset() -> EventReset { EventReset::AfterFire }
///     fn visibility() -> EventVisibility { EventVisibility::Hidden }
/// }
/// ```
pub trait AdvancementEvent {
    /// The trigger type for this event — must convert into [`AdvancementTrigger`].
    type Trigger: Into<AdvancementTrigger>;

    /// The trigger instance that Minecraft watches for.
    fn trigger() -> Self::Trigger;

    /// How to determine the advancement ID.
    fn id() -> EventId {
        EventId::Auto
    }

    /// Whether to revoke the advancement after firing.
    ///
    /// Default is [`EventReset::AfterFire`] — the advancement is revoked
    /// immediately so the trigger can re-arm each time the condition is met.
    /// Override with [`EventReset::OncePerPlayer`] for one-shot milestones or
    /// [`EventReset::Manual`] to manage lifecycle via [`EventHandle`](crate::EventHandle).
    fn reset() -> EventReset {
        EventReset::AfterFire
    }

    /// The advancement's display visibility.
    fn visibility() -> EventVisibility {
        EventVisibility::Hidden
    }

    /// An optional extra condition checked before the handler runs.
    ///
    /// When `Some(condition)` is returned, the handler function starts with
    /// `execute unless <condition> run return 0`, skipping execution when the
    /// condition is not met.
    ///
    /// Useful for adding score-based or entity-based guards beyond what the
    /// advancement trigger itself provides.
    fn guard() -> Option<crate::condition::Condition> {
        None
    }
}

/// Legacy compatibility trait — provides `player()` for bare event-type handler
/// parameters (e.g. `event: OnJoinEvent`).
///
/// The primary event model is `Event<T>` where `T: AdvancementEvent`, which
/// gives you `event.player()` directly:
///
/// ```rust,ignore
/// #[event]
/// pub fn on_kill(event: Event<EntityKillEvent>) {
///     cmd::tellraw(event.player(), Text::new("Killed!"));
/// }
/// ```
///
/// `EventPlayer` is implemented on all built-in event marker types so that
/// legacy bare-parameter handlers compiled before the `Event<T>` model are
/// still accepted by the `#[event]` macro. Prefer `Event<T>` for new code.
pub trait EventPlayer {
    /// Returns `Selector::self_()` — the player who triggered the event.
    fn player(&self) -> crate::cmd::Selector {
        crate::cmd::Selector::self_()
    }
}

// ── Event<E> — handler context ───────────────────────────────────────────────

/// Zero-cost handler context for `#[event]`-annotated functions.
///
/// Inside an `#[event]` handler, the generated code creates an `Event<E>`
/// value that gives you access to context methods like [`player()`]. You
/// never construct `Event<E>` manually — the `#[event]` macro generates it.
///
/// ```rust,ignore
/// use sand_macros::event;
/// use sand_core::prelude::*;
///
/// pub struct AteGoldenApple;
/// impl AdvancementEvent for AteGoldenApple { /* … */ }
///
/// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
///
/// #[event]
/// pub fn ate_golden_apple(event: Event<AteGoldenApple>) {
///     MANA.add(event.player(), 25);
/// }
/// ```
pub struct Event<E: AdvancementEvent> {
    _marker: PhantomData<E>,
}

impl<E: AdvancementEvent> Event<E> {
    /// Construct the handler context value.
    ///
    /// Called by `#[event]`-generated code. Not normally called directly.
    pub fn context() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Returns `Selector::self_()` — the player who triggered the event.
    ///
    /// In advancement-backed events, `@s` is the player at the time the
    /// advancement reward function ran.
    pub fn player(&self) -> crate::cmd::Selector {
        crate::cmd::Selector::self_()
    }

    /// Returns `Selector::self_()` — alias for [`player`](Event::player).
    pub fn subject(&self) -> crate::cmd::Selector {
        crate::cmd::Selector::self_()
    }
}

impl<E: AdvancementEvent> Default for Event<E> {
    fn default() -> Self {
        Self::context()
    }
}

// ── EventAdvancement<E> — internal advancement builder ───────────────────────

/// Internal advancement component builder for `AdvancementEvent`-backed events.
///
/// Users should not construct this directly. The `#[event]` macro and the
/// export pipeline use this to build the final `Advancement` JSON.
///
/// # Migration note
///
/// Code that previously called `Event::<E>::new("ns:path", "ns:handler")` should
/// be updated to use this type instead. The old `Event<E>` builder API is gone
/// — `Event<E>` is now the zero-cost handler context.
pub struct EventAdvancement<E: AdvancementEvent> {
    /// The advancement resource location.
    pub advancement_id: String,
    /// The handler function reference for `rewards.function`.
    pub handler_function: String,
    _marker: PhantomData<E>,
}

impl<E: AdvancementEvent> EventAdvancement<E> {
    /// Create a new typed event advancement with the given IDs.
    ///
    /// - `advancement_id` — full `namespace:path` for the generated advancement
    /// - `handler_function` — full `namespace:path` for the mcfunction to run
    pub fn new(advancement_id: impl Into<String>, handler_function: impl Into<String>) -> Self {
        Self {
            advancement_id: advancement_id.into(),
            handler_function: handler_function.into(),
            _marker: PhantomData,
        }
    }

    /// Build the [`Advancement`](crate::Advancement) component.
    pub fn into_advancement(self) -> crate::Advancement {
        let trigger = E::trigger().into();
        let rl: crate::ResourceLocation = self
            .advancement_id
            .parse()
            .expect("invalid advancement resource location in EventAdvancement::new");

        crate::Advancement::new(rl)
            .criterion("event", crate::Criterion::new(trigger))
            .rewards(crate::AdvancementRewards::new().function(self.handler_function))
    }
}

/// Converts a typed [`EventAdvancement`] into a [`sand_components::Advancement`] component.
///
/// Kept for backward compatibility with code that used the old `Event<E>` builder.
pub trait IntoEventAdvancement<E: AdvancementEvent> {
    /// Build the advancement component from this event.
    fn into_advancement(self) -> crate::Advancement;
}

impl<E: AdvancementEvent> IntoEventAdvancement<E> for EventAdvancement<E> {
    fn into_advancement(self) -> crate::Advancement {
        self.into_advancement()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatapackComponent;

    struct TestTickEvent;

    impl AdvancementEvent for TestTickEvent {
        type Trigger = AdvancementTrigger;
        fn trigger() -> Self::Trigger {
            AdvancementTrigger::Tick
        }
    }

    #[test]
    fn event_context_player_returns_self() {
        let event = Event::<TestTickEvent>::context();
        let sel = event.player();
        assert_eq!(sel.to_string(), "@s");
    }

    #[test]
    fn event_context_subject_alias() {
        let event = Event::<TestTickEvent>::default();
        let sel = event.subject();
        assert_eq!(sel.to_string(), "@s");
    }

    #[test]
    fn event_advancement_builds() {
        let ea =
            EventAdvancement::<TestTickEvent>::new("test_pack:test_event", "test_pack:on_test");
        let adv = ea.into_advancement();
        assert_eq!(adv.resource_location().to_string(), "test_pack:test_event");
    }

    #[test]
    fn event_id_auto_resolves() {
        assert_eq!(
            EventId::Auto.resolve("my_pack", "on_join"),
            "my_pack:on_join"
        );
    }

    #[test]
    fn event_id_explicit() {
        assert_eq!(
            EventId::Explicit("custom:override").resolve("my_pack", "on_join"),
            "custom:override"
        );
    }

    #[test]
    fn event_reset_defaults_to_revoke() {
        assert!(EventReset::AfterFire.should_revoke());
        assert!(EventReset::Auto.should_revoke(), "backward-compat alias");
    }

    #[test]
    fn event_reset_once_does_not_revoke() {
        assert!(!EventReset::OncePerPlayer.should_revoke());
        assert!(!EventReset::Once.should_revoke(), "backward-compat alias");
        assert!(!EventReset::Manual.should_revoke());
    }
}
