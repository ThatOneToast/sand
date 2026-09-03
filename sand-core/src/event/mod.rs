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
#[sand_macros::api(registry = sand_api_contract, path = "sand::event::IntoEventId", module = "sand::event", summary = "Converts an event ID input into a validated Minecraft resource location.", context = "Sand implements this for ResourceLocation and ordinary text inputs so EventId has one validation boundary.", minecraft = "Validation enforces namespace:path syntax before an advancement resource is generated.", use_when = ["Accepting the same ergonomic inputs as EventId::explicit"], avoid_when = ["Defining an unrelated identifier conversion"], example = "let id = sand::event::EventId::explicit(\"demo:events/join\");")]
pub trait IntoEventId {
    /// Resolve to a validated [`ResourceLocation`](crate::ResourceLocation).
    ///
    /// # Panics
    ///
    /// Panics if a raw string value is not a valid `namespace:path` resource
    /// location. Use [`EventId::try_explicit`] for a fallible alternative.
    ///
    /// ```
    /// use sand::event::IntoEventId;
    ///
    /// let location = "demo:events/join".into_event_resource_location();
    /// assert_eq!(location.to_string(), "demo:events/join");
    /// ```
    #[sand_macros::api(registry = sand_api_contract, path = "sand::event::IntoEventId::into_event_resource_location", module = "sand::event", summary = "Validates and returns this value as an event resource location.", context = "This is the common conversion seam behind explicit event IDs.", minecraft = "The result is valid namespace:path text for an advancement resource.", use_when = ["Implementing a supported EventId input"], avoid_when = ["Formatting unchecked resource names"], returns = "A validated Minecraft resource location.", example = "let location = \"demo:events/join\".into_event_resource_location();")]
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
#[sand_macros::api(registry = sand_api_contract, path = "sand::event::AdvancementEvent", aliases = ["sand::prelude::AdvancementEvent"], module = "sand::event", summary = "Defines a custom event backed by one Minecraft advancement trigger.", context = "Implement this stateless trait for the common custom-event case, then receive Event<Self> in an #[on_event] handler.", minecraft = "Sand exports one advancement criterion and reward function, applying reset, guard, and participant behavior.", use_when = ["A custom event maps directly to one vanilla advancement trigger"], avoid_when = ["The event needs tick polling, composition, or lifecycle setup; implement SandEvent instead"], example = "struct AteApple;\nimpl sand::event::AdvancementEvent for AteApple { type Trigger = sand::component::AdvancementTrigger; fn trigger() -> Self::Trigger { sand::component::AdvancementTrigger::Tick } }")]
pub trait AdvancementEvent {
    /// The trigger type for this event — must convert into [`AdvancementTrigger`].
    #[sand_macros::api(registry = sand_api_contract, path = "sand::event::AdvancementEvent::Trigger", aliases = ["sand::prelude::AdvancementEvent::Trigger"], module = "sand::event", kind = "associated_type", summary = "Names the typed vanilla trigger emitted for an event.", context = "The associated trigger keeps a custom event definition tied to Sand's validated trigger model.", minecraft = "It serializes into the advancement criterion Minecraft watches.", use_when = ["Implementing AdvancementEvent"], avoid_when = ["Handling an existing event"], example = "type Trigger = sand::component::AdvancementTrigger;")]
    type Trigger: Into<AdvancementTrigger>;

    /// The trigger instance that Minecraft watches for.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::event::AdvancementEvent::trigger", aliases = ["sand::prelude::AdvancementEvent::trigger"], module = "sand::event", summary = "Returns the criterion Minecraft watches for this event.", context = "Sand calls it during export to construct advancement JSON.", minecraft = "The result becomes the event advancement's criterion conditions.", use_when = ["Implementing a custom advancement-backed event"], avoid_when = ["Inspecting a built-in event's export internals"], returns = "The event's typed advancement trigger.", example = "fn trigger() -> Self::Trigger { sand::component::AdvancementTrigger::Tick }")]
    fn trigger() -> Self::Trigger;

    /// How to determine the advancement ID.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::event::AdvancementEvent::id", kind = "trait_method", aliases = ["sand::prelude::AdvancementEvent::id"], module = "sand::event", summary = "Selects the generated advancement resource ID.", context = "The default follows the event handler path, avoiding manually synchronized names.", minecraft = "Controls the exported advancement namespace:path and revoke target.", use_when = ["A custom event needs a stable non-default ID"], avoid_when = ["The generated handler path is sufficient"], returns = "The event ID policy, Auto by default.", example = "fn id() -> sand::event::EventId { sand::event::EventId::Auto }")]
    fn id() -> EventId {
        EventId::Auto
    }

    /// Whether to revoke the advancement after firing.
    ///
    /// Default is [`EventReset::AfterFire`] — the advancement is revoked
    /// immediately so the trigger can re-arm each time the condition is met.
    /// Override with [`EventReset::OncePerPlayer`] for one-shot milestones or
    /// [`EventReset::Manual`] to manage lifecycle via [`EventHandle`](crate::EventHandle).
    #[sand_macros::api(registry = sand_api_contract, path = "sand::event::AdvancementEvent::reset", kind = "trait_method", aliases = ["sand::prelude::AdvancementEvent::reset"], module = "sand::event", summary = "Selects how advancement grant state re-arms after this event fires.", context = "The default supports repeating triggers while milestones can retain their grant.", minecraft = "AfterFire emits an advancement revoke for the triggering player.", use_when = ["Making an event one-shot or manually resettable"], avoid_when = ["Defining a tick-polled SandEvent"], returns = "The event reset policy, AfterFire by default.", example = "fn reset() -> sand::event::EventReset { sand::event::EventReset::OncePerPlayer }")]
    fn reset() -> EventReset {
        EventReset::AfterFire
    }

    /// The advancement's display visibility.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::event::AdvancementEvent::visibility", kind = "trait_method", aliases = ["sand::prelude::AdvancementEvent::visibility"], module = "sand::event", summary = "Selects the intended announcement visibility for this event.", context = "The default keeps mechanical event advancements silent.", minecraft = "Carries the display policy into advancement export.", use_when = ["A custom event should request advancement-style visibility"], avoid_when = ["Sending a handler-authored message"], returns = "The event visibility policy, Hidden by default.", example = "fn visibility() -> sand::event::EventVisibility { sand::event::EventVisibility::Hidden }")]
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
    #[sand_macros::api(registry = sand_api_contract, path = "sand::event::AdvancementEvent::guard", kind = "trait_method", aliases = ["sand::prelude::AdvancementEvent::guard"], module = "sand::event", summary = "Adds an optional typed condition checked before the handler runs.", context = "This complements, rather than replaces, the advancement trigger's own criterion conditions.", minecraft = "Sand emits an execute-unless guard that returns before user commands when it fails.", use_when = ["A score, entity, or predicate condition must gate a broad trigger"], avoid_when = ["The trigger itself can express the required criterion"], returns = "A guard condition, or None for no extra guard.", example = "fn guard() -> Option<sand::condition::Condition> { None }")]
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
    #[sand_macros::api(registry = sand_api_contract, path = "sand::event::AdvancementEvent::state_defines", kind = "trait_method", aliases = ["sand::prelude::AdvancementEvent::state_defines"], module = "sand::event", summary = "Lists initialization commands required by event-owned state.", context = "This is the low-level seam for a custom event that owns scoreboard-backed variables.", minecraft = "The commands normally create objectives before the event can fire.", use_when = ["A custom event explicitly owns score or timer setup"], avoid_when = ["Derived State or ordinary load setup owns the state"], returns = "Commands that initialize the event's declared state.", example = "fn state_defines() -> Vec<String> { Vec::new() }")]
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
    #[sand_macros::api(registry = sand_api_contract, path = "sand::event::AdvancementEvent::participants", kind = "trait_method", aliases = ["sand::prelude::AdvancementEvent::participants"], module = "sand::event", summary = "Declares the event-time observations this event can expose.", context = "The plan is applied around generated handler execution, giving context access an explicit evidence and cleanup model.", minecraft = "Sand emits the storage observations and cleanup around the advancement reward function.", use_when = ["A custom event needs typed attacker, weapon, or other participant context"], avoid_when = ["The handler only needs its triggering player"], returns = "The participant observation plan, empty by default.", example = "fn participants() -> sand::participant::EventParticipantPlan { sand::participant::EventParticipantPlan::none() }")]
    fn participants() -> crate::participant::EventParticipantPlan {
        crate::participant::EventParticipantPlan::none()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::DamageAdvancementEvent",
    aliases = ["sand::prelude::DamageAdvancementEvent"],
    module = "sand::event",
    summary = "Capability marker for advancement events that represent player damage.",
    context = "Capability marker for advancement events that represent player damage. Vanilla advancement reward functions identify the triggering player as `@s`, but they do not provide exact damage amount to the reward function. Use [`DamageAmount::Fixed`](sand::command::DamageAmount::Fixed) today, or add a real tracking system before using same-as-event damage.",
    minecraft = "It is used with vanilla damage triggers and does not synthesize damage data for unrelated events.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::DamageAdvancementEvent;",
)]
/// Capability marker for advancement events that represent player damage.
///
/// Vanilla advancement reward functions identify the triggering player as
/// `@s`, but they do not provide exact damage amount to the reward function.
/// Use [`DamageAmount::Fixed`](sand_commands::DamageAmount::Fixed) today, or
/// add a real tracking system before using same-as-event damage.
pub trait DamageAdvancementEvent: AdvancementEvent {}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::EventPlayer",
    aliases = ["sand::prelude::EventPlayer"],
    module = "sand::event",
    summary = "Provides the executing-player selector to legacy bare-marker event handlers.",
    context = "Provides the executing-player selector to legacy bare-marker event handlers. `#[on_event]` still accepts built-in marker parameters such as `event: OnJoinEvent`. In that compatibility form the marker is a stateless context value and `player()` returns the `@s` player selected by Sand's generated dispatcher. Prefer [`Event<E>`] for new advancement-backed handlers; this trait keeps existing bare-marker authoring source-compatible.",
    minecraft = "Sand runs supported player-scoped handlers as `@s`; new advancement-backed handlers should prefer Event<E>.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::EventPlayer;",
)]
/// Provides the executing-player selector to legacy bare-marker event
/// handlers.
///
/// `#[on_event]` still accepts built-in marker parameters such as
/// `event: OnJoinEvent`. In that compatibility form the marker is a stateless
/// context value and `player()` returns the `@s` player selected by Sand's
/// generated dispatcher. Prefer [`Event<E>`] for new advancement-backed
/// handlers; this trait keeps existing bare-marker authoring source-compatible.
pub trait EventPlayer {
    /// Returns `@s`, the player for whom the event handler is executing.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::EventPlayer::player", kind = "trait_method",
        aliases = ["sand::prelude::EventPlayer::player"],
        module = "sand::event",
        summary = "Returns `@s`, the player for whom the event handler is executing.",
        context = "Returns `@s`, the player for whom the event handler is executing. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The selector is Minecraft's current `@s` player supplied by Sand's generated dispatcher.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "Returns `@s`, the player for whom the event handler is executing.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::event::EventPlayer>(event_player_value: &T)  {\n    let player = event_player_value.player();\n}",
    )]
    fn player(&self) -> sand_commands::Selector {
        sand_commands::Selector::self_()
    }
}

impl<T: crate::events::SandEvent> EventPlayer for T {}
impl EventPlayer for crate::events::OnJoinEvent {}
impl EventPlayer for crate::events::FirstJoinEvent {}
impl EventPlayer for crate::events::OnDeathEvent {}
impl EventPlayer for crate::events::OnRespawnEvent {}

// ── Event<E> — handler context ───────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::Event",
    aliases = ["sand::prelude::Event"],
    module = "sand::event",
    summary = "Zero-cost runtime context for `#[on_event]`-annotated advancement handlers.",
    context = "Zero-cost runtime context for `#[on_event]`-annotated advancement handlers. Inside an `#[on_event]` handler, the generated code creates an `Event<E>` value that gives you access to context methods like [`Event::player`]. It is shared by advancement-backed and generated tracked events. You never construct `Event<E>` manually — the `#[on_event]` macro generates it. The context contains no instance of `E`; ordinary fields on the marker type are not captured Minecraft values. Event-time data must come from context handles explicitly provided by Sand or from typed state queried in the handler.",
    minecraft = "Inside an `#[on_event]` handler, the generated code creates an `Event<E>` value that gives you access to context methods like [`Event::player`]. It is shared by advancement-backed and generated tracked events. You never construct `Event<E>` manually — the `#[on_event]` macro generates it. The context contains no instance of `E`; ordinary fields on the marker type are not captured Minecraft values. Event-time data must come from context handles explicitly provided by Sand or from typed state queried in the handler.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::Event;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::context",
        aliases = ["sand::prelude::Event::context"],
        module = "sand::event",
        kind = "method",
        summary = "Construct the handler context value. Called by `#[on_event]`-generated code. Not normally called directly.",
        context = "Construct the handler context value. Called by `#[on_event]`-generated code. Not normally called directly. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The context reads the execution player and participant observations stored around the generated reward function.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "An `Event` representing the handler context value. Called by `#[on_event]`-generated code. Not normally called directly.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E: 'static>()  {\n    let event = sand::event::Event ::< E >::context();\n}",
    )]
    pub fn context() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Returns `Selector::self_()` — the player who triggered the event.
    ///
    /// `@s` is the player selected by the advancement reward or generated
    /// per-player dispatcher.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::player",
        aliases = ["sand::prelude::Event::player"],
        module = "sand::event",
        kind = "method",
        summary = "Returns `Selector::self_()` — the player who triggered the event.",
        context = "Returns `Selector::self_()` — the player who triggered the event. `@s` is the player selected by the advancement reward or generated per-player dispatcher.",
        minecraft = "Resolves to the reward function's @s player, not an arbitrary entity.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "Returns `Selector::self_()` — the player who triggered the event.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E: 'static>(event_value: &sand::event::Event < E >)  {\n    let player = event_value.player();\n}",
    )]
    pub fn player(&self) -> sand_commands::Selector {
        sand_commands::Selector::self_()
    }

    /// Returns `Selector::self_()` — alias for [`player`](Event::player).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::subject",
        aliases = ["sand::prelude::Event::subject"],
        module = "sand::event",
        kind = "method",
        summary = "Returns `Selector::self_()` — alias for [`player`](Event::player).",
        context = "Returns `Selector::self_()` — alias for [`player`](Event::player). This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "For advancement events this is the triggering player bound to @s.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "Returns `Selector::self_()` — alias for [`player`](Event::player).",
        example = "use sand::prelude::*;\n\nfn demonstrate<E: 'static>(event_value: &sand::event::Event < E >)  {\n    let subject = event_value.subject();\n}",
    )]
    pub fn subject(&self) -> sand_commands::Selector {
        sand_commands::Selector::self_()
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::state_init",
        aliases = ["sand::prelude::Event::state_init"],
        module = "sand::event",
        kind = "method",
        summary = "`scoreboard objectives add …` commands for every state variable this event declared via [`AdvancementEvent::state_defines`].",
        context = "`scoreboard objectives add …` commands for every state variable this event declared via [`AdvancementEvent::state_defines`]. Call this in your `#[datapack_component(Load)]` function so all objectives exist before the event fires:",
        minecraft = "Call this in your `#[datapack_component(Load)]` function so all objectives exist before the event fires:",
        use_when = ["Call this in your `#[datapack_component(Load)]` function so all objectives exist before the event fires:"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The ordered values produced to emit the documented `scoreboard objectives add …` commands for every state variable this event declared via [`AdvancementEvent::state_defines`] form.",
        example = "fn load() {\nfor cmd in Event::<DrinkManaEvent>::state_init() {\ncmd::raw(cmd);\n}\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::entity",
        aliases = ["sand::prelude::Event::entity"],
        module = "sand::event",
        kind = "method",
        summary = "Access a declared entity participant by role (#230, infallible per #273).",
        context = "Access a declared entity participant by role (#230, infallible per #273). Backed by whatever [`AdvancementEvent::participants`] declared for this event type. Returns the typed participant directly — not wrapped in `Result`/`Option`/[`sand::participant::ParticipantAvailability`] — since a role this event does not declare is a build-time authoring mistake (`sand build`'s mandatory graph validation is expected to catch it before output is written), not a value for ordinary handler code to branch on. See `EventParticipantPlan::require_entity` (crate-private) for the exact reconstruction and panic contract.",
        minecraft = "The value is backed by Sand's event-cycle observation storage and is only available when the event declared that role.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(role = "`role` identifies the declared entity participant by role (#230, infallible per #273)."),
        returns = "Backed by whatever [`AdvancementEvent::participants`] declared for this event type. Returns the typed participant directly — not wrapped in `Result`/`Option`/[`sand::participant::ParticipantAvailability`] — since a role this event does not declare is a build-time authoring mistake (`sand build`'s mandatory graph validation is expected to catch it before output is written), not a value for ordinary handler code to branch on. See `EventParticipantPlan::require_entity` (crate-private) for the exact reconstruction and panic contract.",
        example = "fn on_hit(event: Event<EntityDamagePlayerEvent>) {\nlet attacker = event.entity(EntityParticipantRole::Attacker);\n// build commands against attacker.selector()\n}",
    )]
    pub fn entity(
        &self,
        role: crate::participant::EntityParticipantRole,
    ) -> crate::participant::EntityParticipant {
        E::participants().require_entity(std::any::type_name::<E>(), role)
    }

    /// Access a declared item participant by role (#230, infallible per
    /// #273). See [`Self::entity`] for the contract this mirrors.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::item",
        aliases = ["sand::prelude::Event::item"],
        module = "sand::event",
        kind = "method",
        summary = "Access a declared item participant by role (#230, infallible per #273). See [`Self::entity`] for the contract this mirrors.",
        context = "Access a declared item participant by role (#230, infallible per #273). See [`Self::entity`] for the contract this mirrors. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand captures matching item NBT at trigger time so later commands do not depend on a live inventory slot.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(role = "`role` identifies the declared item participant by role (#230, infallible per #273). See [`Self::entity`] for the contract this mirrors."),
        returns = "The `sand :: item :: ItemSnapshot` value produced to acces a declared item participant by role (#230, infallible per #273). See [`Self::entity`] for the contract this mirrors.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::AdvancementEvent + 'static>(event_value: &sand::event::Event < E >, role: sand::participant::ItemParticipantRole)  {\n    let item = event_value.item(role);\n}",
    )]
    pub fn item(&self, role: crate::participant::ItemParticipantRole) -> crate::item::ItemSnapshot {
        E::participants().require_item(std::any::type_name::<E>(), role)
    }

    /// The entity that caused this event, when declared. `DirectAttacker`
    /// (the immediate causing entity, e.g. an arrow rather than the player
    /// who shot it) is a distinct role vanilla's damage source also draws,
    /// but no credible backend exists for it today — see
    /// `docs/testing/participant-role-evidence.md`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::attacker",
        aliases = ["sand::prelude::Event::attacker"],
        module = "sand::event",
        kind = "method",
        summary = "The entity that caused this event, when declared. `DirectAttacker` (the immediate causing entity, e.g. an arrow rather than the player who shot it) is a distinct role vanilla's damage source also draws, but no credible backend exists for it today — see `docs/testing/participant-role-evidence.md`.",
        context = "The entity that caused this event, when declared. `DirectAttacker` (the immediate causing entity, e.g. an arrow rather than the player who shot it) is a distinct role vanilla's damage source also draws, but no credible backend exists for it today — see `docs/testing/participant-role-evidence.md`. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The participant is available only for triggers and plans that can observe an attacker.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: participant :: EntityParticipant` value produced to use the entity that caused this event, when declared. `DirectAttacker` (the immediate causing entity, e.g. an arrow rather than the player who shot it) is a distinct role vanilla's damage source also draws, but no credible backend exists for it today — see `docs/testing/participant-role-evidence.md`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::AdvancementEvent + 'static>(event_value: &sand::event::Event < E >)  {\n    let attacker = event_value.attacker();\n}",
    )]
    pub fn attacker(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::Attacker)
    }

    /// The entity that landed the killing blow, when declared.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::killer",
        aliases = ["sand::prelude::Event::killer"],
        module = "sand::event",
        kind = "method",
        summary = "The entity that landed the killing blow, when declared.",
        context = "The entity that landed the killing blow, when declared. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It is read from the current event dispatch record, not looked up again after the kill.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: participant :: EntityParticipant` value produced to use the entity that landed the killing blow, when declared.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::AdvancementEvent + 'static>(event_value: &sand::event::Event < E >)  {\n    let killer = event_value.killer();\n}",
    )]
    pub fn killer(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::Killer)
    }

    /// The entity that received damage/an effect, when declared.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::victim",
        aliases = ["sand::prelude::Event::victim"],
        module = "sand::event",
        kind = "method",
        summary = "The entity that received damage/an effect, when declared.",
        context = "The entity that received damage/an effect, when declared. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The value follows the event's participant plan and dispatch-cycle lifetime.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: participant :: EntityParticipant` value produced to use the entity that received damage/an effect, when declared.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::AdvancementEvent + 'static>(event_value: &sand::event::Event < E >)  {\n    let victim = event_value.victim();\n}",
    )]
    pub fn victim(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::Victim)
    }

    /// The entity this player directly interacted with, when declared.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::interacted_entity",
        aliases = ["sand::prelude::Event::interacted_entity"],
        module = "sand::event",
        kind = "method",
        summary = "The entity this player directly interacted with, when declared.",
        context = "The entity this player directly interacted with, when declared. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It reflects the entity matched by the vanilla interaction trigger when that observation is declared.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: participant :: EntityParticipant` value produced to use the entity this player directly interacted with, when declared.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::AdvancementEvent + 'static>(event_value: &sand::event::Event < E >)  {\n    let interacted_entity = event_value.interacted_entity();\n}",
    )]
    pub fn interacted_entity(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::InteractedEntity)
    }

    /// The weapon item snapshot, when declared — see
    /// [`crate::participant::EventParticipantPlan::observe_weapon`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::weapon",
        aliases = ["sand::prelude::Event::weapon"],
        module = "sand::event",
        kind = "method",
        summary = "The weapon item snapshot, when declared — see [`sand::participant::EventParticipantPlan::observe_weapon`].",
        context = "The weapon item snapshot, when declared — see [`sand::participant::EventParticipantPlan::observe_weapon`]. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand stores the trigger-time item NBT so it remains stable through the handler.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: item :: ItemSnapshot` value produced to use the weapon item snapshot, when declared — see [`sand::participant::EventParticipantPlan::observe_weapon`].",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::AdvancementEvent + 'static>(event_value: &sand::event::Event < E >)  {\n    let weapon = event_value.weapon();\n}",
    )]
    pub fn weapon(&self) -> crate::item::ItemSnapshot {
        self.item(crate::participant::ItemParticipantRole::Weapon)
    }

    /// Access a declared bounded item participant by role (#272, infallible
    /// per #273) — the `.within(...)`-crossing counterpart to [`Self::item`].
    /// See [`crate::participant::EventParticipantPlan::inherit_item_within`]
    /// for the full contract.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::bounded_item",
        aliases = ["sand::prelude::Event::bounded_item"],
        module = "sand::event",
        kind = "method",
        summary = "Access a declared bounded item participant by role (#272, infallible per #273) — the `.within(...)`-crossing counterpart to [`Self::item`]. See [`sand::participant::EventParticipantPlan::inherit_item_within`] for the full contract.",
        context = "Access a declared bounded item participant by role (#272, infallible per #273) — the `.within(...)`-crossing counterpart to [`Self::item`]. See [`sand::participant::EventParticipantPlan::inherit_item_within`] for the full contract. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The snapshot is available only while the event graph's configured tick window has not expired.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(role = "`role` identifies the declared bounded item participant by role (#272, infallible per #273) — the `.within(...)`-crossing counterpart to [`Self::item`]. See [`sand::participant::EventParticipantPlan::inherit_item_within`] for the full contract."),
        returns = "The `sand :: participant :: BoundedItemSnapshot` value produced to acces a declared bounded item participant by role (#272, infallible per #273) — the `.within(...)`-crossing counterpart to [`Self::item`]. See [`sand::participant::EventParticipantPlan::inherit_item_within`] for the full contract.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::AdvancementEvent + 'static>(event_value: &sand::event::Event < E >, role: sand::participant::ItemParticipantRole)  {\n    let bounded_item = event_value.bounded_item(role);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::Event::damage",
        aliases = ["sand::prelude::Event::damage"],
        module = "sand::event",
        kind = "method",
        summary = "Start a reflected-damage command builder from this event's player.",
        context = "Start a reflected-damage command builder from this event's player. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It reflects the damage data exposed by Minecraft's advancement trigger context.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: command :: DamageBuilder` value produced to start a reflected-damage command builder from this event's player.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::DamageAdvancementEvent + 'static>(event_value: &sand::event::Event < E >)  {\n    let damage = event_value.damage();\n}",
    )]
    pub fn damage(&self) -> sand_commands::Damage {
        sand_commands::Damage::reflect_from(crate::cmd::SingleEntity::self_())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::event::DamageEvent",
    aliases = ["sand::prelude::DamageEvent"],
    module = "sand::event",
    summary = "Damage-specific event handler context for `#[on_event]` functions.",
    context = "Damage-specific event handler context for `#[on_event]` functions. Use `DamageEvent<T>` when `T: DamageAdvancementEvent`. It exposes the triggering player as a statically single player/entity target and provides a first-class reflected-damage builder.",
    minecraft = "Runs in the triggering player's advancement reward function and reflects the captured vanilla damage predicate context.",
    use_when = ["Use `DamageEvent<T>` when `T: DamageAdvancementEvent`. It exposes the triggering player as a statically single player/entity target and provides a first-class reflected-damage builder."],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::event::DamageEvent;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::DamageEvent::context",
        aliases = ["sand::prelude::DamageEvent::context"],
        module = "sand::event",
        kind = "method",
        summary = "Construct the handler context value. Called by `#[on_event]`-generated code. Not normally called directly.",
        context = "Construct the handler context value. Called by `#[on_event]`-generated code. Not normally called directly. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The value resolves its subject and damage view in the generated reward function.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "A `DamageEvent` representing the handler context value. Called by `#[on_event]`-generated code. Not normally called directly.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::DamageAdvancementEvent + 'static>()  {\n    let damage_event = sand::event::DamageEvent ::< E >::context();\n}",
    )]
    pub fn context() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Returns `@s` as a single player: the player who triggered the event.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::DamageEvent::player",
        aliases = ["sand::prelude::DamageEvent::player"],
        module = "sand::event",
        kind = "method",
        summary = "Returns `@s` as a single player: the player who triggered the event.",
        context = "Returns `@s` as a single player: the player who triggered the event. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Always resolves the advancement reward function's @s player.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "Returns `@s` as a single player: the player who triggered the event.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::DamageAdvancementEvent + 'static>(damage_event_value: &sand::event::DamageEvent < E >)  {\n    let player = damage_event_value.player();\n}",
    )]
    pub fn player(&self) -> crate::cmd::SinglePlayer {
        crate::cmd::SinglePlayer::self_()
    }

    /// Returns `@s` as a single entity: the damaged subject.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::DamageEvent::subject",
        aliases = ["sand::prelude::DamageEvent::subject"],
        module = "sand::event",
        kind = "method",
        summary = "Returns `@s` as a single entity: the damaged subject.",
        context = "Returns `@s` as a single entity: the damaged subject. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It is the triggering player under Minecraft's advancement reward execution.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "Returns `@s` as a single entity: the damaged subject.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::DamageAdvancementEvent + 'static>(damage_event_value: &sand::event::DamageEvent < E >)  {\n    let subject = damage_event_value.subject();\n}",
    )]
    pub fn subject(&self) -> crate::cmd::SingleEntity {
        crate::cmd::SingleEntity::self_()
    }

    /// Start a reflected-damage builder centered on and sourced from the player.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::event::DamageEvent::reflect_damage",
        aliases = ["sand::prelude::DamageEvent::reflect_damage"],
        module = "sand::event",
        kind = "method",
        summary = "Start a reflected-damage builder centered on and sourced from the player.",
        context = "Start a reflected-damage builder centered on and sourced from the player. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Uses the same damage source information Sand can reflect from the advancement-backed event.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: command :: DamageBuilder` value produced to start a reflected-damage builder centered on and sourced from the player.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::event::DamageAdvancementEvent + 'static>(damage_event_value: &sand::event::DamageEvent < E >)  {\n    let reflect_damage = damage_event_value.reflect_damage();\n}",
    )]
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
    }

    #[test]
    fn event_reset_once_does_not_revoke() {
        assert!(!EventReset::OncePerPlayer.should_revoke());
        assert!(!EventReset::Manual.should_revoke());
    }
}
