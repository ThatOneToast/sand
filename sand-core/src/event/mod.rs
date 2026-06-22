//! Typed event model — strongly-typed advancement-backed event framework.
//!
//! # Core types
//!
//! | Type | Purpose |
//! |---|---|
//! | [`AdvancementEvent`] | Trait for events backed by an advancement trigger |
//! | [`Event`] | Strongly-typed advancement component builder |
//! | [`EventId`] | Controls how the advancement ID is determined |
//! | [`EventReset`] | Controls re-arming after firing |
//! | [`EventVisibility`] | Controls toast/chat visibility |
//! | [`IntoEventAdvancement`] | Extension: builds the full advancement from an event |

pub mod trigger;

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
#[derive(Clone, Debug)]
pub enum EventReset {
    /// Use the trait's default behavior.
    Auto,
    /// Always revoke the advancement after firing (re-arm for next trigger).
    Revoke,
    /// Fire only once per player, ever (no revocation).
    Once,
}

impl EventReset {
    /// Whether the advancement should be revoked after the handler runs.
    pub fn should_revoke(&self) -> bool {
        match self {
            EventReset::Auto => true,
            EventReset::Revoke => true,
            EventReset::Once => false,
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
/// advancement ID is derived, and whether it re-arms.
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
///     fn reset() -> EventReset { EventReset::Auto }
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
    fn reset() -> EventReset {
        EventReset::Auto
    }

    /// The advancement's display visibility.
    fn visibility() -> EventVisibility {
        EventVisibility::Hidden
    }
}

// ── Event<E> builder ────────────────────────────────────────────────────────

/// Strongly-typed advancement-backed event builder.
///
/// `Event<E>` constructs a [`sand_components::Advancement`] whose trigger,
/// ID, reset, and visibility are all determined at compile time by the
/// [`AdvancementEvent`] implementation for `E`.
///
/// # Example
///
/// ```rust,ignore
/// use sand_core::event::{AdvancementEvent, Event};
/// use sand_core::AdvancementTrigger;
///
/// pub struct DrankHoney;
/// impl AdvancementEvent for DrankHoney {
///     type Trigger = AdvancementTrigger;
///     fn trigger() -> Self::Trigger {
///         AdvancementTrigger::ConsumeItem { item: None }
///     }
/// }
///
/// let event = Event::<DrankHoney>::new(
///     "my_pack:drank_honey",
///     "my_pack:handler_drank_honey",
/// );
/// let advancement = event.into_advancement();
/// ```
pub struct Event<E: AdvancementEvent> {
    /// The advancement resource location.
    advancement_id: String,
    /// The handler function reference for `rewards.function`.
    handler_function: String,
    _marker: PhantomData<E>,
}

impl<E: AdvancementEvent> Event<E> {
    /// Create a new typed event with the given advancement ID and handler function.
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
}

/// Converts a typed [`Event`] into a [`sand_components::Advancement`] component.
pub trait IntoEventAdvancement<E: AdvancementEvent> {
    /// Build the advancement component from this event.
    fn into_advancement(self) -> crate::Advancement;
}

impl<E: AdvancementEvent> IntoEventAdvancement<E> for Event<E> {
    fn into_advancement(self) -> crate::Advancement {
        let trigger = E::trigger().into();
        let rl: crate::ResourceLocation = self
            .advancement_id
            .parse()
            .expect("invalid advancement resource location in Event::new");

        crate::Advancement::new(rl)
            .criterion("event", crate::Criterion::new(trigger))
            .rewards(crate::AdvancementRewards::new().function(self.handler_function))
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
    fn event_builds_advancement() {
        let event = Event::<TestTickEvent>::new("test_pack:test_event", "test_pack:on_test_event");
        let adv = event.into_advancement();
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
        assert!(EventReset::Auto.should_revoke());
    }

    #[test]
    fn event_reset_once_does_not_revoke() {
        assert!(!EventReset::Once.should_revoke());
    }
}
