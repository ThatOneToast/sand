#![allow(clippy::result_large_err)]
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

pub mod builder;
pub mod handle;
pub mod trigger;
pub mod vanilla;

pub use builder::{EventBuilder, EventConfig};

use crate::AdvancementTrigger;
use std::marker::PhantomData;

// ── Configuration enums ─────────────────────────────────────────────────────

/// Converts a value into a validated event/advancement [`ResourceLocation`](crate::ResourceLocation).
///
/// Mirrors [`crate::function::IntoFunctionRef`]'s conversion table: a typed
/// [`ResourceLocation`] value passes through unchanged (already validated at
/// construction), while raw `&str`/`String` values are parsed and validated
/// here, panicking with an actionable diagnostic on malformed input.
///
/// This keeps existing string call sites (`EventBuilder::id("my_pack:foo")`,
/// `EventConfig::advancement("my_pack:foo", ...)`) source-compatible while
/// making [`ResourceLocation`] the preferred, pre-validated normal path — see #196.
/// Invalid explicit event IDs are rejected here, at the API boundary, rather
/// than silently passed through to `resolve()`/export.
pub trait IntoEventId {
    /// Resolve to a validated [`ResourceLocation`](crate::ResourceLocation).
    ///
    /// # Panics
    ///
    /// Panics if a raw string value is not a valid `namespace:path` resource
    /// location. Use [`EventId::try_explicit`] for a fallible alternative.
    fn into_event_resource_location(self) -> crate::ResourceLocation;
}

impl IntoEventId for crate::ResourceLocation {
    fn into_event_resource_location(self) -> crate::ResourceLocation {
        self
    }
}

impl IntoEventId for &crate::ResourceLocation {
    fn into_event_resource_location(self) -> crate::ResourceLocation {
        self.clone()
    }
}

impl IntoEventId for &str {
    fn into_event_resource_location(self) -> crate::ResourceLocation {
        self.parse().unwrap_or_else(|_| {
            panic!(
                "invalid event/advancement resource location `{self}`: must be a valid \
                 `namespace:path` resource location (e.g. `my_pack:on_elevator_placed`); \
                 use EventId::try_explicit(...) for a fallible alternative"
            )
        })
    }
}

impl IntoEventId for String {
    fn into_event_resource_location(self) -> crate::ResourceLocation {
        self.as_str().into_event_resource_location()
    }
}

/// Controls how the advancement's resource-location ID is determined.
#[derive(Clone, Debug)]
pub enum EventId {
    /// Auto-generate from the event handler function path.
    Auto,
    /// Use an explicit, validated resource location.
    Explicit(crate::ResourceLocation),
}

impl EventId {
    /// Construct an explicit event ID from a typed [`ResourceLocation`](crate::ResourceLocation)
    /// or a raw string.
    ///
    /// Raw strings are parsed and validated immediately; invalid input panics
    /// with an actionable diagnostic rather than being silently accepted and
    /// only failing later at export/`resolve()` time. Prefer passing an
    /// already-validated `ResourceLocation` when one is available. Use
    /// [`try_explicit`](Self::try_explicit) if you need a non-panicking path.
    pub fn explicit(id: impl IntoEventId) -> Self {
        Self::Explicit(id.into_event_resource_location())
    }

    /// Fallible explicit event ID constructor.
    ///
    /// Returns `Err` instead of panicking when `id` is not a valid
    /// `namespace:path` resource location.
    pub fn try_explicit(id: impl AsRef<str>) -> Result<Self, sand_components::SandError> {
        id.as_ref()
            .parse::<crate::ResourceLocation>()
            .map(Self::Explicit)
    }

    /// Resolve to a full `namespace:path` string.
    pub fn resolve(&self, namespace: &str, path: &str) -> String {
        match self {
            EventId::Auto => format!("{namespace}:{path}"),
            EventId::Explicit(rl) => rl.to_string(),
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

/// Stateless definition trait for events backed by one Minecraft advancement
/// trigger.
///
/// Implement this on your event type to define how it fires, how its
/// advancement ID is derived, whether it re-arms, and any typed guard
/// condition. Handle the event with `#[event] fn handler(event: Event<T>)`.
/// Sand never constructs `T`: fields declared on the definition type are not
/// runtime event data and are not exposed by `Event<T>`.
///
/// # Example
///
/// ```rust,ignore
/// use sand_core::prelude::*;
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
#[diagnostic::on_unimplemented(
    message = "`{Self}` is used as `Event<{Self}>` but does not implement `AdvancementEvent`",
    label = "`Event<T>` requires `T: AdvancementEvent`",
    note = "AdvancementEvent is a stateless marker for a single vanilla advancement trigger; \
            for custom tick-polled or lifecycle-owned dispatch, implement `SandEvent` \
            (sand_core::events::SandEvent) instead"
)]
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

    /// `scoreboard objectives add …` / storage init commands for state variables
    /// this event depends on.
    ///
    /// Override to list every [`ScoreVar`], [`Flag`], [`Cooldown`], or [`Timer`]
    /// the event's handler reads or writes.  The export pipeline — and your
    /// `#[component(Load)]` function — can call `Event::<Self>::state_init()` to
    /// collect these commands without knowing the concrete types.
    ///
    /// [`ScoreVar`]: crate::state::ScoreVar
    /// [`Flag`]: crate::state::Flag
    /// [`Cooldown`]: crate::state::Cooldown
    /// [`Timer`]: crate::state::Timer
    fn state_defines() -> Vec<String> {
        vec![]
    }

    /// Build a value-based [`EventConfig`] from this trait impl.
    ///
    /// This bridges the trait-based and builder-based APIs: you can obtain an
    /// `EventConfig` from any `AdvancementEvent` impl without knowing whether
    /// it was defined via a struct+impl or via [`EventBuilder`].
    fn into_config() -> crate::event::builder::EventConfig {
        crate::event::builder::EventBuilder::new()
            .trigger(Self::trigger().into())
            .reset(Self::reset())
            .visibility(Self::visibility())
            .build()
        // Note: guard and state_defines are not forwarded here because Condition
        // is not Clone. Override into_config() on your event if you need them.
    }
}

/// Capability marker for advancement events that represent player damage.
///
/// Vanilla advancement reward functions identify the triggering player as
/// `@s`, but they do not provide exact damage amount to the reward function.
/// Use [`DamageAmount::Fixed`](sand_commands::DamageAmount::Fixed) today, or
/// add a real tracking system before using same-as-event damage.
pub trait DamageAdvancementEvent: AdvancementEvent {}

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

/// Zero-cost runtime context for `#[event]`-annotated advancement handlers.
///
/// Inside an `#[event]` handler, the generated code creates an `Event<E>`
/// value that gives you access to context methods like [`Event::player`]. It is
/// shared by advancement-backed and generated tracked events. You never
/// construct `Event<E>` manually — the `#[event]` macro generates it.
/// The context contains no instance of `E`; ordinary fields on the marker type
/// are not captured Minecraft values. Event-time data must come from context
/// handles explicitly provided by Sand or from typed state queried in the
/// handler.
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
pub struct Event<E> {
    _marker: PhantomData<E>,
}

impl<E> Event<E> {
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
    /// `@s` is the player selected by the advancement reward or generated
    /// per-player dispatcher.
    pub fn player(&self) -> crate::cmd::Selector {
        crate::cmd::Selector::self_()
    }

    /// Returns `Selector::self_()` — alias for [`player`](Event::player).
    pub fn subject(&self) -> crate::cmd::Selector {
        crate::cmd::Selector::self_()
    }
}

impl<E: AdvancementEvent> Event<E> {
    /// `scoreboard objectives add …` commands for every state variable this
    /// event declared via [`AdvancementEvent::state_defines`].
    ///
    /// Call this in your `#[component(Load)]` function so all objectives exist
    /// before the event fires:
    ///
    /// ```rust,ignore
    /// #[component(Load)]
    /// fn load() {
    ///     for cmd in Event::<DrinkManaEvent>::state_init() {
    ///         cmd::raw(cmd);
    ///     }
    /// }
    /// ```
    pub fn state_init() -> Vec<String> {
        E::state_defines()
    }

    /// Build a value-based [`EventConfig`] from this event's trait impl.
    ///
    /// Convenience wrapper over [`AdvancementEvent::into_config`].
    pub fn config() -> crate::event::builder::EventConfig {
        E::into_config()
    }
}

impl<E> Default for Event<E> {
    fn default() -> Self {
        Self::context()
    }
}

impl<E: DamageAdvancementEvent> Event<E> {
    /// Start a reflected-damage command builder from this event's player.
    pub fn damage(&self) -> sand_commands::Damage {
        sand_commands::Damage::reflect_from(crate::cmd::SingleEntity::self_())
    }
}

/// Damage-specific event handler context for `#[event]` functions.
///
/// Use `DamageEvent<T>` when `T: DamageAdvancementEvent`. It exposes the
/// triggering player as a statically single player/entity target and provides
/// a first-class reflected-damage builder.
pub struct DamageEvent<E: DamageAdvancementEvent> {
    _marker: PhantomData<E>,
}

impl<E: DamageAdvancementEvent> DamageEvent<E> {
    /// Construct the handler context value.
    ///
    /// Called by `#[event]`-generated code. Not normally called directly.
    pub fn context() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Returns `@s` as a single player: the player who triggered the event.
    pub fn player(&self) -> crate::cmd::SinglePlayer {
        crate::cmd::SinglePlayer::self_()
    }

    /// Returns `@s` as a single entity: the damaged subject.
    pub fn subject(&self) -> crate::cmd::SingleEntity {
        crate::cmd::SingleEntity::self_()
    }

    /// Start a reflected-damage builder centered on and sourced from the player.
    pub fn reflect_damage(&self) -> sand_commands::Damage {
        sand_commands::Damage::reflect_from(self.subject())
    }
}

impl<E: DamageAdvancementEvent> Default for DamageEvent<E> {
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
            EventId::explicit("custom:override").resolve("my_pack", "on_join"),
            "custom:override"
        );
    }

    #[test]
    fn event_id_explicit_accepts_typed_resource_location() {
        let rl: crate::ResourceLocation = "custom:override".parse().unwrap();
        assert_eq!(
            EventId::explicit(rl).resolve("my_pack", "on_join"),
            "custom:override"
        );
    }

    #[test]
    #[should_panic(expected = "invalid event/advancement resource location")]
    fn event_id_explicit_panics_on_invalid_string() {
        EventId::explicit("not a valid id!");
    }

    #[test]
    fn event_id_try_explicit_rejects_invalid_id() {
        assert!(EventId::try_explicit("not a valid id!").is_err());
    }

    #[test]
    fn event_id_try_explicit_accepts_valid_id() {
        let id = EventId::try_explicit("custom:override").unwrap();
        assert_eq!(id.resolve("my_pack", "on_join"), "custom:override");
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
