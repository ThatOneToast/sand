#![allow(clippy::result_large_err)]
//! Typed event model — strongly-typed advancement-backed event framework.
//!
//! # Core types
//!
//! | Type | Purpose |
//! |---|---|
//! | [`AdvancementEvent`] | Trait for events backed by an advancement trigger |
//! | [`Event`] | Zero-cost handler context passed to `#[on_event]` handlers |
//! | [`EventId`] | Controls how the advancement ID is determined |
//! | [`EventReset`] | Controls re-arming after firing |
//! | [`EventVisibility`] | Controls toast/chat visibility |

pub mod handle;
pub mod trigger;
pub mod vanilla;

use crate::AdvancementTrigger;
use std::marker::PhantomData;

// ── Configuration enums ─────────────────────────────────────────────────────

/// Converts a value into a validated event/advancement [`ResourceLocation`](crate::ResourceLocation).
///
/// Mirrors [`crate::function::IntoFunctionRef`]'s conversion table: a typed
/// [`ResourceLocation`](crate::ResourceLocation) value passes through unchanged (already validated at
/// construction), while raw `&str`/`String` values are parsed and validated
/// here, panicking with an actionable diagnostic on malformed input.
///
/// This makes [`ResourceLocation`](crate::ResourceLocation) the preferred,
/// pre-validated path for explicit event identities.
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
#[sand_macros::api(registry = sand_api_contract, path = "sand::event::EventId", aliases = ["sand::prelude::EventId"], summary = "Chooses the resource location used for a custom advancement-backed event.", context = "An explicit ID gives a handler a stable Minecraft resource name; Auto keeps the name aligned with the generated handler path.", minecraft = "Becomes the advancement JSON resource location and the target used by generated revoke commands.", use_when = ["Overriding an event's generated advancement ID", "Validating a resource location before export"], avoid_when = ["Naming an ordinary function or component"], variants(Auto = "Derives the ID from the generated event handler path.", Explicit = "Uses the supplied validated Minecraft resource location."), variant_fields(Explicit = ["The validated namespace:path identifier."]), example = "let id = EventId::try_explicit(\"demo:events/join\")?;")]
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
    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::event::EventId::explicit", summary = "Builds an explicit event ID, validating raw resource-location text.", context = "This is the convenient constructor when an event must retain a chosen Minecraft advancement name.", minecraft = "The resulting namespace:path becomes the generated advancement resource ID.", use_when = ["A static event needs a deliberate advancement identifier"], avoid_when = ["The generated handler path is already the intended ID"], params(id = "A validated resource location or raw namespace:path text."), returns = "An explicit event ID, or panics for malformed raw text.", example = "let id = EventId::explicit(\"demo:events/join\");")]
    pub fn explicit(id: impl IntoEventId) -> Self {
        Self::Explicit(id.into_event_resource_location())
    }

    /// Fallible explicit event ID constructor.
    ///
    /// Returns `Err` instead of panicking when `id` is not a valid
    /// `namespace:path` resource location.
    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::event::EventId::try_explicit", summary = "Fallibly validates a requested event resource location.", context = "Use this at a configuration boundary that must return an error rather than panic on external text.", minecraft = "Rejects names that Minecraft cannot use as a namespace:path advancement ID.", use_when = ["Parsing configurable event IDs"], avoid_when = ["Using a compile-time known valid resource location"], params(id = "Text expected to contain a namespace:path resource location."), returns = "An explicit event ID or Sand's resource-location validation error.", example = "let id = EventId::try_explicit(\"demo:events/join\")?;")]
    pub fn try_explicit(id: impl AsRef<str>) -> Result<Self, sand_components::SandError> {
        id.as_ref()
            .parse::<crate::ResourceLocation>()
            .map(Self::Explicit)
    }

    /// Resolve to a full `namespace:path` string.
    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::event::EventId::resolve", summary = "Resolves an automatic or explicit ID to namespace:path text.", context = "Export wiring uses this to make the automatic path explicit before generating advancement resources.", minecraft = "Returns the exact resource location written into generated advancement and command references.", use_when = ["Implementing a custom event export adapter"], avoid_when = ["Ordinary #[on_event] authoring, where Sand resolves the ID"], params(namespace = "Namespace used only when this ID is Auto.", path = "Generated path used only when this ID is Auto."), returns = "The resolved namespace:path identifier.", example = "assert_eq!(EventId::Auto.resolve(\"demo\", \"events/join\"), \"demo:events/join\");")]
    pub fn resolve(&self, namespace: &str, path: &str) -> String {
        match self {
            EventId::Auto => format!("{namespace}:{path}"),
            EventId::Explicit(rl) => rl.to_string(),
        }
    }
}

/// Controls whether the event re-arms itself after firing.
/// Controls when a fired advancement-backed event re-arms itself.
#[sand_macros::api(registry = sand_api_contract, path = "sand::event::EventReset", aliases = ["sand::prelude::EventReset"], summary = "Controls whether an advancement-backed event re-arms after it dispatches.", context = "The reset policy prevents repeating triggers from becoming permanently granted while allowing genuine per-player milestones.", minecraft = "AfterFire emits an advancement revoke for the triggering player; the other choices leave grant state intact.", use_when = ["Defining an AdvancementEvent with a non-default lifecycle"], avoid_when = ["Controlling a tick-polled SandEvent, which has no advancement grant to revoke"], variants(AfterFire = "Revokes immediately so a later matching action can fire again.", OncePerPlayer = "Leaves the advancement granted permanently after its first dispatch.", Manual = "Leaves lifecycle management to explicit advancement commands."), example = "fn reset() -> EventReset { EventReset::OncePerPlayer }")]
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
}

impl EventReset {
    /// Whether the export pipeline should prepend an `advancement revoke` line.
    #[sand_macros::api(kind = "method", registry = sand_api_contract, path = "sand::event::EventReset::should_revoke", summary = "Reports whether this policy emits an immediate re-arm revoke.", context = "This is export-facing lifecycle inspection; most event authors choose a variant and let Sand apply it.", minecraft = "True means the generated reward path revokes the triggering player's advancement after dispatch.", use_when = ["Writing a custom export adapter"], avoid_when = ["Deciding normal event behavior; select the policy variant directly"], returns = "Whether the advancement should be revoked after firing.", example = "assert!(EventReset::AfterFire.should_revoke());")]
    pub fn should_revoke(&self) -> bool {
        match self {
            EventReset::AfterFire => true,
            EventReset::OncePerPlayer | EventReset::Manual => false,
        }
    }
}

/// Controls the advancement toast and chat message visibility.
#[sand_macros::api(registry = sand_api_contract, path = "sand::event::EventVisibility", aliases = ["sand::prelude::EventVisibility"], summary = "Describes the intended player-facing announcement level for an advancement-backed event.", context = "Event definitions retain this policy so the event model can express whether an occurrence should surface an advancement-style announcement.", minecraft = "Maps to the advancement display visibility chosen by Sand's event export path.", use_when = ["Defining a custom AdvancementEvent's display policy"], avoid_when = ["Sending a bespoke message from the handler; use a command or text component"], variants(Hidden = "Suppresses advancement toast and chat output.", Toast = "Requests an advancement toast without chat.", Chat = "Requests both advancement toast and chat announcement."), example = "fn visibility() -> EventVisibility { EventVisibility::Hidden }")]
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
/// condition. Handle the event with `#[on_event] fn handler(event: Event<T>)`.
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
    /// `#[datapack_component(Load)]` function — can call `Event::<Self>::state_init()` to
    /// collect these commands without knowing the concrete types.
    ///
    /// [`ScoreVar`]: crate::state::ScoreVar
    /// [`Flag`]: crate::state::Flag
    /// [`Cooldown`]: crate::state::Cooldown
    /// [`Timer`]: crate::state::Timer
    fn state_defines() -> Vec<String> {
        vec![]
    }

    /// Which participant observations this event declares (#230).
    ///
    /// Defaults to [`crate::participant::EventParticipantPlan::none`] — a
    /// genuinely additive default; every existing `AdvancementEvent`
    /// implementation is unaffected. Unlike [`crate::events::SandEvent::participants`]
    /// (its tick-dispatch counterpart), a declared plan here **is**
    /// automatically applied by the export pipeline: an advancement-backed
    /// handler has no separate `EventSetup` pre/post-observation phase (its
    /// generated body *is* the entire handler), so the compiler splices the
    /// plan's setup commands at the start of the generated body and its
    /// cleanup commands at the end — see
    /// `sand-core/src/compiler/export/pipeline.rs`'s `EventDispatch::Advancement`
    /// handling. Authors do not need to call
    /// [`crate::events::EventSetup::with_participants`] themselves for this
    /// dispatch kind.
    fn participants() -> crate::participant::EventParticipantPlan {
        crate::participant::EventParticipantPlan::none()
    }
}

/// Capability marker for advancement events that represent player damage.
///
/// Vanilla advancement reward functions identify the triggering player as
/// `@s`, but they do not provide exact damage amount to the reward function.
/// Use [`DamageAmount::Fixed`](sand_commands::DamageAmount::Fixed) today, or
/// add a real tracking system before using same-as-event damage.
pub trait DamageAdvancementEvent: AdvancementEvent {}

// ── Event<E> — handler context ───────────────────────────────────────────────

/// Zero-cost runtime context for `#[on_event]`-annotated advancement handlers.
///
/// Inside an `#[on_event]` handler, the generated code creates an `Event<E>`
/// value that gives you access to context methods like [`Event::player`]. It is
/// shared by advancement-backed and generated tracked events. You never
/// construct `Event<E>` manually — the `#[on_event]` macro generates it.
/// The context contains no instance of `E`; ordinary fields on the marker type
/// are not captured Minecraft values. Event-time data must come from context
/// handles explicitly provided by Sand or from typed state queried in the
/// handler.
///
/// ```rust,ignore
/// use sand_macros::on_event;
/// use sand_core::prelude::*;
///
/// pub struct AteGoldenApple;
/// impl AdvancementEvent for AteGoldenApple { /* … */ }
///
/// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
///
/// #[on_event]
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
    /// Called by `#[on_event]`-generated code. Not normally called directly.
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
    /// Call this in your `#[datapack_component(Load)]` function so all objectives exist
    /// before the event fires:
    ///
    /// ```rust,ignore
    /// #[datapack_component(Load)]
    /// fn load() {
    ///     for cmd in Event::<DrinkManaEvent>::state_init() {
    ///         cmd::raw(cmd);
    ///     }
    /// }
    /// ```
    pub fn state_init() -> Vec<String> {
        E::state_defines()
    }

    /// Access a declared entity participant by role (#230, infallible per
    /// #273).
    ///
    /// Backed by whatever [`AdvancementEvent::participants`] declared for
    /// this event type. Returns the typed participant directly — not
    /// wrapped in `Result`/`Option`/[`crate::participant::ParticipantAvailability`]
    /// — since a role this event does not declare is a build-time authoring
    /// mistake (`sand build`'s mandatory graph validation is expected to
    /// catch it before output is written), not a value for ordinary handler
    /// code to branch on. See `EventParticipantPlan::require_entity` (crate-private)
    /// for the exact reconstruction and panic contract.
    ///
    /// ```rust,ignore
    /// #[on_event]
    /// fn on_hit(event: Event<EntityDamagePlayerEvent>) {
    ///     let attacker = event.entity(EntityParticipantRole::Attacker);
    ///     // build commands against attacker.selector()
    /// }
    /// ```
    pub fn entity(
        &self,
        role: crate::participant::EntityParticipantRole,
    ) -> crate::participant::EntityParticipant {
        E::participants().require_entity(std::any::type_name::<E>(), role)
    }

    /// Access a declared item participant by role (#230, infallible per
    /// #273). See [`Self::entity`] for the contract this mirrors.
    pub fn item(&self, role: crate::participant::ItemParticipantRole) -> crate::item::ItemSnapshot {
        E::participants().require_item(std::any::type_name::<E>(), role)
    }

    /// The entity that caused this event, when declared. `DirectAttacker`
    /// (the immediate causing entity, e.g. an arrow rather than the player
    /// who shot it) is a distinct role vanilla's damage source also draws,
    /// but no credible backend exists for it today — see
    /// `docs/testing/participant-role-evidence.md`.
    pub fn attacker(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::Attacker)
    }

    /// The entity that landed the killing blow, when declared.
    pub fn killer(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::Killer)
    }

    /// The entity that received damage/an effect, when declared.
    pub fn victim(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::Victim)
    }

    /// The entity this player directly interacted with, when declared.
    pub fn interacted_entity(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::InteractedEntity)
    }

    /// The weapon item snapshot, when declared — see
    /// [`crate::participant::EventParticipantPlan::observe_weapon`].
    pub fn weapon(&self) -> crate::item::ItemSnapshot {
        self.item(crate::participant::ItemParticipantRole::Weapon)
    }

    /// Access a declared bounded item participant by role (#272, infallible
    /// per #273) — the `.within(...)`-crossing counterpart to [`Self::item`].
    /// See [`crate::participant::EventParticipantPlan::inherit_item_within`]
    /// for the full contract.
    pub fn bounded_item(
        &self,
        role: crate::participant::ItemParticipantRole,
    ) -> crate::participant::BoundedItemSnapshot {
        E::participants().require_bounded_item(std::any::type_name::<E>(), role)
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

/// Damage-specific event handler context for `#[on_event]` functions.
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
    /// Called by `#[on_event]`-generated code. Not normally called directly.
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
