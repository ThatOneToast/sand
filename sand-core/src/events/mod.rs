//! Built-in event types and the advanced [`SandEvent`] custom-event trait.
//!
//! New custom advancement-backed events should implement
//! [`AdvancementEvent`](crate::event::AdvancementEvent) and use
//! [`Event<T>`](crate::event::Event) as the handler parameter:
//!
//! ```rust,ignore
//! use sand_core::prelude::*;
//! use sand_core::event::trigger::ConsumeItemTrigger;
//! use sand_macros::on_event;
//!
//! pub struct AteGoldenAppleEvent;
//!
//! impl AdvancementEvent for AteGoldenAppleEvent {
//!     type Trigger = ConsumeItemTrigger;
//!     fn trigger() -> Self::Trigger {
//!         ConsumeItemTrigger::new().item(ItemPredicate::id(ItemId::minecraft("golden_apple")?))
//!     }
//! }
//!
//! #[on_event]
//! pub fn on_ate_golden_apple(event: Event<AteGoldenAppleEvent>) {
//!     cmd::say("Golden apple eaten");
//! }
//! ```
//!
//! # Built-in events
//!
//! | Type | When it fires | Required filters |
//! |---|---|---|
//! | [`OnJoinEvent`] | First tick after load, or new player mid-session | — |
//! | [`FirstJoinEvent`] | Very first join ever | — |
//! | [`OnDeathEvent`] | Any death (mob, fall, void, `/kill`, …) | — |
//! | [`OnRespawnEvent`] | First Sand tick observing post-death activity | — |
//! | [`ArmorEquipEvent`] | Item equipped in an equipment slot | `slot` |
//! | [`ArmorUnequipEvent`] | Item removed from an equipment slot | `slot` |
//! | [`HoldingItemEvent`] | Holding item (every tick) | `item` |
//! | [`CurrentlyWearingEvent`] | Wearing item in armor slot (every tick) | `slot`, `item` |
//!
//! # Usage
//!
//! Use the `#[on_event]` attribute macro from `sand_macros` on a free-standing
//! function. The primary handler parameter is `Event<T>` where `T` implements
//! [`AdvancementEvent`](crate::event::AdvancementEvent):
//!
//! ```rust,ignore
//! use sand_macros::on_event;
//! use sand_core::prelude::*;
//! use sand_core::events::{OnJoinEvent, OnDeathEvent, ArmorEquipEvent};
//!
//! static TOTAL_DEATHS: ScoreVar<i32> = ScoreVar::new("total_deaths");
//!
//! #[on_event]
//! pub fn on_join(event: Event<OnJoinEvent>) {
//!     cmd::tellraw(
//!         Selector::self_(),
//!         Text::new("Welcome!").gold(),
//!     );
//! }
//!
//! #[on_event]
//! pub fn on_death(event: Event<OnDeathEvent>) {
//!     TOTAL_DEATHS.add(event.player(), 1);
//! }
//!
//! // Slot filter required; item is optional
//! #[on_event(slot = Head, item = "minecraft:diamond_helmet")]
//! pub fn equipped_diamond_helmet(event: Event<ArmorEquipEvent>) {
//!     cmd::say("Diamond helmet on!");
//! }
//! ```
//!
//! # Custom advancement events
//!
//! For custom advancement-backed events, implement
//! [`AdvancementEvent`](crate::event::AdvancementEvent) on a marker struct and
//! handle it with `Event<T>`:
//!
//! ```rust,ignore
//! use sand_core::event::trigger::ConsumeItemTrigger;
//! use sand_core::prelude::*;
//! use sand_macros::on_event;
//!
//! pub struct AteGoldenAppleEvent;
//!
//! impl AdvancementEvent for AteGoldenAppleEvent {
//!     type Trigger = ConsumeItemTrigger;
//!     fn trigger() -> Self::Trigger {
//!         ConsumeItemTrigger::new().item(ItemPredicate::id(ItemId::minecraft("golden_apple")?))
//!     }
//! }
//!
//! #[on_event]
//! pub fn on_ate_golden_apple(event: Event<AteGoldenAppleEvent>) {
//!     cmd::say("Golden apple eaten");
//! }
//! ```
//!
//! # `SandEvent`: advanced custom events
//!
//! [`SandEvent`] is not a legacy fallback — it is Sand's primary extension
//! point for advanced custom events: typed tick dispatch built from the same
//! [`Condition`](crate::condition::Condition) IR used everywhere else, event-owned
//! lifecycle (setup objectives, pre/post-observation commands via
//! [`SandEvent::setup`]), and generic event families with distinct concrete
//! identities. Implement [`AdvancementEvent`](crate::event::AdvancementEvent)
//! instead when your event maps to exactly one vanilla advancement trigger and
//! needs no owned lifecycle — that is the lighter-weight, common case.
//!
//! ```rust,ignore
//! use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
//! use sand_core::prelude::*;
//! use sand_macros::on_event;
//!
//! static JUMPS: ScoreVar<i32> = ScoreVar::new("jumps");
//! static SYNC_JUMPS: ScoreVar<i32> = ScoreVar::new("sync_jumps");
//!
//! pub struct PlayerJumpEvent;
//!
//! impl SandEvent for PlayerJumpEvent {
//!     fn dispatch() -> SandEventDispatch {
//!         SandEventDispatch::tick()
//!             .as_players()
//!             .when(SYNC_JUMPS.of("@s").lt_score(JUMPS.of("@s")))
//!             .into()
//!     }
//!
//!     fn setup() -> EventSetup {
//!         EventSetup {
//!             objectives: vec![
//!                 "scoreboard objectives add jumps minecraft.custom:minecraft.jump".into(),
//!                 "scoreboard objectives add sync_jumps dummy".into(),
//!             ],
//!             pre_observation: vec![],
//!             // Runs unconditionally after detection each tick, so the sync
//!             // score never overwrites the value being compared against
//!             // before it's observed.
//!             post_observation: vec![
//!                 "execute as @a run scoreboard players operation @s sync_jumps = @s jumps".into(),
//!             ],
//!         }
//!     }
//! }
//!
//! #[on_event]
//! pub fn on_jump(_event: PlayerJumpEvent) {
//!     cmd::say("Jumped!");
//! }
//! ```
//!
//! Unlike `Event<T>`, a bare `SandEvent` parameter is the concrete marker
//! value generated for the handler. Keep subscribed markers constructible as
//! unit types. Generic `SandEvent` definitions are supported, with distinct
//! identity for each concrete monomorphization; use a concrete unit adapter
//! when a generic definition stores `PhantomData` or other fields.
//!
//! [`SandEventDispatch::chain`] implements concise single-parent same-cycle
//! chaining for tick-backed `SandEvent`s. [`SandEventDispatch::compose`],
//! [`ChainEventDispatch::after_any`], and [`ChainEventDispatch::after_all`]
//! add deterministic multi-parent same-cycle clauses. A composed child can
//! additionally require explicit persistent state with
//! [`ChainEventDispatch::while_`], or bounded cross-tick correlation with
//! [`ChainEventDispatch::within`] (see [`TickWindow`] for the exact boundary
//! convention). Advancement-backed graph parents and participant-rich
//! contexts remain future work and are not current APIs.
//!
//! Simple advancement-backed or single-fragment tick-poll `SandEvent` impls
//! remain supported via [`SandEventDispatch::AdvancementTrigger`] and
//! [`SandEventDispatch::TickCondition`] — both lower into the same internal
//! representation as [`SandEventDispatch::tick()`].

/// Event dependency graph construction for same-cycle chained dispatch (#240).
///
/// The graph is exporter wiring. Authors compose public `SandEventDispatch`
/// builders instead of depending on graph nodes or edge records directly.
#[allow(dead_code)]
pub(crate) mod graph;

// ── Custom event API ──────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::TickScope",
    module = "sand::events",
    summary = "Execution scope for a structured [`TickEventDispatch`], or (as of #240 Phase 6) the graph execution-context capability a parent provides.",
    context = "Execution scope for a structured [`TickEventDispatch`], or (as of #240 Phase 6) the graph execution-context capability a parent provides. This is the graph's one deterministic, non-reflective capability seam: every parent resolution site checks a concrete `TickScope` value rather than inspecting handler code. More scopes (e.g. arbitrary entity queries) remain a natural future extension point.",
    minecraft = "Players polls each online player; AdvancementPlayer is the exact triggering player in an advancement reward function.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::TickScope;",
    variants(AdvancementPlayer = "A single exact player subject bound to `@s`, provided synchronously inside a vanilla advancement reward function rather than polled by `minecraft:tick` — see [`ChainEventDispatch::after`] on an advancement-backed `SandEvent` (#240 Phase 6).", Players = "Evaluated as each online player (`execute as @a ... at @s run ...`)."),
)]
/// Execution scope for a structured [`TickEventDispatch`], or (as of #240
/// Phase 6) the graph execution-context capability a parent provides.
///
/// This is the graph's one deterministic, non-reflective capability seam:
/// every parent resolution site checks a concrete `TickScope` value rather
/// than inspecting handler code. More scopes (e.g. arbitrary entity queries)
/// remain a natural future extension point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TickScope {
    /// Evaluated as each online player (`execute as @a ... at @s run ...`).
    #[default]
    Players,
    /// A single exact player subject bound to `@s`, provided synchronously
    /// inside a vanilla advancement reward function rather than polled by
    /// `minecraft:tick` — see [`ChainEventDispatch::after`] on an
    /// advancement-backed `SandEvent` (#240 Phase 6).
    ///
    /// Narrower than [`Players`](Self::Players): it guarantees a player
    /// subject but not a per-tick polling frame, so it is compatible with
    /// same-cycle composition only in the single, sole-parent
    /// `after::<E>()` shape — never `after_any`/`after_all` (which require
    /// the tick coordinator to observe multiple parents' marks in one
    /// deterministic pass) and never `within::<E>(...)` (which requires the
    /// coordinator to maintain a per-tick age counter). See
    /// [`TickScope::has_player_subject`].
    AdvancementPlayer,
}

impl TickScope {
    /// Whether this scope guarantees an exact, single player subject bound
    /// to `@s` — true for both [`Players`](Self::Players) (tick-polled) and
    /// [`AdvancementPlayer`](Self::AdvancementPlayer) (advancement
    /// reward-triggered). Used to validate that a child's inherited-player
    /// requirement is satisfiable by a candidate parent's scope, independent
    /// of *how* that parent is detected.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickScope::has_player_subject",
        module = "sand::events",
        kind = "method",
        summary = "Whether this scope guarantees an exact, single player subject bound to `@s` — true for both [`Players`](Self::Players) (tick-polled) and [`AdvancementPlayer`](Self::AdvancementPlayer) (advancement reward-triggered). Used to validate that a child's inherited-player requirement is satisfiable by a candidate parent's scope, independent of *how* that parent is detected.",
        context = "Whether this scope guarantees an exact, single player subject bound to `@s` — true for both [`Players`](Self::Players) (tick-polled) and [`AdvancementPlayer`](Self::AdvancementPlayer) (advancement reward-triggered). Used to validate that a child's inherited-player requirement is satisfiable by a candidate parent's scope, independent of *how* that parent is detected. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand uses this capability when validating participant inheritance in composed events.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "`true` when the documented condition holds to determine whether this scope guarantees an exact, single player subject bound to `@s` — true for both [`Players`](Self::Players) (tick-polled) and [`AdvancementPlayer`](Self::AdvancementPlayer) (advancement reward-triggered). Used to validate that a child's inherited-player requirement is satisfiable by a candidate parent's scope, independent of *how* that parent is detected; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tick_scope_value: sand::events::TickScope)  {\n    let is_has_player_subject = tick_scope_value.has_player_subject();\n}",
    )]
    pub fn has_player_subject(self) -> bool {
        matches!(self, Self::Players | Self::AdvancementPlayer)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PersistentEventCondition",
    module = "sand::events",
    summary = "A directly queryable persistent event condition.",
    context = "A directly queryable persistent event condition. Unlike [`TickEventDispatch`], this value describes current truth, not an independently firing occurrence detector. The condition is evaluated at a chained child's dispatch boundary under the inherited player `@s` and position. It does not run the provider event's detector or lifecycle.",
    minecraft = "It is evaluated under the inherited player at a child dispatch boundary instead of independently firing a detector.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::PersistentEventCondition;",
)]
/// A directly queryable persistent event condition.
///
/// Unlike [`TickEventDispatch`], this value describes current truth, not an
/// independently firing occurrence detector. The condition is evaluated at a
/// chained child's dispatch boundary under the inherited player `@s` and
/// position. It does not run the provider event's detector or lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistentEventCondition {
    pub(crate) scope: TickScope,
    pub(crate) condition: crate::condition::Condition,
}

impl PersistentEventCondition {
    /// Define a condition that is safe to evaluate as the inherited player.
    ///
    /// Prefer typed [`Condition`](crate::condition::Condition) constructors.
    /// A [`Condition::raw`](crate::condition::Condition::raw) value remains an
    /// explicit compatibility escape hatch whose target-version semantics are
    /// user-owned when Sand cannot validate the fragment.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::PersistentEventCondition::players",
        module = "sand::events",
        kind = "method",
        summary = "Define a condition that is safe to evaluate as the inherited player.",
        context = "Define a condition that is safe to evaluate as the inherited player. Prefer typed [`Condition`](sand::condition::Condition) constructors. A [`Condition::raw`](sand::condition::Condition::raw) value remains an explicit compatibility escape hatch whose target-version semantics are user-owned when Sand cannot validate the fragment.",
        minecraft = "Its typed condition becomes execute clauses under @s when a composed child is considered.",
        use_when = ["Prefer typed [`Condition`](sand::condition::Condition) constructors. A [`Condition::raw`](sand::condition::Condition::raw) value remains an explicit compatibility escape hatch whose target-version semantics are user-owned when Sand cannot validate the fragment."],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(condition = "`condition` provides the condition that gates the operation used to define a condition that is safe to evaluate as the inherited player."),
        returns = "A `PersistentEventCondition` defining a condition that is safe to evaluate as the inherited player.",
        example = "use sand::prelude::*;\n\nfn demonstrate(condition: impl Into < sand::condition::Condition >)  {\n    let persistent_event_condition = sand::events::PersistentEventCondition::players(condition);\n}",
    )]
    pub fn players(condition: impl Into<crate::condition::Condition>) -> Self {
        Self {
            scope: TickScope::Players,
            condition: condition.into(),
        }
    }

    /// The execution scope required by this condition.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::PersistentEventCondition::scope",
        module = "sand::events",
        kind = "method",
        summary = "The execution scope required by this condition.",
        context = "The execution scope required by this condition. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The scope determines whether a parent can safely provide the condition's @s player.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `TickScope` value produced to use the execution scope required by this condition.",
        example = "use sand::prelude::*;\n\nfn demonstrate(persistent_event_condition_value: &sand::events::PersistentEventCondition)  {\n    let scope = persistent_event_condition_value.scope();\n}",
    )]
    pub fn scope(&self) -> TickScope {
        self.scope
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PersistentSandEvent",
    module = "sand::events",
    summary = "Explicit opt-in contract for event types that represent persistent state.",
    context = "Explicit opt-in contract for event types that represent persistent state. Implementing [`SandEvent`] alone is intentionally insufficient: a tick event may represent an occurrence or transition rather than a state that remains true. Only types with a direct current-state representation should implement this trait. A provider must keep [`SandEvent::setup()`] empty. `while_` never runs a provider detector or observation lifecycle, so objectives and other prerequisites must be provisioned independently (for example through typed state lifecycle). Export rejects a non-empty provider setup and names both the child and provider rather than silently omitting it.",
    minecraft = "A provider must keep [`SandEvent::setup()`] empty. `while_` never runs a provider detector or observation lifecycle, so objectives and other prerequisites must be provisioned independently (for example through typed state lifecycle). Export rejects a non-empty provider setup and names both the child and provider rather than silently omitting it.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::PersistentSandEvent;",
)]
/// Explicit opt-in contract for event types that represent persistent state.
///
/// Implementing [`SandEvent`] alone is intentionally insufficient: a tick
/// event may represent an occurrence or transition rather than a state that
/// remains true. Only types with a direct current-state representation should
/// implement this trait.
///
/// A provider must keep [`SandEvent::setup()`] empty. `while_` never runs a
/// provider detector or observation lifecycle, so objectives and other
/// prerequisites must be provisioned independently (for example through
/// typed state lifecycle). Export rejects a non-empty provider setup and names
/// both the child and provider rather than silently omitting it.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used with `while_::<E>()` because it does not implement `PersistentSandEvent`",
    label = "this event type has no explicit persistent-state representation",
    note = "`SandEvent` dispatch describes when an event fires; implement `PersistentSandEvent` only when the type can also provide a directly queryable current condition"
)]
pub trait PersistentSandEvent: SandEvent {
    /// Return the current-state condition for this event type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::PersistentSandEvent::persistent_condition",
        module = "sand::events",
        summary = "Return the current-state condition for this event type.",
        context = "Return the current-state condition for this event type. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand emits it at the composed child's dispatch boundary under the inherited player.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "Return the current-state condition for this event type.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::PersistentSandEvent>()  {\n    let persistent_condition = <T as sand::events::PersistentSandEvent>::persistent_condition();\n}",
    )]
    fn persistent_condition() -> PersistentEventCondition;
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::TickWindow",
    module = "sand::events",
    summary = "A validated bounded cross-tick correlation window for [`ChainEventDispatch::within`].",
    context = "A validated bounded cross-tick correlation window for [`ChainEventDispatch::within`]. `within::<E>(TickWindow::new(N)?)` is satisfied for the current subject when `E` fired during the current evaluation cycle or within the previous `N - 1` completed tick boundaries. Concretely, tracking an integer *age* — ticks elapsed since `E` last fired for this subject, reset to `0` the cycle `E` fires — the window holds while `age <= N - 1`: - `N = 1` is satisfied only by a same-cycle occurrence (`age == 0`), identical to `after::<E>()`. - `age` reaches `N - 1` on the last tick the window still holds; the very next tick without a fresh occurrence (`age == N`) it does not. - A new occurrence at any point resets `age` to `0`, refreshing the full window regardless of how much of the prior window remained. Rejects `0` (a window must cover at least the current cycle) and windows larger than [`TickWindow::MAX_TICKS`], so callers cannot accidentally repurpose bounded correlation as an unbounded session/persistence mechanism — see [`TickWindowError`].",
    minecraft = "Sand stores occurrence age and accepts a parent only before the configured tick window expires.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::TickWindow;",
)]
/// A validated bounded cross-tick correlation window for
/// [`ChainEventDispatch::within`].
///
/// `within::<E>(TickWindow::new(N)?)` is satisfied for the current subject
/// when `E` fired during the current evaluation cycle **or** within the
/// previous `N - 1` completed tick boundaries. Concretely, tracking an
/// integer *age* — ticks elapsed since `E` last fired for this subject,
/// reset to `0` the cycle `E` fires — the window holds while `age <= N - 1`:
///
/// - `N = 1` is satisfied only by a same-cycle occurrence (`age == 0`),
///   identical to `after::<E>()`.
/// - `age` reaches `N - 1` on the last tick the window still holds; the
///   very next tick without a fresh occurrence (`age == N`) it does not.
/// - A new occurrence at any point resets `age` to `0`, refreshing the full
///   window regardless of how much of the prior window remained.
///
/// Rejects `0` (a window must cover at least the current cycle) and windows
/// larger than [`TickWindow::MAX_TICKS`], so callers cannot accidentally
/// repurpose bounded correlation as an unbounded session/persistence
/// mechanism — see [`TickWindowError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TickWindow(u32);

impl TickWindow {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickWindow::MIN_TICKS",
        module = "sand::events",
        kind = "associated_const",
        summary = "The smallest representable window: current-cycle occurrence only.",
        context = "The smallest representable window: current-cycle occurrence only. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "One tick includes the current dispatch cycle only.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        example = "use sand::events::TickWindow;",
    )]
    /// The smallest representable window: current-cycle occurrence only.
    pub const MIN_TICKS: u32 = 1;
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickWindow::MAX_TICKS",
        module = "sand::events",
        kind = "associated_const",
        summary = "The largest representable window (20 minutes at 20 ticks/second).",
        context = "The largest representable window (20 minutes at 20 ticks/second). Bounded correlation is meant for short cross-tick coordination windows, not long-lived session state — use durable per-player state (e.g. `sand::state`) instead.",
        minecraft = "Sand rejects larger windows to keep generated scoreboard lifecycle bounded.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        example = "use sand::events::TickWindow;",
    )]
    /// The largest representable window (20 minutes at 20 ticks/second).
    ///
    /// Bounded correlation is meant for short cross-tick coordination
    /// windows, not long-lived session state — use durable per-player state
    /// (e.g. `sand::state`) instead.
    pub const MAX_TICKS: u32 = 24_000;

    /// Validate `ticks` as a bounded correlation window.
    ///
    /// Returns [`TickWindowError::Zero`] for `0` and
    /// [`TickWindowError::TooLarge`] above [`TickWindow::MAX_TICKS`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickWindow::new",
        module = "sand::events",
        kind = "method",
        summary = "Validate `ticks` as a bounded correlation window.",
        context = "Validate `ticks` as a bounded correlation window. Returns [`TickWindowError::Zero`] for `0` and [`TickWindowError::TooLarge`] above [`TickWindow::MAX_TICKS`].",
        minecraft = "Rejects zero and values above Sand's generated event-state limit.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(ticks = "Validate `ticks` as a bounded correlation window."),
        returns = "Returns [`TickWindowError::Zero`] for `0` and [`TickWindowError::TooLarge`] above [`TickWindow::MAX_TICKS`].",
        example = "use sand::prelude::*;\n\nfn demonstrate(ticks: u32)  {\n    let tick_window_result = sand::events::TickWindow::new(ticks);\n}",
    )]
    pub fn new(ticks: u32) -> Result<Self, TickWindowError> {
        if ticks < Self::MIN_TICKS {
            return Err(TickWindowError::Zero);
        }
        if ticks > Self::MAX_TICKS {
            return Err(TickWindowError::TooLarge {
                requested: ticks,
                max: Self::MAX_TICKS,
            });
        }
        Ok(Self(ticks))
    }

    /// The validated window width, in ticks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickWindow::ticks",
        module = "sand::events",
        kind = "method",
        summary = "The validated window width, in ticks.",
        context = "The validated window width, in ticks. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The value controls how long Sand retains an occurrence mark.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `u32` value produced to use the validated window width, in ticks.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tick_window_value: sand::events::TickWindow)  {\n    let ticks = tick_window_value.ticks();\n}",
    )]
    pub fn ticks(self) -> u32 {
        self.0
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::TickWindowError",
    module = "sand::events",
    summary = "[`TickWindow::new`] validation failure.",
    context = "[`TickWindow::new`] validation failure. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
    minecraft = "Invalid windows cannot be represented safely by Sand's generated tick lifecycle.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::TickWindowError;",
    variants(TooLarge = "`requested` exceeds [`TickWindow::MAX_TICKS`].", Zero = "A window must cover at least the current cycle (`N >= 1`)."),
    variant_fields(TooLarge(max = "The largest window Sand accepts.", requested = "The rejected requested window length.")),
)]
/// [`TickWindow::new`] validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickWindowError {
    /// A window must cover at least the current cycle (`N >= 1`).
    Zero,
    /// `requested` exceeds [`TickWindow::MAX_TICKS`].
    TooLarge {
        #[doc = "The rejected requested window length."]
        requested: u32,
        #[doc = "The largest window Sand accepts."]
        max: u32,
    },
}

impl std::fmt::Display for TickWindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(
                f,
                "bounded correlation window must be at least 1 tick (0 means \"never\", not \"current cycle\")"
            ),
            Self::TooLarge { requested, max } => write!(
                f,
                "bounded correlation window of {requested} ticks exceeds the supported maximum of {max} ticks"
            ),
        }
    }
}

impl std::error::Error for TickWindowError {}

/// One typed bounded cross-tick correlation dependency attached to a chained
/// event. See [`ChainEventDispatch::within`].
pub(crate) struct BoundedEventDependency {
    #[doc(hidden)]
    pub(crate) event_type_id: fn() -> std::any::TypeId,
    #[doc(hidden)]
    pub(crate) event_type_name: fn() -> &'static str,
    #[doc(hidden)]
    pub(crate) event_dispatch: fn() -> SandEventDispatch,
    #[doc(hidden)]
    pub(crate) event_setup: fn() -> EventSetup,
    #[doc(hidden)]
    pub(crate) window: TickWindow,
}

/// One typed persistent-state dependency attached to a chained event.
pub(crate) struct PersistentEventDependency {
    #[doc(hidden)]
    pub(crate) event_type_id: fn() -> std::any::TypeId,
    #[doc(hidden)]
    pub(crate) event_type_name: fn() -> &'static str,
    #[doc(hidden)]
    pub(crate) event_dispatch: fn() -> SandEventDispatch,
    #[doc(hidden)]
    pub(crate) event_setup: fn() -> EventSetup,
    #[doc(hidden)]
    pub(crate) make_condition: fn() -> PersistentEventCondition,
}

/// One typed same-cycle event occurrence dependency.
#[derive(Clone, Copy)]
pub(crate) struct SameCycleEventDependency {
    #[doc(hidden)]
    pub(crate) event_type_id: fn() -> std::any::TypeId,
    #[doc(hidden)]
    pub(crate) event_type_name: fn() -> &'static str,
    #[doc(hidden)]
    pub(crate) event_dispatch: fn() -> SandEventDispatch,
    #[doc(hidden)]
    pub(crate) event_setup: fn() -> EventSetup,
    /// `E::setup()` called directly, with no participant-plan merge — unlike
    /// `event_setup` (which is `dependency_setup`, crate-private, the
    /// participants-merged view a same-cycle child's own recursively-discovered `EventSetup`
    /// must match). The advancement-bridge eligibility check (#269) needs
    /// this raw form: a bridge parent's own lifecycle `setup()` must still
    /// be empty (Phase 6 never runs it), but a non-empty `participants()`
    /// plan is no longer disqualifying — the export pipeline applies it
    /// directly around the synthesized bridge entry instead (see
    /// `sand-core/src/events/graph.rs`'s bridge-eligibility check and
    /// `sand-core/src/compiler/export/pipeline.rs`'s bridge loop).
    #[doc(hidden)]
    pub(crate) event_raw_setup: fn() -> EventSetup,
    /// `E::participants()` called directly — the raw plan factory carried
    /// forward so the export pipeline can apply a bridge parent's own plan
    /// (#269).
    #[doc(hidden)]
    pub(crate) event_participants: fn() -> crate::participant::EventParticipantPlan,
    /// Whether this parent's advancement is revoked after firing —
    /// [`SandEvent::revoke`]. Only meaningful when the parent resolves to
    /// advancement-backed dispatch (#240 Phase 6); ignored for tick-backed
    /// parents, which have no advancement to revoke.
    #[doc(hidden)]
    pub(crate) event_revoke: fn() -> bool,
}

/// `E::setup()` with `E::participants()` merged in exactly the same way the
/// export pipeline's `apply_participants_to_setup` merges it for `E`'s own
/// direct registration (#264) — used as this dependency's `event_setup`
/// factory so a same-cycle child's recursively-discovered view of a
/// participant-declaring parent's `EventSetup` is byte-identical to that
/// parent's own explicitly-registered `EventSetup`, not a stale
/// participant-less copy (which the graph's own consistency validation
/// would then correctly reject as two conflicting definitions of one
/// event).
///
/// Resolved against the permissive default profile (`LATEST_KNOWN`), the
/// same one `resolve_participant_profile(None)` uses — matching every
/// caller of this factory that exports without an explicit target version.
/// A version-profiled export whose target actually changes the merged
/// commands for this specific event is a known, narrow limitation: fully
/// closing it would require threading `ExportCtx` through this factory
/// type, which is out of scope here (see #264's final report).
fn dependency_setup<E: SandEvent + 'static>() -> EventSetup {
    let plan = E::participants();
    if plan.is_empty() {
        return E::setup();
    }
    let profile = crate::version::VersionProfile::resolve(
        &crate::version::MinecraftVersion::parse(crate::version::LATEST_KNOWN).unwrap(),
    )
    .expect("LATEST_KNOWN always resolves");
    E::setup().with_participants::<E>(plan, &profile).expect(
        "a participant plan declared on a same-cycle graph parent must support LATEST_KNOWN \
             — Sand does not know how to target a version newer than its own latest supported one",
    )
}

impl SameCycleEventDependency {
    fn of<E: SandEvent + 'static>() -> Self {
        Self {
            event_type_id: std::any::TypeId::of::<E>,
            event_type_name: std::any::type_name::<E>,
            event_dispatch: || E::dispatch().into(),
            event_setup: dependency_setup::<E>,
            event_raw_setup: E::setup,
            event_participants: E::participants,
            event_revoke: E::revoke,
        }
    }
}

/// One explicit same-cycle occurrence clause in a composed event definition.
pub(crate) enum SameCycleEventRequirement {
    /// One concrete parent must have fired.
    After(SameCycleEventDependency),
    /// At least one parent in the group must have fired.
    AfterAny(Vec<SameCycleEventDependency>),
    /// Every parent in the group must have fired.
    AfterAll(Vec<SameCycleEventDependency>),
}

mod event_group_private {
    use super::ChainEventDispatch;

    pub trait Sealed {
        fn apply(dispatch: ChainEventDispatch, all: bool) -> ChainEventDispatch;
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::SameCycleEventGroup",
    module = "sand::events",
    summary = "A typed tuple of two through eight concrete [`SandEvent`] parent types.",
    context = "A typed tuple of two through eight concrete [`SandEvent`] parent types. This trait is implemented by Sand for supported tuple arities and is not intended for manual implementation.",
    minecraft = "Sand uses the tuple's concrete event types to generate deterministic any-parent or all-parent correlation.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::SameCycleEventGroup;",
)]
/// A typed tuple of two through eight concrete [`SandEvent`] parent types.
///
/// This trait is implemented by Sand for supported tuple arities and is not
/// intended for manual implementation.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a supported same-cycle event group",
    label = "expected a tuple of 2 through 8 concrete `SandEvent` types",
    note = "use `after::<E>()` for one parent, or `after_any::<(A, B)>()` / `after_all::<(A, B)>()` for 2 through 8 parents"
)]
pub trait SameCycleEventGroup: event_group_private::Sealed {}

fn apply_event_group(
    mut dispatch: ChainEventDispatch,
    all: bool,
    dependencies: Vec<SameCycleEventDependency>,
) -> ChainEventDispatch {
    dispatch.occurrence.push(if all {
        SameCycleEventRequirement::AfterAll(dependencies)
    } else {
        SameCycleEventRequirement::AfterAny(dependencies)
    });
    dispatch
}

impl<A: SandEvent + 'static, B: SandEvent + 'static> event_group_private::Sealed for (A, B) {
    fn apply(dispatch: ChainEventDispatch, all: bool) -> ChainEventDispatch {
        apply_event_group(
            dispatch,
            all,
            vec![
                SameCycleEventDependency::of::<A>(),
                SameCycleEventDependency::of::<B>(),
            ],
        )
    }
}

impl<A: SandEvent + 'static, B: SandEvent + 'static> SameCycleEventGroup for (A, B) {}

impl<A: SandEvent + 'static, B: SandEvent + 'static, C: SandEvent + 'static>
    event_group_private::Sealed for (A, B, C)
{
    fn apply(dispatch: ChainEventDispatch, all: bool) -> ChainEventDispatch {
        apply_event_group(
            dispatch,
            all,
            vec![
                SameCycleEventDependency::of::<A>(),
                SameCycleEventDependency::of::<B>(),
                SameCycleEventDependency::of::<C>(),
            ],
        )
    }
}

impl<A: SandEvent + 'static, B: SandEvent + 'static, C: SandEvent + 'static> SameCycleEventGroup
    for (A, B, C)
{
}

impl<A: SandEvent + 'static, B: SandEvent + 'static, C: SandEvent + 'static, D: SandEvent + 'static>
    event_group_private::Sealed for (A, B, C, D)
{
    fn apply(dispatch: ChainEventDispatch, all: bool) -> ChainEventDispatch {
        apply_event_group(
            dispatch,
            all,
            vec![
                SameCycleEventDependency::of::<A>(),
                SameCycleEventDependency::of::<B>(),
                SameCycleEventDependency::of::<C>(),
                SameCycleEventDependency::of::<D>(),
            ],
        )
    }
}

impl<A: SandEvent + 'static, B: SandEvent + 'static, C: SandEvent + 'static, D: SandEvent + 'static>
    SameCycleEventGroup for (A, B, C, D)
{
}

impl<
    A: SandEvent + 'static,
    B: SandEvent + 'static,
    C: SandEvent + 'static,
    D: SandEvent + 'static,
    E: SandEvent + 'static,
> event_group_private::Sealed for (A, B, C, D, E)
{
    fn apply(dispatch: ChainEventDispatch, all: bool) -> ChainEventDispatch {
        apply_event_group(
            dispatch,
            all,
            vec![
                SameCycleEventDependency::of::<A>(),
                SameCycleEventDependency::of::<B>(),
                SameCycleEventDependency::of::<C>(),
                SameCycleEventDependency::of::<D>(),
                SameCycleEventDependency::of::<E>(),
            ],
        )
    }
}

impl<
    A: SandEvent + 'static,
    B: SandEvent + 'static,
    C: SandEvent + 'static,
    D: SandEvent + 'static,
    E: SandEvent + 'static,
> SameCycleEventGroup for (A, B, C, D, E)
{
}

impl<
    A: SandEvent + 'static,
    B: SandEvent + 'static,
    C: SandEvent + 'static,
    D: SandEvent + 'static,
    E: SandEvent + 'static,
    F: SandEvent + 'static,
> event_group_private::Sealed for (A, B, C, D, E, F)
{
    fn apply(dispatch: ChainEventDispatch, all: bool) -> ChainEventDispatch {
        apply_event_group(
            dispatch,
            all,
            vec![
                SameCycleEventDependency::of::<A>(),
                SameCycleEventDependency::of::<B>(),
                SameCycleEventDependency::of::<C>(),
                SameCycleEventDependency::of::<D>(),
                SameCycleEventDependency::of::<E>(),
                SameCycleEventDependency::of::<F>(),
            ],
        )
    }
}

impl<
    A: SandEvent + 'static,
    B: SandEvent + 'static,
    C: SandEvent + 'static,
    D: SandEvent + 'static,
    E: SandEvent + 'static,
    F: SandEvent + 'static,
> SameCycleEventGroup for (A, B, C, D, E, F)
{
}

impl<
    A: SandEvent + 'static,
    B: SandEvent + 'static,
    C: SandEvent + 'static,
    D: SandEvent + 'static,
    E: SandEvent + 'static,
    F: SandEvent + 'static,
    G: SandEvent + 'static,
> event_group_private::Sealed for (A, B, C, D, E, F, G)
{
    fn apply(dispatch: ChainEventDispatch, all: bool) -> ChainEventDispatch {
        apply_event_group(
            dispatch,
            all,
            vec![
                SameCycleEventDependency::of::<A>(),
                SameCycleEventDependency::of::<B>(),
                SameCycleEventDependency::of::<C>(),
                SameCycleEventDependency::of::<D>(),
                SameCycleEventDependency::of::<E>(),
                SameCycleEventDependency::of::<F>(),
                SameCycleEventDependency::of::<G>(),
            ],
        )
    }
}

impl<
    A: SandEvent + 'static,
    B: SandEvent + 'static,
    C: SandEvent + 'static,
    D: SandEvent + 'static,
    E: SandEvent + 'static,
    F: SandEvent + 'static,
    G: SandEvent + 'static,
> SameCycleEventGroup for (A, B, C, D, E, F, G)
{
}

impl<
    A: SandEvent + 'static,
    B: SandEvent + 'static,
    C: SandEvent + 'static,
    D: SandEvent + 'static,
    E: SandEvent + 'static,
    F: SandEvent + 'static,
    G: SandEvent + 'static,
    H: SandEvent + 'static,
> event_group_private::Sealed for (A, B, C, D, E, F, G, H)
{
    fn apply(dispatch: ChainEventDispatch, all: bool) -> ChainEventDispatch {
        apply_event_group(
            dispatch,
            all,
            vec![
                SameCycleEventDependency::of::<A>(),
                SameCycleEventDependency::of::<B>(),
                SameCycleEventDependency::of::<C>(),
                SameCycleEventDependency::of::<D>(),
                SameCycleEventDependency::of::<E>(),
                SameCycleEventDependency::of::<F>(),
                SameCycleEventDependency::of::<G>(),
                SameCycleEventDependency::of::<H>(),
            ],
        )
    }
}

impl<
    A: SandEvent + 'static,
    B: SandEvent + 'static,
    C: SandEvent + 'static,
    D: SandEvent + 'static,
    E: SandEvent + 'static,
    F: SandEvent + 'static,
    G: SandEvent + 'static,
    H: SandEvent + 'static,
> SameCycleEventGroup for (A, B, C, D, E, F, G, H)
{
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::EventSetup",
    module = "sand::events",
    summary = "Lifecycle resources a [`SandEvent`] owns: objectives to create at load time, commands to run before each observation, and commands to run after a successful observation (e.g. synchronizing a delta-tracking score).",
    context = "Lifecycle resources a [`SandEvent`] owns: objectives to create at load time, commands to run before each observation, and commands to run after a successful observation (e.g. synchronizing a delta-tracking score). Returned by [`SandEvent::setup`]. When multiple `#[on_event]` handlers subscribe to the same event type, Sand deduplicates the setup so objectives and detector/synchronization functions are emitted once.",
    minecraft = "Objectives run at load; pre-observation commands run before detection and post-observation commands run after each detector pass.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::EventSetup;",
    fields(objectives = "`scoreboard objectives add …` (or other init) commands, run once from `minecraft:load`.", post_observation = "Commands that must run after a successful or completed observation each tick (e.g. copying the current score into a synchronized score).", pre_observation = "Commands that must run before the observation/detection check each tick (e.g. snapshotting a value)."),
)]
/// Lifecycle resources a [`SandEvent`] owns: objectives to create at load time,
/// commands to run before each observation, and commands to run after a
/// successful observation (e.g. synchronizing a delta-tracking score).
///
/// Returned by [`SandEvent::setup`]. When multiple `#[on_event]` handlers
/// subscribe to the same event type, Sand deduplicates the setup so
/// objectives and detector/synchronization functions are emitted once.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventSetup {
    /// `scoreboard objectives add …` (or other init) commands, run once from
    /// `minecraft:load`.
    pub objectives: Vec<String>,
    /// Commands that must run before the observation/detection check each
    /// tick (e.g. snapshotting a value).
    pub pre_observation: Vec<String>,
    /// Commands that must run after a successful or completed observation
    /// each tick (e.g. copying the current score into a synchronized score).
    ///
    /// These run unconditionally after the detection line(s), regardless of
    /// whether the condition matched this tick, so tracked state always
    /// advances — see [`TickEventDispatch`] for the ordering guarantee.
    pub post_observation: Vec<String>,
}

impl EventSetup {
    /// An empty setup — no objectives or lifecycle commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::EventSetup::none",
        module = "sand::events",
        kind = "method",
        summary = "An empty setup — no objectives or lifecycle commands.",
        context = "An empty setup — no objectives or lifecycle commands. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It emits no load or detector setup resources.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "An `EventSetup` configured for an empty setup — no objectives or lifecycle commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let event_setup = sand::events::EventSetup::none();\n}",
    )]
    pub fn none() -> Self {
        Self::default()
    }

    /// Whether every lifecycle-owned collection is empty — the single
    /// canonical check for "this event owns no setup/lifecycle resources".
    /// Covers all fields by construction (`self == &Self::none()`) rather
    /// than re-listing them, so a future field addition to `EventSetup`
    /// cannot silently bypass this check the way an independently
    /// maintained per-field comparison could.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::EventSetup::is_empty",
        module = "sand::events",
        kind = "method",
        summary = "Whether every lifecycle-owned collection is empty — the single canonical check for \"this event owns no setup/lifecycle resources\". Covers all fields by construction (`self == &Self::none()`) rather than re-listing them, so a future field addition to `EventSetup` cannot silently bypass this check the way an independently maintained per-field comparison could.",
        context = "Whether every lifecycle-owned collection is empty — the single canonical check for \"this event owns no setup/lifecycle resources\". Covers all fields by construction (`self == &Self::none()`) rather than re-listing them, so a future field addition to `EventSetup` cannot silently bypass this check the way an independently maintained per-field comparison could. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand uses this distinction when validating dispatch composition.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "`true` when the documented condition holds to determine whether every lifecycle-owned collection is empty — the single canonical check for \"this event owns no setup/lifecycle resources\". Covers all fields by construction (`self == &Self::none()`) rather than re-listing them, so a future field addition to `EventSetup` cannot silently bypass this check the way an independently maintained per-field comparison could; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(event_setup_value: &sand::events::EventSetup)  {\n    let is_is_empty = event_setup_value.is_empty();\n}",
    )]
    pub fn is_empty(&self) -> bool {
        self == &Self::none()
    }

    /// The name of the first non-empty lifecycle category, in the order
    /// they run (`objectives` at load, then `pre_observation`, then
    /// `post_observation`), or `None` if [`is_empty`](Self::is_empty).
    /// Intended for diagnostics that need to name which category blocked an
    /// operation — not a substitute for [`is_empty`](Self::is_empty), which
    /// remains the authoritative full-coverage check.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::EventSetup::first_non_empty_category",
        module = "sand::events",
        kind = "method",
        summary = "The name of the first non-empty lifecycle category, in the order they run (`objectives` at load, then `pre_observation`, then `post_observation`), or `None` if [`is_empty`](Self::is_empty). Intended for diagnostics that need to name which category blocked an operation — not a substitute for [`is_empty`](Self::is_empty), which remains the authoritative full-coverage check.",
        context = "The name of the first non-empty lifecycle category, in the order they run (`objectives` at load, then `pre_observation`, then `post_observation`), or `None` if [`is_empty`](Self::is_empty). Intended for diagnostics that need to name which category blocked an operation — not a substitute for [`is_empty`](Self::is_empty), which remains the authoritative full-coverage check. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It supports deterministic diagnostics for generated load and tick wiring.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The matching value used to use the name of the first non-empty lifecycle category, in the order they run (`objectives` at load, then `pre_observation`, then `post_observation`), or `None` if [`is_empty`](Self::is_empty). Intended for diagnostics that need to name which category blocked an operation — not a substitute for [`is_empty`](Self::is_empty), which remains the authoritative full-coverage check, or `None` when that value is unavailable.",
        example = "use sand::prelude::*;\n\nfn demonstrate(event_setup_value: &sand::events::EventSetup)  {\n    let first_non_empty_category = event_setup_value.first_non_empty_category();\n}",
    )]
    pub fn first_non_empty_category(&self) -> Option<&'static str> {
        if !self.objectives.is_empty() {
            Some("objectives")
        } else if !self.pre_observation.is_empty() {
            Some("pre_observation")
        } else if !self.post_observation.is_empty() {
            Some("post_observation")
        } else {
            None
        }
    }
}

/// Explicit result of expanding a [`TickEventDispatch`]'s conditions into
/// concrete `execute` clause plans.
///
/// This exists specifically so "no conditions were declared" and "the
/// condition expands into more than one OR-alternative execute plan" can
/// never be conflated into a single `None` — every caller must handle both
/// cases explicitly.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TickExecutionPlans {
    /// No `when`/`unless` conditions were declared. The event dispatches
    /// unconditionally every tick — no `if`/`unless` clauses at all, e.g.
    /// `execute as @a at @s run function ...`.
    Unconditional,
    /// One or more OR-alternative execute plans. Each inner `Vec<String>` is
    /// an ordered list of `if`/`unless` clause strings (e.g.
    /// `"if score @s mana matches 25.."`) to chain into one `execute`
    /// command.
    ///
    /// More than one entry means the underlying condition can match through
    /// multiple alternative branches (e.g. a top-level `Any`). Because more
    /// than one plan can match the same subject on the same tick, callers
    /// dispatching from multiple plans must apply an explicit
    /// once-per-subject-per-tick policy rather than invoking the handler
    /// once per matching plan.
    Plans(Vec<Vec<String>>),
}

/// Internal typed counterpart retained through event export lowering.
#[derive(Debug, Clone)]
pub(crate) enum TickExecutionIrPlans {
    Unconditional,
    Plans(Vec<crate::condition::ExecuteIrPlan>),
}

impl TickExecutionIrPlans {
    pub(crate) fn render_compat(self) -> TickExecutionPlans {
        match self {
            Self::Unconditional => TickExecutionPlans::Unconditional,
            Self::Plans(plans) => TickExecutionPlans::Plans(
                plans
                    .into_iter()
                    .map(|plan| plan.into_iter().map(|clause| clause.render()).collect())
                    .collect(),
            ),
        }
    }
}

#[allow(dead_code)]
impl TickExecutionPlans {
    /// `true` if this is [`Unconditional`](Self::Unconditional).
    pub(crate) fn is_unconditional(&self) -> bool {
        matches!(self, Self::Unconditional)
    }

    /// The OR-alternative plans, or an empty slice for
    /// [`Unconditional`](Self::Unconditional).
    pub(crate) fn plans(&self) -> &[Vec<String>] {
        match self {
            Self::Unconditional => &[],
            Self::Plans(p) => p,
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::TickEventDispatch",
    module = "sand::events",
    summary = "Structured, typed tick-poll dispatch definition.",
    context = "Structured, typed tick-poll dispatch definition. Built via [`SandEventDispatch::tick`]. Conditions are composed from the same [`Condition`](sand::condition::Condition) IR used throughout Sand (score comparisons, flags, predicates, entity checks, and the explicit [`Condition::raw`](sand::condition::Condition::raw) escape hatch) rather than hand-formatted strings.",
    minecraft = "Sand emits per-player execute checks and dispatches a handler only while all typed conditions hold.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::TickEventDispatch;",
)]
/// Structured, typed tick-poll dispatch definition.
///
/// Built via [`SandEventDispatch::tick`]. Conditions are composed from the
/// same [`Condition`](crate::condition::Condition) IR used throughout Sand
/// (score comparisons, flags, predicates, entity checks, and the explicit
/// [`Condition::raw`](crate::condition::Condition::raw) escape hatch) rather
/// than hand-formatted strings.
///
/// ```rust,ignore
/// use sand_core::events::{SandEvent, SandEventDispatch};
/// use sand_core::state::ScoreVar;
///
/// static JUMPS: ScoreVar<i32> = ScoreVar::new("jumps");
/// static SYNC_JUMPS: ScoreVar<i32> = ScoreVar::new("sync_jumps");
///
/// pub struct PlayerJumpEvent;
///
/// impl SandEvent for PlayerJumpEvent {
///     fn dispatch() -> SandEventDispatch {
///         SandEventDispatch::tick()
///             .as_players()
///             .when(SYNC_JUMPS.of("@s").lt_score(JUMPS.of("@s")))
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TickEventDispatch {
    /// The execution scope handlers are dispatched under.
    pub(crate) scope: TickScope,
    /// Positive conditions — all must hold (ANDed).
    pub(crate) when: Vec<crate::condition::Condition>,
    /// Negative conditions — none may hold (ANDed as `unless`).
    pub(crate) unless: Vec<crate::condition::Condition>,
}

impl TickEventDispatch {
    /// Evaluate as each online player. Currently the only supported scope;
    /// present for API clarity and forward-compatibility with future scopes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickEventDispatch::as_players",
        module = "sand::events",
        kind = "method",
        summary = "Evaluate as each online player. Currently the only supported scope; present for API clarity and forward-compatibility with future scopes.",
        context = "Evaluate as each online player. Currently the only supported scope; present for API clarity and forward-compatibility with future scopes. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand renders execute as @a at @s for the detector.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `TickEventDispatch` value with the documented change applied to evaluate as each online player. Currently the only supported scope; present for API clarity and forward-compatibility with future scopes.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tick_event_dispatch_value: sand::events::TickEventDispatch)  {\n    let updated_tick_event_dispatch = tick_event_dispatch_value.as_players();\n}",
    )]
    pub fn as_players(mut self) -> Self {
        self.scope = TickScope::Players;
        self
    }

    /// Add a positive condition — the event fires only while this holds.
    ///
    /// Multiple calls are ANDed together.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickEventDispatch::when",
        module = "sand::events",
        kind = "method",
        summary = "Add a positive condition — the event fires only while this holds.",
        context = "Add a positive condition — the event fires only while this holds. Multiple calls are ANDed together.",
        minecraft = "All when clauses become conjunctions in the generated execute test.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(condition = "`condition` provides the condition that gates the operation used to add a positive condition — the event fires only while this holds."),
        returns = "The `TickEventDispatch` value with the documented change applied to add a positive condition — the event fires only while this holds.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tick_event_dispatch_value: sand::events::TickEventDispatch, condition: impl Into < sand::condition::Condition >)  {\n    let updated_tick_event_dispatch = tick_event_dispatch_value.when(condition);\n}",
    )]
    pub fn when(mut self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.when.push(condition.into());
        self
    }

    /// Ergonomic alias for [`when`](Self::when).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickEventDispatch::if_",
        module = "sand::events",
        kind = "method",
        summary = "Ergonomic alias for [`when`](Self::when).",
        context = "Ergonomic alias for [`when`](Self::when). This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It has the same conjunction semantics as when.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(condition = "`condition` provides the condition that gates the operation used to use ergonomic alias for [`when`](Self::when)."),
        returns = "The `TickEventDispatch` value with the documented change applied to use ergonomic alias for [`when`](Self::when).",
        example = "use sand::prelude::*;\n\nfn demonstrate(tick_event_dispatch_value: sand::events::TickEventDispatch, condition: impl Into < sand::condition::Condition >)  {\n    let updated_tick_event_dispatch = tick_event_dispatch_value.if_(condition);\n}",
    )]
    pub fn if_(self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.when(condition)
    }

    /// Add a negative condition — the event does not fire while this holds.
    ///
    /// Multiple calls are ANDed together (i.e. every `unless` condition must
    /// fail to hold).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickEventDispatch::unless",
        module = "sand::events",
        kind = "method",
        summary = "Add a negative condition — the event does not fire while this holds.",
        context = "Add a negative condition — the event does not fire while this holds. Multiple calls are ANDed together (i.e. every `unless` condition must fail to hold).",
        minecraft = "Each clause becomes an execute-unless requirement.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(condition = "`condition` provides the condition that gates the operation used to add a negative condition — the event does not fire while this holds."),
        returns = "The `TickEventDispatch` value with the documented change applied to add a negative condition — the event does not fire while this holds.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tick_event_dispatch_value: sand::events::TickEventDispatch, condition: impl Into < sand::condition::Condition >)  {\n    let updated_tick_event_dispatch = tick_event_dispatch_value.unless(condition);\n}",
    )]
    pub fn unless(mut self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.unless.push(condition.into());
        self
    }

    /// No-op cadence marker: the event is checked every tick.
    ///
    /// Present so dispatch definitions can be explicit about cadence; there
    /// is currently no other supported cadence.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickEventDispatch::every_tick",
        module = "sand::events",
        kind = "method",
        summary = "No-op cadence marker: the event is checked every tick.",
        context = "No-op cadence marker: the event is checked every tick. Present so dispatch definitions can be explicit about cadence; there is currently no other supported cadence.",
        minecraft = "The detector runs once per Minecraft tick.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `TickEventDispatch` value with the documented change applied to no-op cadence marker: the event is checked every tick.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tick_event_dispatch_value: sand::events::TickEventDispatch)  {\n    let updated_tick_event_dispatch = tick_event_dispatch_value.every_tick();\n}",
    )]
    pub fn every_tick(self) -> Self {
        self
    }

    /// Combine `when`/`unless` into a single [`Condition`](crate::condition::Condition),
    /// or `None` if no conditions were declared (dispatch is unconditional).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::TickEventDispatch::combined_condition",
        module = "sand::events",
        kind = "method",
        summary = "Combine `when`/`unless` into a single [`Condition`](sand::condition::Condition), or `None` if no conditions were declared (dispatch is unconditional).",
        context = "Combine `when`/`unless` into a single [`Condition`](sand::condition::Condition), or `None` if no conditions were declared (dispatch is unconditional). This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The resulting condition corresponds to generated execute clauses, or None for an unconditional detector.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The matching value used to combine `when`/`unless` into a single [`Condition`](sand::condition::Condition), or `None` if no conditions were declared (dispatch is unconditional), or `None` when that value is unavailable.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tick_event_dispatch_value: &sand::events::TickEventDispatch)  {\n    let combined_condition = tick_event_dispatch_value.combined_condition();\n}",
    )]
    pub fn combined_condition(&self) -> Option<crate::condition::Condition> {
        if self.when.is_empty() && self.unless.is_empty() {
            return None;
        }
        let mut combined = if self.when.is_empty() {
            crate::condition::Condition::all([])
        } else {
            crate::condition::Condition::all(self.when.clone())
        };
        for u in &self.unless {
            combined = combined.and_not(u.clone());
        }
        Some(combined)
    }

    /// Expand this dispatch's conditions into explicit [`TickExecutionPlans`].
    ///
    /// Unlike a bare `Option<String>`, this never conflates "no conditions —
    /// dispatch unconditionally" with "the condition expanded into more than
    /// one OR-alternative execute plan." Callers must handle both
    /// [`TickExecutionPlans::Unconditional`] and every entry of
    /// [`TickExecutionPlans::Plans`] explicitly.
    #[allow(dead_code)]
    pub(crate) fn execution_plans(&self) -> TickExecutionPlans {
        self.execution_ir_plans().render_compat()
    }

    pub(crate) fn execution_ir_plans(&self) -> TickExecutionIrPlans {
        match self.combined_condition() {
            None => TickExecutionIrPlans::Unconditional,
            Some(combined) => TickExecutionIrPlans::Plans(combined.to_ir_plans(false)),
        }
    }
}

impl From<TickEventDispatch> for SandEventDispatch {
    fn from(tick: TickEventDispatch) -> Self {
        SandEventDispatch::Tick(tick)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ChainEventDispatch",
    module = "sand::events",
    summary = "Structured, typed same-cycle chained dispatch definition.",
    context = "Structured, typed same-cycle chained dispatch definition. Built via [`SandEventDispatch::chain`]. Declares that this event is evaluated only from its parent [`SandEvent`]'s successful dispatch cycle — same execution subject (`@s`), same position, same tick — rather than independently re-detecting the parent's condition. The parent is identified by function-pointer factories rather than a constructed value so the parent marker type never needs to be instantiated and generic parent/child families keep distinct identities. See the `#[on_event]` macro, which supplies these factories automatically from a `SandEvent::dispatch() -> SandEventDispatch::chain::<Parent>()` call — you should not need to construct this struct's function pointers by hand.",
    minecraft = "Sand propagates the parent subject in the same dispatch cycle and can retain bounded parent marks across ticks.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::ChainEventDispatch;",
)]
/// Structured, typed same-cycle chained dispatch definition.
///
/// Built via [`SandEventDispatch::chain`]. Declares that this event is
/// evaluated only from its parent [`SandEvent`]'s successful dispatch cycle —
/// same execution subject (`@s`), same position, same tick — rather than
/// independently re-detecting the parent's condition.
///
/// The parent is identified by function-pointer factories rather than a
/// constructed value so the parent marker type never needs to be
/// instantiated and generic parent/child families keep distinct identities.
/// See the `#[on_event]` macro, which supplies these factories automatically
/// from a `SandEvent::dispatch() -> SandEventDispatch::chain::<Parent>()`
/// call — you should not need to construct this struct's function pointers
/// by hand.
///
/// ```rust,ignore
/// use sand_core::events::{SandEvent, SandEventDispatch};
/// use sand_core::condition::Condition;
///
/// pub struct JumpedOnElevator;
///
/// impl SandEvent for JumpedOnElevator {
///     fn dispatch() -> SandEventDispatch {
///         SandEventDispatch::chain::<PlayerJumpEvent>()
///             .when(Condition::raw("block ~ ~-1 ~ minecraft:white_wool"))
///             .into()
///     }
/// }
/// ```
pub struct ChainEventDispatch {
    /// Explicit same-cycle occurrence clauses. Clauses are conjunctive;
    /// `AfterAny` is disjunctive only within its own parent group.
    pub(crate) occurrence: Vec<SameCycleEventRequirement>,
    /// Persistent current-state requirements, kept distinct from the
    /// same-cycle occurrence parent and from ordinary anonymous conditions.
    pub(crate) persistent: Vec<PersistentEventDependency>,
    /// Bounded cross-tick correlation requirements. Distinct from `occurrence`
    /// (same-cycle only) and `persistent` (current state, no occurrence). See
    /// [`ChainEventDispatch::within`].
    pub(crate) bounded: Vec<BoundedEventDependency>,
    /// Positive conditions — all must hold (ANDed) for this child to fire
    /// once its occurrence requirements are satisfied.
    pub(crate) conditions: Vec<crate::condition::Condition>,
    /// Negative conditions — none may hold.
    pub(crate) excluded_conditions: Vec<crate::condition::Condition>,
}

impl ChainEventDispatch {
    /// Require one additional event to have fired for the same subject during
    /// the current event cycle.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ChainEventDispatch::after",
        module = "sand::events",
        kind = "method",
        summary = "Require one additional event to have fired for the same subject during the current event cycle.",
        context = "Require one additional event to have fired for the same subject during the current event cycle. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The child is wired after the parent's successful generated dispatch, without an independent detector.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `ChainEventDispatch` value with the documented change applied to require one additional event to have fired for the same subject during the current event cycle.",
        example = "use sand::prelude::*;\n\nfn demonstrate<E : sand::events::SandEvent + 'static>(chain_event_dispatch_value: sand::events::ChainEventDispatch)  {\n    let updated_chain_event_dispatch = chain_event_dispatch_value.after::<E>();\n}",
    )]
    pub fn after<E: SandEvent + 'static>(mut self) -> Self {
        self.occurrence.push(SameCycleEventRequirement::After(
            SameCycleEventDependency::of::<E>(),
        ));
        self
    }

    /// Require at least one event in `G` to have fired for the same subject
    /// during the current event cycle.
    ///
    /// `G` is a tuple of two through eight concrete [`SandEvent`] types.
    /// Multiple `after_any` groups in one definition are rejected at export
    /// because their coalescing boundary would otherwise be ambiguous.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ChainEventDispatch::after_any",
        module = "sand::events",
        kind = "method",
        summary = "Require at least one event in `G` to have fired for the same subject during the current event cycle.",
        context = "Require at least one event in `G` to have fired for the same subject during the current event cycle. `G` is a tuple of two through eight concrete [`SandEvent`] types. Multiple `after_any` groups in one definition are rejected at export because their coalescing boundary would otherwise be ambiguous.",
        minecraft = "`G` is a tuple of two through eight concrete [`SandEvent`] types. Multiple `after_any` groups in one definition are rejected at export because their coalescing boundary would otherwise be ambiguous.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `ChainEventDispatch` value with the documented change applied to require at least one event in `G` to have fired for the same subject during the current event cycle.",
        example = "use sand::prelude::*;\n\nfn demonstrate<G : sand::events::SameCycleEventGroup + 'static>(chain_event_dispatch_value: sand::events::ChainEventDispatch)  {\n    let updated_chain_event_dispatch = chain_event_dispatch_value.after_any::<G>();\n}",
    )]
    pub fn after_any<G: SameCycleEventGroup>(self) -> Self {
        <G as event_group_private::Sealed>::apply(self, false)
    }

    /// Require every event in `G` to have fired for the same subject during
    /// the current event cycle.
    ///
    /// `G` is a tuple of two through eight concrete [`SandEvent`] types.
    /// Multiple `after_all` groups in one definition are rejected at export.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ChainEventDispatch::after_all",
        module = "sand::events",
        kind = "method",
        summary = "Require every event in `G` to have fired for the same subject during the current event cycle.",
        context = "Require every event in `G` to have fired for the same subject during the current event cycle. `G` is a tuple of two through eight concrete [`SandEvent`] types. Multiple `after_all` groups in one definition are rejected at export.",
        minecraft = "`G` is a tuple of two through eight concrete [`SandEvent`] types. Multiple `after_all` groups in one definition are rejected at export.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `ChainEventDispatch` value with the documented change applied to require every event in `G` to have fired for the same subject during the current event cycle.",
        example = "use sand::prelude::*;\n\nfn demonstrate<G : sand::events::SameCycleEventGroup + 'static>(chain_event_dispatch_value: sand::events::ChainEventDispatch)  {\n    let updated_chain_event_dispatch = chain_event_dispatch_value.after_all::<G>();\n}",
    )]
    pub fn after_all<G: SameCycleEventGroup>(self) -> Self {
        <G as event_group_private::Sealed>::apply(self, true)
    }

    /// Require `E`'s persistent state to be true when this child is considered.
    ///
    /// This does not require `E` to have fired in the same cycle and does not
    /// invoke `E`'s detector. Multiple calls are conjunctive and duplicate
    /// requirements for the same concrete type are deduplicated at export.
    ///
    /// ```rust,no_run
    /// use sand_core::events::{
    ///     PlayerSneakEvent, SandEvent, SandEventDispatch,
    /// };
    ///
    /// struct ParentOccurrence;
    /// impl SandEvent for ParentOccurrence {
    ///     fn dispatch() -> impl Into<SandEventDispatch> {
    ///         SandEventDispatch::tick().as_players()
    ///     }
    /// }
    ///
    /// let child = SandEventDispatch::chain::<ParentOccurrence>()
    ///     .while_::<PlayerSneakEvent>();
    /// # let _: SandEventDispatch = child.into();
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ChainEventDispatch::while_",
        module = "sand::events",
        kind = "method",
        summary = "Require `E`'s persistent state to be true when this child is considered.",
        context = "Require `E`'s persistent state to be true when this child is considered. This does not require `E` to have fired in the same cycle and does not invoke `E`'s detector. Multiple calls are conjunctive and duplicate requirements for the same concrete type are deduplicated at export.",
        minecraft = "This does not require `E` to have fired in the same cycle and does not invoke `E`'s detector. Multiple calls are conjunctive and duplicate requirements for the same concrete type are deduplicated at export.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `ChainEventDispatch` value with the documented change applied to require `E`'s persistent state to be true when this child is considered.",
        example = "use sand::events::{\nPlayerSneakEvent, SandEvent, SandEventDispatch,\n};\nstruct ParentOccurrence;\nimpl SandEvent for ParentOccurrence {\nfn dispatch() -> impl Into<SandEventDispatch> {\nSandEventDispatch::tick().as_players()\n}\n}\nlet child = SandEventDispatch::chain::<ParentOccurrence>()\n.while_::<PlayerSneakEvent>();",
    )]
    pub fn while_<E: PersistentSandEvent + 'static>(mut self) -> Self {
        self.persistent.push(PersistentEventDependency {
            event_type_id: std::any::TypeId::of::<E>,
            event_type_name: std::any::type_name::<E>,
            event_dispatch: || E::dispatch().into(),
            event_setup: dependency_setup::<E>,
            make_condition: E::persistent_condition,
        });
        self
    }

    /// Require `E` to have fired for the same subject during the current
    /// cycle or within the previous `window.ticks() - 1` completed tick
    /// boundaries. See [`TickWindow`] for the exact boundary convention.
    ///
    /// Unlike `after`, `E`'s occurrence may have happened on an earlier tick.
    /// Unlike `while_`, `E` is an occurrence, not a directly queryable
    /// current-state condition — its own detector still runs and its
    /// same-cycle occurrence mark still drives the age tracked for this
    /// window. Distinct `.within` calls for different concrete parent types
    /// are conjunctive. A repeated `.within::<E>(window)` call with the same
    /// `window` is deduplicated; a repeated call for the same `E` with a
    /// **different** `window` is rejected at export as an unrepresentable
    /// conflicting declaration — declare a second concrete parent type
    /// instead if two different windows against the same underlying event
    /// are genuinely required.
    ///
    /// ```rust,no_run
    /// use sand_core::events::{SandEvent, SandEventDispatch, TickWindow};
    ///
    /// struct CurrentEvent;
    /// impl SandEvent for CurrentEvent {
    ///     fn dispatch() -> impl Into<SandEventDispatch> {
    ///         SandEventDispatch::tick().as_players()
    ///     }
    /// }
    ///
    /// struct PriorEvent;
    /// impl SandEvent for PriorEvent {
    ///     fn dispatch() -> impl Into<SandEventDispatch> {
    ///         SandEventDispatch::tick().as_players()
    ///     }
    /// }
    ///
    /// let child = SandEventDispatch::compose()
    ///     .after::<CurrentEvent>()
    ///     .within::<PriorEvent>(TickWindow::new(20).expect("nonzero, in range"));
    /// # let _: SandEventDispatch = child.into();
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ChainEventDispatch::within",
        module = "sand::events",
        kind = "method",
        summary = "Require `E` to have fired for the same subject during the current cycle or within the previous `window.ticks() - 1` completed tick boundaries. See [`TickWindow`] for the exact boundary convention.",
        context = "Require `E` to have fired for the same subject during the current cycle or within the previous `window.ticks() - 1` completed tick boundaries. See [`TickWindow`] for the exact boundary convention. Unlike `after`, `E`'s occurrence may have happened on an earlier tick. Unlike `while_`, `E` is an occurrence, not a directly queryable current-state condition — its own detector still runs and its same-cycle occurrence mark still drives the age tracked for this window. Distinct `.within` calls for different concrete parent types are conjunctive. A repeated `.within::<E>(window)` call with the same `window` is deduplicated; a repeated call for the same `E` with a different `window` is rejected at export as an unrepresentable conflicting declaration — declare a second concrete parent type instead if two different windows against the same underlying event are genuinely required.",
        minecraft = "Unlike `after`, `E`'s occurrence may have happened on an earlier tick. Unlike `while_`, `E` is an occurrence, not a directly queryable current-state condition — its own detector still runs and its same-cycle occurrence mark still drives the age tracked for this window. Distinct `.within` calls for different concrete parent types are conjunctive. A repeated `.within::<E>(window)` call with the same `window` is deduplicated; a repeated call for the same `E` with a different `window` is rejected at export as an unrepresentable conflicting declaration — declare a second concrete parent type instead if two different windows against the same underlying event are genuinely required.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(window = "Unlike `after`, `E`'s occurrence may have happened on an earlier tick. Unlike `while_`, `E` is an occurrence, not a directly queryable current-state condition — its own detector still runs and its same-cycle occurrence mark still drives the age tracked for this window. Distinct `.within` calls for different concrete parent types are conjunctive. A repeated `.within::<E>(window)` call with the same `window` is deduplicated; a repeated call for the same `E` with a different `window` is rejected at export as an unrepresentable conflicting declaration — declare a second concrete parent type instead if two different windows against the same underlying event are genuinely required."),
        returns = "The `ChainEventDispatch` value with the documented change applied to require `E` to have fired for the same subject during the current cycle or within the previous `window.ticks() - 1` completed tick boundaries. See [`TickWindow`] for the exact boundary convention.",
        example = "use {sand::events::SandEvent, sand::events::SandEventDispatch, sand::events::TickWindow};\nstruct CurrentEvent;\nimpl SandEvent for CurrentEvent {\nfn dispatch() -> impl Into<SandEventDispatch> {\nSandEventDispatch::tick().as_players()\n}\n}\nstruct PriorEvent;\nimpl SandEvent for PriorEvent {\nfn dispatch() -> impl Into<SandEventDispatch> {\nSandEventDispatch::tick().as_players()\n}\n}\nlet child = SandEventDispatch::compose()\n.after::<CurrentEvent>()\n.within::<PriorEvent>(TickWindow::new(20).expect(\"nonzero, in range\"));",
    )]
    pub fn within<E: SandEvent + 'static>(mut self, window: TickWindow) -> Self {
        self.bounded.push(BoundedEventDependency {
            event_type_id: std::any::TypeId::of::<E>,
            event_type_name: std::any::type_name::<E>,
            event_dispatch: || E::dispatch().into(),
            event_setup: dependency_setup::<E>,
            window,
        });
        self
    }

    /// Add a positive condition — the child fires only while this holds, in
    /// addition to the parent having fired this cycle.
    ///
    /// Multiple calls are ANDed together.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ChainEventDispatch::when",
        module = "sand::events",
        kind = "method",
        summary = "Add a positive condition — the child fires only while this holds, in addition to the parent having fired this cycle.",
        context = "Add a positive condition — the child fires only while this holds, in addition to the parent having fired this cycle. Multiple calls are ANDed together.",
        minecraft = "The condition is evaluated after its parent relationship has been satisfied.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(condition = "`condition` provides the condition that gates the operation used to add a positive condition — the child fires only while this holds, in addition to the parent having fired this cycle."),
        returns = "The `ChainEventDispatch` value with the documented change applied to add a positive condition — the child fires only while this holds, in addition to the parent having fired this cycle.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chain_event_dispatch_value: sand::events::ChainEventDispatch, condition: impl Into < sand::condition::Condition >)  {\n    let updated_chain_event_dispatch = chain_event_dispatch_value.when(condition);\n}",
    )]
    pub fn when(mut self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    /// Ergonomic alias for [`when`](Self::when).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ChainEventDispatch::if_",
        module = "sand::events",
        kind = "method",
        summary = "Ergonomic alias for [`when`](Self::when).",
        context = "Ergonomic alias for [`when`](Self::when). This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It has the same dispatch semantics as when.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(condition = "`condition` provides the condition that gates the operation used to use ergonomic alias for [`when`](Self::when)."),
        returns = "The `ChainEventDispatch` value with the documented change applied to use ergonomic alias for [`when`](Self::when).",
        example = "use sand::prelude::*;\n\nfn demonstrate(chain_event_dispatch_value: sand::events::ChainEventDispatch, condition: impl Into < sand::condition::Condition >)  {\n    let updated_chain_event_dispatch = chain_event_dispatch_value.if_(condition);\n}",
    )]
    pub fn if_(self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.when(condition)
    }

    /// Add a negative condition — the child does not fire while this holds.
    ///
    /// Multiple calls are ANDed together (i.e. every `unless` condition must
    /// fail to hold).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ChainEventDispatch::unless",
        module = "sand::events",
        kind = "method",
        summary = "Add a negative condition — the child does not fire while this holds.",
        context = "Add a negative condition — the child does not fire while this holds. Multiple calls are ANDed together (i.e. every `unless` condition must fail to hold).",
        minecraft = "The child dispatches only when the condition does not hold.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(condition = "`condition` provides the condition that gates the operation used to add a negative condition — the child does not fire while this holds."),
        returns = "The `ChainEventDispatch` value with the documented change applied to add a negative condition — the child does not fire while this holds.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chain_event_dispatch_value: sand::events::ChainEventDispatch, condition: impl Into < sand::condition::Condition >)  {\n    let updated_chain_event_dispatch = chain_event_dispatch_value.unless(condition);\n}",
    )]
    pub fn unless(mut self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.excluded_conditions.push(condition.into());
        self
    }

    /// Combine `when`/`unless` into a single [`Condition`](crate::condition::Condition),
    /// or `None` if no conditions were declared (the child fires
    /// unconditionally whenever its parent fires).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ChainEventDispatch::combined_condition",
        module = "sand::events",
        kind = "method",
        summary = "Combine `when`/`unless` into a single [`Condition`](sand::condition::Condition), or `None` if no conditions were declared (the child fires unconditionally whenever its parent fires).",
        context = "Combine `when`/`unless` into a single [`Condition`](sand::condition::Condition), or `None` if no conditions were declared (the child fires unconditionally whenever its parent fires). This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It excludes parent occurrence requirements, which Sand lowers separately.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The matching value used to combine `when`/`unless` into a single [`Condition`](sand::condition::Condition), or `None` if no conditions were declared (the child fires unconditionally whenever its parent fires), or `None` when that value is unavailable.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chain_event_dispatch_value: &sand::events::ChainEventDispatch)  {\n    let combined_condition = chain_event_dispatch_value.combined_condition();\n}",
    )]
    pub fn combined_condition(&self) -> Option<crate::condition::Condition> {
        if self.conditions.is_empty() && self.excluded_conditions.is_empty() {
            return None;
        }
        let mut combined = if self.conditions.is_empty() {
            crate::condition::Condition::all([])
        } else {
            crate::condition::Condition::all(self.conditions.clone())
        };
        for u in &self.excluded_conditions {
            combined = combined.and_not(u.clone());
        }
        Some(combined)
    }

    /// Expand this child's conditions into explicit [`TickExecutionPlans`],
    /// same shape as [`TickEventDispatch::execution_plans`].
    #[allow(dead_code)]
    pub(crate) fn execution_plans(&self) -> TickExecutionPlans {
        self.execution_ir_plans().render_compat()
    }

    #[allow(dead_code)]
    pub(crate) fn execution_ir_plans(&self) -> TickExecutionIrPlans {
        match self.combined_condition() {
            None => TickExecutionIrPlans::Unconditional,
            Some(combined) => TickExecutionIrPlans::Plans(combined.to_ir_plans(false)),
        }
    }
}

impl From<ChainEventDispatch> for SandEventDispatch {
    fn from(chain: ChainEventDispatch) -> Self {
        SandEventDispatch::Chain(chain)
    }
}

#[allow(clippy::large_enum_variant)]
pub(crate) enum EventDispatchRepresentation {
    AdvancementTrigger(crate::AdvancementTrigger),
    TickCondition(String),
    Tick(TickEventDispatch),
    Chain(ChainEventDispatch),
    Tracked(crate::TrackedTransition),
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::SandEventDispatch",
    aliases = ["sand::prelude::SandEventDispatch"],
    module = "sand::events",
    summary = "How a custom [`SandEvent`] is dispatched at runtime.",
    context = "How a custom [`SandEvent`] is dispatched at runtime. Returned by [`SandEvent::dispatch`]. Sand inspects this at build time to generate the correct detection mechanism (advancement JSON or tick loop). Its transport representation is intentionally private; authors select a semantic constructor or return one of the typed builders instead.",
    minecraft = "Returned by [`SandEvent::dispatch`]. Sand inspects this at build time to generate the correct detection mechanism (advancement JSON or tick loop). Its transport representation is intentionally private; authors select a semantic constructor or return one of the typed builders instead.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::SandEventDispatch;",
)]
/// How a custom [`SandEvent`] is dispatched at runtime.
///
/// Returned by [`SandEvent::dispatch`]. Sand inspects this at build time to
/// generate the correct detection mechanism (advancement JSON or tick loop).
/// Its transport representation is intentionally private; authors select a
/// semantic constructor or return one of the typed builders instead.
///
/// ```compile_fail
/// use sand_core::events::SandEventDispatch;
///
/// let dispatch: SandEventDispatch = SandEventDispatch::tick().as_players().into();
/// let SandEventDispatch(_) = dispatch;
/// ```
pub struct SandEventDispatch(EventDispatchRepresentation);

/// Normalized internal representation of a [`SandEventDispatch`], used by the
/// export pipeline and by tests asserting on lowering behavior.
///
/// Every `SandEventDispatch` variant — including the legacy `AdvancementTrigger`
/// and `TickCondition` compatibility constructors — lowers into one of these
/// shapes, so the exporter has a single normalized IR to consume rather
/// than juggling multiple representations.
#[allow(clippy::large_enum_variant, dead_code)]
pub(crate) enum NormalizedEventDispatch {
    /// Advancement-backed dispatch.
    Advancement(crate::AdvancementTrigger),
    /// Tick-poll dispatch, always in the structured [`TickEventDispatch`] shape.
    Tick(TickEventDispatch),
    /// Same-cycle chained dispatch. See [`ChainEventDispatch`].
    Chain(ChainEventDispatch),
    /// Reusable tracked-transition dispatch (#49).
    ///
    /// Not currently supported as a same-cycle chain/compose parent —
    /// `discover()` rejects it with a diagnostic pointing at direct
    /// subscription instead, mirroring how advancement-backed parents were
    /// unsupported before their own dedicated integration (#240 Phase 6).
    /// Tracked graph-parent bridging is tracked as follow-up scope.
    Tracked(crate::TrackedTransition),
}

impl SandEventDispatch {
    /// Dispatch from one typed advancement trigger.
    ///
    /// Sand generates an advancement JSON file and wires the handler function
    /// as its reward. The advancement is revoked after firing by default so it
    /// can trigger again.
    #[allow(non_snake_case)]
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventDispatch::AdvancementTrigger",
        aliases = ["sand::prelude::SandEventDispatch::AdvancementTrigger"],
        module = "sand::events",
        kind = "method",
        summary = "Dispatch from one typed advancement trigger. Sand generates an advancement JSON file and wires the handler function as its reward. The advancement is revoked after firing by default so it can trigger again.",
        context = "Dispatch from one typed advancement trigger. Sand generates an advancement JSON file and wires the handler function as its reward. The advancement is revoked after firing by default so it can trigger again. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand generates an advancement JSON file and wires the handler function as its reward. The advancement is revoked after firing by default so it can trigger again.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(trigger = "`trigger` is used to dispatch from one typed advancement trigger. Sand generates an advancement JSON file and wires the handler function as its reward. The advancement is revoked after firing by default so it can trigger again."),
        returns = "A `SandEventDispatch` that dispatches from one typed advancement trigger. Sand generates an advancement JSON file and wires the handler function as its reward. The advancement is revoked after firing by default so it can trigger again.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trigger: sand::component::AdvancementTrigger)  {\n    let sand_event_dispatch = sand::events::SandEventDispatch::AdvancementTrigger(trigger);\n}",
    )]
    pub fn AdvancementTrigger(trigger: crate::AdvancementTrigger) -> Self {
        Self(EventDispatchRepresentation::AdvancementTrigger(trigger))
    }

    /// Dispatch by polling one explicit `execute if` condition for each player.
    ///
    /// Prefer [`SandEventDispatch::tick`] for typed conditions and lifecycle
    /// resources. This compatibility constructor preserves the raw-condition
    /// escape hatch without exposing Sand's dispatch representation.
    #[allow(non_snake_case)]
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventDispatch::TickCondition",
        aliases = ["sand::prelude::SandEventDispatch::TickCondition"],
        module = "sand::events",
        kind = "method",
        summary = "Dispatch by polling one explicit `execute if` condition for each player.",
        context = "Dispatch by polling one explicit `execute if` condition for each player. Prefer [`SandEventDispatch::tick`] for typed conditions and lifecycle resources. This compatibility constructor preserves the raw-condition escape hatch without exposing Sand's dispatch representation.",
        minecraft = "Sand polls it for each player; prefer the typed tick builder when possible.",
        use_when = ["Prefer [`SandEventDispatch::tick`] for typed conditions and lifecycle resources. This compatibility constructor preserves the raw-condition escape hatch without exposing Sand's dispatch representation."],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(condition = "`condition` provides the condition that gates the operation used to dispatch by polling one explicit `execute if` condition for each player."),
        returns = "A `SandEventDispatch` that dispatches by polling one explicit `execute if` condition for each player.",
        example = "use sand::prelude::*;\n\nfn demonstrate(condition: String)  {\n    let sand_event_dispatch = sand::events::SandEventDispatch::TickCondition(condition);\n}",
    )]
    pub fn TickCondition(condition: String) -> Self {
        Self(EventDispatchRepresentation::TickCondition(condition))
    }

    /// Wrap a structured typed tick-poll dispatch.
    #[allow(non_snake_case)]
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventDispatch::Tick",
        aliases = ["sand::prelude::SandEventDispatch::Tick"],
        module = "sand::events",
        kind = "method",
        summary = "Wrap a structured typed tick-poll dispatch.",
        context = "Wrap a structured typed tick-poll dispatch. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand emits the builder's typed conditions as per-player execute checks.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(tick = "`tick` provides the tick wrapped when creating a structured typed tick-poll dispatch."),
        returns = "A `SandEventDispatch` wrapping a structured typed tick-poll dispatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tick: sand::events::TickEventDispatch)  {\n    let sand_event_dispatch = sand::events::SandEventDispatch::Tick(tick);\n}",
    )]
    pub fn Tick(tick: TickEventDispatch) -> Self {
        Self(EventDispatchRepresentation::Tick(tick))
    }

    /// Wrap a structured event-composition dispatch.
    #[allow(non_snake_case)]
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventDispatch::Chain",
        aliases = ["sand::prelude::SandEventDispatch::Chain"],
        module = "sand::events",
        kind = "method",
        summary = "Wrap a structured event-composition dispatch.",
        context = "Wrap a structured event-composition dispatch. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand lowers parent occurrences and inherited subjects into generated composition functions.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(chain = "`chain` provides the chain wrapped when creating a structured event-composition dispatch."),
        returns = "A `SandEventDispatch` wrapping a structured event-composition dispatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chain: sand::events::ChainEventDispatch)  {\n    let sand_event_dispatch = sand::events::SandEventDispatch::Chain(chain);\n}",
    )]
    pub fn Chain(chain: ChainEventDispatch) -> Self {
        Self(EventDispatchRepresentation::Chain(chain))
    }

    /// Construct a structured, typed tick-poll dispatch builder.
    ///
    /// ```rust,ignore
    /// SandEventDispatch::tick()
    ///     .as_players()
    ///     .when(SYNC_JUMPS.of("@s").lt_score(JUMPS.of("@s")))
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventDispatch::tick",
        aliases = ["sand::prelude::SandEventDispatch::tick"],
        module = "sand::events",
        kind = "method",
        summary = "Construct a structured, typed tick-poll dispatch builder.",
        context = "Construct a structured, typed tick-poll dispatch builder. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The resulting builder emits execute checks for online players.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `TickEventDispatch` value produced to construct a structured, typed tick-poll dispatch builder.",
        example = "SandEventDispatch::tick()\n.as_players()\n.when(SYNC_JUMPS.of(\"@s\").lt_score(JUMPS.of(\"@s\")))",
    )]
    pub fn tick() -> TickEventDispatch {
        TickEventDispatch::default()
    }

    /// Construct a structured, same-cycle chained dispatch builder.
    ///
    /// Declares that this event evaluates only from `Parent`'s successful
    /// dispatch cycle, inheriting its execution subject and position, rather
    /// than independently re-detecting `Parent`'s condition. `Parent` need
    /// not have any direct `#[on_event]` handler of its own — only a `SandEvent`
    /// impl.
    ///
    /// ```rust,ignore
    /// SandEventDispatch::chain::<PlayerJumpEvent>()
    ///     .when(Condition::raw("block ~ ~-1 ~ minecraft:white_wool"))
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventDispatch::chain",
        aliases = ["sand::prelude::SandEventDispatch::chain"],
        module = "sand::events",
        kind = "method",
        summary = "Construct a structured, same-cycle chained dispatch builder.",
        context = "Construct a structured, same-cycle chained dispatch builder. Declares that this event evaluates only from `Parent`'s successful dispatch cycle, inheriting its execution subject and position, rather than independently re-detecting `Parent`'s condition. `Parent` need not have any direct `#[on_event]` handler of its own — only a `SandEvent` impl.",
        minecraft = "The child can run in the parent's successful event cycle with its subject inherited.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `ChainEventDispatch` value produced to construct a structured, same-cycle chained dispatch builder.",
        example = "SandEventDispatch::chain::<PlayerJumpEvent>()\n.when(Condition::raw(\"block ~ ~-1 ~ minecraft:white_wool\"))",
    )]
    pub fn chain<P: SandEvent + 'static>() -> ChainEventDispatch {
        Self::compose().after::<P>()
    }

    /// Construct a same-cycle composition builder without choosing a parent.
    ///
    /// Add at least one [`ChainEventDispatch::after`],
    /// [`ChainEventDispatch::after_any`], or
    /// [`ChainEventDispatch::after_all`] clause before returning it from
    /// [`SandEvent::dispatch`]. Empty compositions are rejected at export.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventDispatch::compose",
        aliases = ["sand::prelude::SandEventDispatch::compose"],
        module = "sand::events",
        kind = "method",
        summary = "Construct a same-cycle composition builder without choosing a parent.",
        context = "Construct a same-cycle composition builder without choosing a parent. Add at least one [`ChainEventDispatch::after`], [`ChainEventDispatch::after_any`], or [`ChainEventDispatch::after_all`] clause before returning it from [`SandEvent::dispatch`]. Empty compositions are rejected at export.",
        minecraft = "Add at least one [`ChainEventDispatch::after`], [`ChainEventDispatch::after_any`], or [`ChainEventDispatch::after_all`] clause before returning it from [`SandEvent::dispatch`]. Empty compositions are rejected at export.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `ChainEventDispatch` value produced to construct a same-cycle composition builder without choosing a parent.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let compose = sand::events::SandEventDispatch::compose();\n}",
    )]
    pub fn compose() -> ChainEventDispatch {
        ChainEventDispatch {
            occurrence: Vec::new(),
            persistent: Vec::new(),
            bounded: Vec::new(),
            conditions: Vec::new(),
            excluded_conditions: Vec::new(),
        }
    }

    /// Start a typed any-parent same-cycle composition.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventDispatch::after_any",
        aliases = ["sand::prelude::SandEventDispatch::after_any"],
        module = "sand::events",
        kind = "method",
        summary = "Start a typed any-parent same-cycle composition.",
        context = "Start a typed any-parent same-cycle composition. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand coalesces tuple occurrence marks in the current cycle.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `ChainEventDispatch` value produced to start a typed any-parent same-cycle composition.",
        example = "use sand::prelude::*;\n\nfn demonstrate<G : sand::events::SameCycleEventGroup + 'static>()  {\n    let after_any = sand::events::SandEventDispatch::after_any::<G>();\n}",
    )]
    pub fn after_any<G: SameCycleEventGroup>() -> ChainEventDispatch {
        Self::compose().after_any::<G>()
    }

    /// Start a typed all-parent same-cycle composition.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventDispatch::after_all",
        aliases = ["sand::prelude::SandEventDispatch::after_all"],
        module = "sand::events",
        kind = "method",
        summary = "Start a typed all-parent same-cycle composition.",
        context = "Start a typed all-parent same-cycle composition. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand waits for every tuple occurrence mark in the same cycle.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `ChainEventDispatch` value produced to start a typed all-parent same-cycle composition.",
        example = "use sand::prelude::*;\n\nfn demonstrate<G : sand::events::SameCycleEventGroup + 'static>()  {\n    let after_all = sand::events::SandEventDispatch::after_all::<G>();\n}",
    )]
    pub fn after_all<G: SameCycleEventGroup>() -> ChainEventDispatch {
        Self::compose().after_all::<G>()
    }

    pub(crate) fn tracked(transition: crate::TrackedTransition) -> Self {
        Self(EventDispatchRepresentation::Tracked(transition))
    }

    pub(crate) fn into_advancement(self) -> Option<crate::AdvancementTrigger> {
        match self.0 {
            EventDispatchRepresentation::AdvancementTrigger(trigger) => Some(trigger),
            _ => None,
        }
    }

    pub(crate) fn into_tick_condition(self) -> Option<String> {
        match self.0 {
            EventDispatchRepresentation::TickCondition(condition) => Some(condition),
            _ => None,
        }
    }

    pub(crate) fn into_tick(self) -> Option<TickEventDispatch> {
        match self.0 {
            EventDispatchRepresentation::Tick(tick) => Some(tick),
            _ => None,
        }
    }

    pub(crate) fn into_chain(self) -> Option<ChainEventDispatch> {
        match self.0 {
            EventDispatchRepresentation::Chain(chain) => Some(chain),
            _ => None,
        }
    }

    pub(crate) fn into_tracked(self) -> Option<crate::TrackedTransition> {
        match self.0 {
            EventDispatchRepresentation::Tracked(transition) => Some(transition),
            _ => None,
        }
    }

    /// Lower this dispatch into the normalized internal IR.
    ///
    /// - `AdvancementTrigger(t)` → `Advancement(t)` unchanged.
    /// - `TickCondition(s)` → `Tick(...)` with `s` carried as a single
    ///   [`Condition::raw`](crate::condition::Condition::raw) `when` clause.
    /// - `Tick(t)` → `Tick(t)` unchanged.
    /// - `Chain(c)` → `Chain(c)` unchanged.
    pub(crate) fn normalize(self) -> NormalizedEventDispatch {
        match self.0 {
            EventDispatchRepresentation::AdvancementTrigger(t) => {
                NormalizedEventDispatch::Advancement(t)
            }
            EventDispatchRepresentation::TickCondition(s) => NormalizedEventDispatch::Tick(
                TickEventDispatch::default().when(crate::condition::Condition::raw(s)),
            ),
            EventDispatchRepresentation::Tick(t) => NormalizedEventDispatch::Tick(t),
            EventDispatchRepresentation::Chain(c) => NormalizedEventDispatch::Chain(c),
            EventDispatchRepresentation::Tracked(t) => NormalizedEventDispatch::Tracked(t),
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::SandEvent",
    aliases = ["sand::prelude::SandEvent"],
    module = "sand::events",
    summary = "Implement this trait on your own type to define a custom Sand event.",
    context = "Implement this trait on your own type to define a custom Sand event. Your concrete type is the single parameter of a custom `#[on_event]` handler. Sand inspects [`dispatch`](Self::dispatch) at build time to emit the appropriate datapack files. This differs from an advancement-backed [`Event<T>`](sand::event::Event) context: a bare `SandEvent` marker is constructed by generated handler code, so subscribed markers should be constructible unit types.",
    minecraft = "Your concrete type is the single parameter of a custom `#[on_event]` handler. Sand inspects [`dispatch`](Self::dispatch) at build time to emit the appropriate datapack files. This differs from an advancement-backed [`Event<T>`](sand::event::Event) context: a bare `SandEvent` marker is constructed by generated handler code, so subscribed markers should be constructible unit types.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::SandEvent;",
)]
/// Implement this trait on your own type to define a custom Sand event.
///
/// Your concrete type is the single parameter of a custom `#[on_event]` handler.
/// Sand inspects [`dispatch`](Self::dispatch) at build time to emit the
/// appropriate datapack files. This differs from an advancement-backed
/// [`Event<T>`](crate::event::Event) context: a bare `SandEvent` marker is
/// constructed by generated handler code, so subscribed markers should be
/// constructible unit types.
///
/// # Example
///
/// ```rust,ignore
/// use sand_core::events::{SandEvent, SandEventDispatch};
/// use sand_core::prelude::*;
/// use sand_macros::on_event;
///
/// /// Fires while the player has the `ready` tag.
/// pub struct PlayerReady;
///
/// impl SandEvent for PlayerReady {
///     fn dispatch() -> SandEventDispatch {
///         SandEventDispatch::tick()
///             .as_players()
///             .when(Condition::raw("entity @s[tag=ready]"))
///             .into()
///     }
/// }
///
/// #[on_event]
/// pub fn on_ready(_event: PlayerReady) {
///     cmd::say("Ready!");
/// }
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` is used as a bare `#[on_event]` handler parameter but does not implement `SandEvent`",
    label = "bare marker parameters require `T: SandEvent`",
    note = "AdvancementEvent-backed events are stateless triggers handled through `Event<T>` \
            (see sand_core::event::AdvancementEvent); SandEvent-backed events define custom \
            tick/advancement dispatch and lifecycle via `impl SandEvent for {Self}`"
)]
pub trait SandEvent {
    /// Return the dispatch strategy for this event type.
    ///
    /// Returns `impl Into<SandEventDispatch>` so both the plain enum
    /// constructors (`SandEventDispatch::AdvancementTrigger(...)`) and the
    /// typed [`SandEventDispatch::tick()`] builder chain (which yields a bare
    /// [`TickEventDispatch`]) can be returned directly, without an explicit
    /// `.into()` at every call site.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEvent::dispatch",
        aliases = ["sand::prelude::SandEvent::dispatch"],
        module = "sand::events",
        summary = "Return the dispatch strategy for this event type.",
        context = "Return the dispatch strategy for this event type. Returns `impl Into<SandEventDispatch>` so both the plain enum constructors (`SandEventDispatch::AdvancementTrigger(...)`) and the typed [`SandEventDispatch::tick()`] builder chain (which yields a bare [`TickEventDispatch`]) can be returned directly, without an explicit `.into()` at every call site.",
        minecraft = "Sand lowers it to the generated runtime mechanism for each subscribed handler.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "Return the dispatch strategy for this event type.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEvent>()  {\n    let dispatch = <T as sand::events::SandEvent>::dispatch();\n}",
    )]
    fn dispatch() -> impl Into<SandEventDispatch>;

    /// Lifecycle resources this event owns: objectives, pre-observation, and
    /// post-observation commands.
    ///
    /// Defaults to [`EventSetup::none`]. Override for events that need to
    /// create scoreboard objectives or run commands around detection — see
    /// [`EventSetup`] for the ordering guarantee (detection always runs
    /// before `post_observation`).
    ///
    /// When several `#[on_event]` handlers subscribe to the same event type,
    /// Sand deduplicates setup by the event's in-process type identity so
    /// objectives and detector functions are only emitted once.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEvent::setup", kind = "trait_method",
        aliases = ["sand::prelude::SandEvent::setup"],
        module = "sand::events",
        summary = "Lifecycle resources this event owns: objectives, pre-observation, and post-observation commands.",
        context = "Lifecycle resources this event owns: objectives, pre-observation, and post-observation commands. Defaults to [`EventSetup::none`]. Override for events that need to create scoreboard objectives or run commands around detection — see [`EventSetup`] for the ordering guarantee (detection always runs before `post_observation`). When several `#[on_event]` handlers subscribe to the same event type, Sand deduplicates setup by the event's in-process type identity so objectives and detector functions are only emitted once.",
        minecraft = "Defaults to [`EventSetup::none`]. Override for events that need to create scoreboard objectives or run commands around detection — see [`EventSetup`] for the ordering guarantee (detection always runs before `post_observation`).",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `EventSetup` value produced to lifecycle resources this event owns: objectives, pre-observation, and post-observation commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEvent>()  {\n    let setup = <T as sand::events::SandEvent>::setup();\n}",
    )]
    fn setup() -> EventSetup {
        EventSetup::none()
    }

    /// Which participant observations this event declares (#230 Phase 10).
    ///
    /// Defaults to [`crate::participant::EventParticipantPlan::none`] — a
    /// genuinely additive default; every existing `SandEvent` implementation
    /// is unaffected by this method's existence. The export pipeline applies
    /// a declared plan automatically for every dispatch backend that can
    /// meaningfully own one — tick-lifecycle/tick-poll dispatch, same-cycle
    /// chained dispatch (#264), and tracked-transition dispatch (#270) all
    /// merge it into the generated detector/handler around the same
    /// pre/post-observation boundary [`EventSetup::with_participants`]
    /// documents; you do not need to call `with_participants` yourself
    /// unless you are hand-assembling an `EventSetup` outside the normal
    /// `#[on_event]` path. See `sand-core/src/participant/plan.rs`'s module doc
    /// for the exact lifecycle ordering per backend.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEvent::participants", kind = "trait_method",
        aliases = ["sand::prelude::SandEvent::participants"],
        module = "sand::events",
        summary = "Which participant observations this event declares (#230 Phase 10).",
        context = "Which participant observations this event declares (#230 Phase 10). Defaults to [`sand::participant::EventParticipantPlan::none`] — a genuinely additive default; every existing `SandEvent` implementation is unaffected by this method's existence. The export pipeline applies a declared plan automatically for every dispatch backend that can meaningfully own one — tick-lifecycle/tick-poll dispatch, same-cycle chained dispatch (#264), and tracked-transition dispatch (#270) all merge it into the generated detector/handler around the same pre/post-observation boundary [`EventSetup::with_participants`] documents; you do not need to call `with_participants` yourself unless you are hand-assembling an `EventSetup` outside the normal `#[on_event]` path. See `sand-core/src/participant/plan.rs`'s module doc for the exact lifecycle ordering per backend.",
        minecraft = "Defaults to [`sand::participant::EventParticipantPlan::none`] — a genuinely additive default; every existing `SandEvent` implementation is unaffected by this method's existence. The export pipeline applies a declared plan automatically for every dispatch backend that can meaningfully own one — tick-lifecycle/tick-poll dispatch, same-cycle chained dispatch (#264), and tracked-transition dispatch (#270) all merge it into the generated detector/handler around the same pre/post-observation boundary [`EventSetup::with_participants`] documents; you do not need to call `with_participants` yourself unless you are hand-assembling an `EventSetup` outside the normal `#[on_event]` path. See `sand-core/src/participant/plan.rs`'s module doc for the exact lifecycle ordering per backend.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: participant :: EventParticipantPlan` value produced to which participant observations this event declares (#230 Phase 10).",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEvent>()  {\n    let participants = <T as sand::events::SandEvent>::participants();\n}",
    )]
    fn participants() -> crate::participant::EventParticipantPlan {
        crate::participant::EventParticipantPlan::none()
    }

    /// Whether to revoke the advancement after it fires.
    ///
    /// Defaults to `true` — the advancement is revoked immediately so it can
    /// fire again the next time the trigger is satisfied.
    ///
    /// Set to `false` for one-shot events that should fire **only once per
    /// player, ever** (e.g. first-time rewards).
    ///
    /// Only relevant when [`dispatch`](Self::dispatch) returns
    /// [`SandEventDispatch::AdvancementTrigger`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEvent::revoke", kind = "trait_method",
        aliases = ["sand::prelude::SandEvent::revoke"],
        module = "sand::events",
        summary = "Whether to revoke the advancement after it fires.",
        context = "Whether to revoke the advancement after it fires. Defaults to `true` — the advancement is revoked immediately so it can fire again the next time the trigger is satisfied. Set to `false` for one-shot events that should fire only once per player, ever (e.g. first-time rewards). Only relevant when [`dispatch`](Self::dispatch) returns [`SandEventDispatch::AdvancementTrigger`].",
        minecraft = "True emits an advancement revoke for the triggering player; tick events have no advancement grant to revoke.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "Only relevant when [`dispatch`](Self::dispatch) returns [`SandEventDispatch::AdvancementTrigger`].",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEvent>()  {\n    let is_revoke = <T as sand::events::SandEvent>::revoke();\n}",
    )]
    fn revoke() -> bool {
        true
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::SandEventParticipants",
    aliases = ["sand::prelude::SandEventParticipants"],
    module = "sand::events",
    summary = "Infallible participant accessors for bare `SandEvent`-backed `#[on_event]` handlers (`fn handler(event: MarkerType)`), mirroring [`sand::event::Event`]'s `AdvancementEvent`-backed accessor sugar without a second overlapping blanket `impl<E: SandEvent> Event<E>` (#273) — coherence would reject that the moment a type could implement both `AdvancementEvent` and `SandEvent`, which every built-in combat event already does. This trait sidesteps the conflict entirely: it is implemented once, for every `T: SandEvent + 'static`, as inherent-feeling methods on the concrete marker type itself rather than through a second generic wrapper.",
    context = "Infallible participant accessors for bare `SandEvent`-backed `#[on_event]` handlers (`fn handler(event: MarkerType)`), mirroring [`sand::event::Event`]'s `AdvancementEvent`-backed accessor sugar without a second overlapping blanket `impl<E: SandEvent> Event<E>` (#273) — coherence would reject that the moment a type could implement both `AdvancementEvent` and `SandEvent`, which every built-in combat event already does. This trait sidesteps the conflict entirely: it is implemented once, for every `T: SandEvent + 'static`, as inherent-feeling methods on the concrete marker type itself rather than through a second generic wrapper. A blanket `impl` is provided for every `SandEvent` — no manual implementation needed: See [`sand::participant::ParticipantBuilder`] for how to declare the plan these accessors read from.",
    minecraft = "Each accessor reads evidence Sand captured for the current dispatch cycle.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::SandEventParticipants;",
)]
/// Infallible participant accessors for bare `SandEvent`-backed `#[on_event]`
/// handlers (`fn handler(event: MarkerType)`), mirroring
/// [`crate::event::Event`]'s `AdvancementEvent`-backed accessor sugar
/// without a second overlapping blanket `impl<E: SandEvent> Event<E>` (#273)
/// — coherence would reject that the moment a type could implement both
/// `AdvancementEvent` and `SandEvent`, which every built-in combat event
/// already does. This trait sidesteps the conflict entirely: it is
/// implemented once, for every `T: SandEvent + 'static`, as inherent-feeling
/// methods on the concrete marker type itself rather than through a second
/// generic wrapper.
///
/// A blanket `impl` is provided for every `SandEvent` — no manual
/// implementation needed:
///
/// ```rust,ignore
/// #[on_event]
/// fn special_kill(event: SpecialKillEvent) {
///     let killer = event.killer();
///     let weapon = event.weapon();
/// }
/// ```
///
/// See [`crate::participant::ParticipantBuilder`] for how to declare the
/// plan these accessors read from.
pub trait SandEventParticipants: SandEvent + Sized + 'static {
    /// Access a declared entity participant by role. See
    /// [`crate::event::Event::entity`] for the identical infallible
    /// contract this mirrors.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventParticipants::entity", kind = "trait_method",
        aliases = ["sand::prelude::SandEventParticipants::entity"],
        module = "sand::events",
        summary = "Access a declared entity participant by role. See [`sand::event::Event::entity`] for the identical infallible contract this mirrors.",
        context = "Access a declared entity participant by role. See [`sand::event::Event::entity`] for the identical infallible contract this mirrors. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The value is available only when the event plan captured that role.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(role = "`role` identifies the declared entity participant by role. See [`sand::event::Event::entity`] for the identical infallible contract this mirrors."),
        returns = "The `sand :: participant :: EntityParticipant` value produced to acces a declared entity participant by role. See [`sand::event::Event::entity`] for the identical infallible contract this mirrors.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEventParticipants>(sand_event_participants_value: &T, role: sand::participant::EntityParticipantRole)  {\n    let entity = sand_event_participants_value.entity(role);\n}",
    )]
    fn entity(
        &self,
        role: crate::participant::EntityParticipantRole,
    ) -> crate::participant::EntityParticipant {
        Self::participants().require_entity(std::any::type_name::<Self>(), role)
    }

    /// Access a declared item participant by role. See [`Self::entity`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventParticipants::item", kind = "trait_method",
        aliases = ["sand::prelude::SandEventParticipants::item"],
        module = "sand::events",
        summary = "Access a declared item participant by role. See [`Self::entity`].",
        context = "Access a declared item participant by role. See [`Self::entity`]. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand captures its NBT at dispatch time rather than depending on a live slot.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(role = "`role` identifies the declared item participant by role. See [`Self::entity`]."),
        returns = "The `sand :: item :: ItemSnapshot` value produced to acces a declared item participant by role. See [`Self::entity`].",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEventParticipants>(sand_event_participants_value: &T, role: sand::participant::ItemParticipantRole)  {\n    let item = sand_event_participants_value.item(role);\n}",
    )]
    fn item(&self, role: crate::participant::ItemParticipantRole) -> crate::item::ItemSnapshot {
        Self::participants().require_item(std::any::type_name::<Self>(), role)
    }

    /// The entity that caused this event, when declared.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventParticipants::attacker", kind = "trait_method",
        aliases = ["sand::prelude::SandEventParticipants::attacker"],
        module = "sand::events",
        summary = "The entity that caused this event, when declared.",
        context = "The entity that caused this event, when declared. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It is evidence-backed and unavailable unless the event plan captured an attacker.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: participant :: EntityParticipant` value produced to use the entity that caused this event, when declared.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEventParticipants>(sand_event_participants_value: &T)  {\n    let attacker = sand_event_participants_value.attacker();\n}",
    )]
    fn attacker(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::Attacker)
    }

    /// The entity that landed the killing blow, when declared.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventParticipants::killer", kind = "trait_method",
        aliases = ["sand::prelude::SandEventParticipants::killer"],
        module = "sand::events",
        summary = "The entity that landed the killing blow, when declared.",
        context = "The entity that landed the killing blow, when declared. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The value belongs to the current event dispatch record.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: participant :: EntityParticipant` value produced to use the entity that landed the killing blow, when declared.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEventParticipants>(sand_event_participants_value: &T)  {\n    let killer = sand_event_participants_value.killer();\n}",
    )]
    fn killer(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::Killer)
    }

    /// The entity that received damage/an effect, when declared.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventParticipants::victim", kind = "trait_method",
        aliases = ["sand::prelude::SandEventParticipants::victim"],
        module = "sand::events",
        summary = "The entity that received damage/an effect, when declared.",
        context = "The entity that received damage/an effect, when declared. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The value belongs to the current event dispatch record.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: participant :: EntityParticipant` value produced to use the entity that received damage/an effect, when declared.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEventParticipants>(sand_event_participants_value: &T)  {\n    let victim = sand_event_participants_value.victim();\n}",
    )]
    fn victim(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::Victim)
    }

    /// The entity this player directly interacted with, when declared.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventParticipants::interacted_entity", kind = "trait_method",
        aliases = ["sand::prelude::SandEventParticipants::interacted_entity"],
        module = "sand::events",
        summary = "The entity this player directly interacted with, when declared.",
        context = "The entity this player directly interacted with, when declared. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It reflects evidence captured by the trigger or participant plan.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: participant :: EntityParticipant` value produced to use the entity this player directly interacted with, when declared.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEventParticipants>(sand_event_participants_value: &T)  {\n    let interacted_entity = sand_event_participants_value.interacted_entity();\n}",
    )]
    fn interacted_entity(&self) -> crate::participant::EntityParticipant {
        self.entity(crate::participant::EntityParticipantRole::InteractedEntity)
    }

    /// The weapon item snapshot, when declared.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventParticipants::weapon", kind = "trait_method",
        aliases = ["sand::prelude::SandEventParticipants::weapon"],
        module = "sand::events",
        summary = "The weapon item snapshot, when declared.",
        context = "The weapon item snapshot, when declared. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand stores the trigger-time weapon NBT for the dispatch cycle.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        returns = "The `sand :: item :: ItemSnapshot` value produced to use the weapon item snapshot, when declared.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEventParticipants>(sand_event_participants_value: &T)  {\n    let weapon = sand_event_participants_value.weapon();\n}",
    )]
    fn weapon(&self) -> crate::item::ItemSnapshot {
        self.item(crate::participant::ItemParticipantRole::Weapon)
    }

    /// Access a declared bounded item participant by role (#272) — the
    /// `.within(...)`-crossing counterpart to [`Self::item`]. Backed by
    /// [`crate::participant::EventParticipantPlan::inherit_item_within`]
    /// instead of a same-cycle capture; see that method's doc for the full
    /// replacement/expiry/absence contract.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventParticipants::bounded_item", kind = "trait_method",
        aliases = ["sand::prelude::SandEventParticipants::bounded_item"],
        module = "sand::events",
        summary = "Access a declared bounded item participant by role (#272) — the `.within(...)`-crossing counterpart to [`Self::item`]. Backed by [`sand::participant::EventParticipantPlan::inherit_item_within`] instead of a same-cycle capture; see that method's doc for the full replacement/expiry/absence contract.",
        context = "Access a declared bounded item participant by role (#272) — the `.within(...)`-crossing counterpart to [`Self::item`]. Backed by [`sand::participant::EventParticipantPlan::inherit_item_within`] instead of a same-cycle capture; see that method's doc for the full replacement/expiry/absence contract. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "The snapshot expires according to the composed event's TickWindow.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(role = "`role` identifies the declared bounded item participant by role (#272) — the `.within(...)`-crossing counterpart to [`Self::item`]. Backed by [`sand::participant::EventParticipantPlan::inherit_item_within`] instead of a same-cycle capture; see that method's doc for the full replacement/expiry/absence contract."),
        returns = "The `sand :: participant :: BoundedItemSnapshot` value produced to acces a declared bounded item participant by role (#272) — the `.within(...)`-crossing counterpart to [`Self::item`]. Backed by [`sand::participant::EventParticipantPlan::inherit_item_within`] instead of a same-cycle capture; see that method's doc for the full replacement/expiry/absence contract.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::events::SandEventParticipants>(sand_event_participants_value: &T, role: sand::participant::ItemParticipantRole)  {\n    let bounded_item = sand_event_participants_value.bounded_item(role);\n}",
    )]
    fn bounded_item(
        &self,
        role: crate::participant::ItemParticipantRole,
    ) -> crate::participant::BoundedItemSnapshot {
        Self::participants().require_bounded_item(std::any::type_name::<Self>(), role)
    }
}

impl<T: SandEvent + Sized + 'static> SandEventParticipants for T {}

// ── Built-in event marker types ───────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::OnJoinEvent",
    aliases = ["sand::event::vanilla::OnJoin"],
    module = "sand::events",
    summary = "Fires on the first tick after a server start, `/reload`, or when a new player joins mid-session.",
    context = "Fires on the first tick after a server start, `/reload`, or when a new player joins mid-session. The supported author-facing identity is `sand::events::OnJoinEvent`. Implemented as a `JoinTick` scoreboard check: the `__sand_join` scoreboard objective is created and reset on `minecraft:load`; players whose score is not 1 trigger all handlers, after which their score is set to 1. Vanilla limitation: mid-session disconnect → reconnect without a `/reload` does not re-fire because the player's score persists in `scoreboard.dat`. True per-login detection requires a mod or plugin.",
    minecraft = "Implemented as a `JoinTick` scoreboard check: the `__sand_join` scoreboard objective is created and reset on `minecraft:load`; players whose score is not 1 trigger all handlers, after which their score is set to 1.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn joined(event: sand::event::Event<sand::events::OnJoinEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires on the first tick after a server start, `/reload`, or when a new
/// player joins mid-session.
///
/// The supported author-facing identity is `sand::events::OnJoinEvent`.
///
/// Implemented as a `JoinTick` scoreboard check: the `__sand_join` scoreboard
/// objective is created and reset on `minecraft:load`; players whose score is
/// not 1 trigger all handlers, after which their score is set to 1.
///
/// **Vanilla limitation:** mid-session disconnect → reconnect without a
/// `/reload` does **not** re-fire because the player's score persists in
/// `scoreboard.dat`. True per-login detection requires a mod or plugin.
///
/// # Example
///
/// ```rust,ignore
/// #[on_event]
/// pub fn on_join(event: Event<OnJoinEvent>) {
///     cmd::tellraw(
///         Selector::self_(),
///         Text::new("Welcome back!").gold(),
///     );
/// }
/// ```
pub struct OnJoinEvent;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::FirstJoinEvent",
    aliases = ["sand::event::vanilla::FirstJoin"],
    module = "sand::events",
    summary = "Fires the very first time a player ever joins. Never fires again.",
    context = "Fires the very first time a player ever joins. Never fires again. The supported author-facing identity is `sand::events::FirstJoinEvent`. Implemented as an `Advancement + Tick` trigger without revocation. Once the advancement is granted it stays, so the event fires exactly once per player across all sessions.",
    minecraft = "Uses Sand's persisted first-join state and does not re-arm for that player.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn first_join(event: sand::event::Event<sand::events::FirstJoinEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires the very first time a player ever joins. Never fires again.
///
/// The supported author-facing identity is `sand::events::FirstJoinEvent`.
///
/// Implemented as an `Advancement + Tick` trigger **without** revocation.
/// Once the advancement is granted it stays, so the event fires exactly once
/// per player across all sessions.
///
/// # Example
///
/// ```rust,ignore
/// #[on_event]
/// pub fn first_join(event: Event<FirstJoinEvent>) {
///     cmd::tellraw(
///         Selector::self_(),
///         Text::new("Welcome for the very first time!").aqua(),
///     );
///     cmd::give(Selector::self_(), "minecraft:diamond").count(3);
/// }
/// ```
pub struct FirstJoinEvent;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::OnDeathEvent",
    aliases = ["sand::event::vanilla::OnDeath"],
    module = "sand::events",
    summary = "Fires on the tick a player dies (any cause: mob, fall, void, `/kill`, …).",
    context = "Fires on the tick a player dies (any cause: mob, fall, void, `/kill`, …). The supported author-facing identity is `sand::events::OnDeathEvent`. Implemented via the `deathCount` scoreboard criterion. The handler runs as `@s` = the dying player.",
    minecraft = "Implemented via the `deathCount` scoreboard criterion. The handler runs as `@s` = the dying player.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn died(event: sand::event::Event<sand::events::OnDeathEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires on the tick a player dies (any cause: mob, fall, void, `/kill`, …).
///
/// The supported author-facing identity is `sand::events::OnDeathEvent`.
///
/// Implemented via the `deathCount` scoreboard criterion. The handler runs as
/// `@s` = the dying player.
///
/// # Example
///
/// ```rust,ignore
/// static TOTAL_DEATHS: ScoreVar<i32> = ScoreVar::new("total_deaths");
///
/// #[on_event]
/// pub fn on_death(event: Event<OnDeathEvent>) {
///     TOTAL_DEATHS.add(event.player(), 1);
/// }
/// ```
pub struct OnDeathEvent;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::OnRespawnEvent",
    aliases = ["sand::event::vanilla::OnRespawn"],
    module = "sand::events",
    summary = "Fires on the first Sand tick that observes the player active after death.",
    context = "Fires on the first Sand tick that observes the player active after death. The supported author-facing identity is `sand::events::OnRespawnEvent`. Sand records a per-player waiting phase when `deathCount` observes a death. Vanilla resets `minecraft.custom:minecraft.time_since_death` to zero on death and increments it only while the player is alive; the event dispatches once that score is positive, then returns the phase to idle. The completion check runs before new death observation in one generated coordinator, so [`OnDeathEvent`] and this event cannot dispatch from the same death observation. This is a tick-boundary signal rather than the exact client respawn packet. Remaining on the death screen (including disconnecting while dead) leaves the statistic at zero and the lifecycle waiting. Immediate respawn is supported, but still dispatches no earlier than the next Sand observation cycle. Hardcore's post-death spectator transition counts as active again if vanilla begins incrementing the statistic. Ordinary dimension changes do not enter the waiting phase and therefore do not dispatch this event. A respawn and another death that both complete between Sand ticks can coalesce beca...",
    minecraft = "Sand records a per-player waiting phase when `deathCount` observes a death. Vanilla resets `minecraft.custom:minecraft.time_since_death` to zero on death and increments it only while the player is alive; the event dispatches once that score is positive, then returns the phase to idle. The completion check runs before new death observation in one generated coordinator, so [`OnDeathEvent`] and this event cannot dispatch from the same death observation.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn respawned(event: sand::event::Event<sand::events::OnRespawnEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires on the first Sand tick that observes the player active after death.
///
/// The supported author-facing identity is `sand::events::OnRespawnEvent`.
///
/// Sand records a per-player waiting phase when `deathCount` observes a death.
/// Vanilla resets `minecraft.custom:minecraft.time_since_death` to zero on
/// death and increments it only while the player is alive; the event dispatches
/// once that score is positive, then returns the phase to idle. The completion
/// check runs before new death observation in one generated coordinator, so
/// [`OnDeathEvent`] and this event cannot dispatch from the same death
/// observation.
///
/// This is a tick-boundary signal rather than the exact client respawn packet.
/// Remaining on the death screen (including disconnecting while dead) leaves
/// the statistic at zero and the lifecycle waiting. Immediate respawn is
/// supported, but still dispatches no earlier than the next Sand observation
/// cycle. Hardcore's post-death spectator transition counts as active again if
/// vanilla begins incrementing the statistic. Ordinary dimension changes do
/// not enter the waiting phase and therefore do not dispatch this event. A
/// respawn and another death that both complete between Sand ticks can coalesce
/// because vanilla exposes no intermediate datapack callback.
///
/// # Example
///
/// ```rust,ignore
/// #[on_event]
/// pub fn on_respawn(event: Event<OnRespawnEvent>) {
///     cmd::tellraw(
///         Selector::self_(),
///         Text::new("You respawned!").green(),
///     );
/// }
/// ```
pub struct OnRespawnEvent;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ArmorEquipEvent",
    module = "sand::events",
    summary = "Fires on the tick a player equips an item in an equipment slot.",
    context = "Fires on the tick a player equips an item in an equipment slot. Uses tick-based state tracking via entity tags — no advancement required. Sand maintains a `__armor_<slot>` tag per player to detect transitions. - `slot = Head | Chest | Legs | Feet | Offhand` - `item = \"namespace:item_id\"` — only trigger for this item - `custom_data = \"{key:1b}\"` — match `minecraft:custom_data` component (SNBT)",
    minecraft = "- `item = \"namespace:item_id\"` — only trigger for this item - `custom_data = \"{key:1b}\"` — match `minecraft:custom_data` component (SNBT)",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event(slot = Feet)]\nfn equip(event: sand::event::Event<sand::events::ArmorEquipEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires on the tick a player **equips** an item in an equipment slot.
///
/// Uses tick-based state tracking via entity tags — no advancement required.
/// Sand maintains a `__armor_<slot>` tag per player to detect transitions.
///
/// # Required filter
///
/// - `slot = Head | Chest | Legs | Feet | Offhand`
///
/// # Optional filters
///
/// - `item = "namespace:item_id"` — only trigger for this item
/// - `custom_data = "{key:1b}"` — match `minecraft:custom_data` component (SNBT)
///
/// # Example
///
/// ```rust,ignore
/// static MANA_REGEN: Flag = Flag::new("mana_regen");
///
/// // Any item equipped in the feet slot
/// #[on_event(slot = Feet)]
/// pub fn any_boots_equipped(event: Event<ArmorEquipEvent>) {
///     cmd::say("Boots equipped!");
/// }
///
/// // Specific item with custom NBT
/// #[on_event(slot = Feet, item = "minecraft:leather_boots", custom_data = "{mana_boots:1b}")]
/// pub fn mana_boots_equipped(event: Event<ArmorEquipEvent>) {
///     MANA_REGEN.enable(event.player());
/// }
/// ```
pub struct ArmorEquipEvent;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ArmorUnequipEvent",
    module = "sand::events",
    summary = "Fires on the tick a player removes an item from an equipment slot.",
    context = "Fires on the tick a player removes an item from an equipment slot. Same filter syntax as [`ArmorEquipEvent`].",
    minecraft = "Sand tracks equipment-slot state each tick; #[on_event] requires a slot filter and may further filter item/custom data.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event(slot = Feet)]\nfn unequip(event: sand::event::Event<sand::events::ArmorUnequipEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires on the tick a player **removes** an item from an equipment slot.
///
/// Same filter syntax as [`ArmorEquipEvent`].
///
/// # Example
///
/// ```rust,ignore
/// static MANA_REGEN: Flag = Flag::new("mana_regen");
///
/// #[on_event(slot = Feet, item = "minecraft:leather_boots", custom_data = "{mana_boots:1b}")]
/// pub fn mana_boots_removed(event: Event<ArmorUnequipEvent>) {
///     MANA_REGEN.disable(event.player());
/// }
/// ```
pub struct ArmorUnequipEvent;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::HoldingItemEvent",
    module = "sand::events",
    summary = "Fires every tick a player is holding a specific item.",
    context = "Fires every tick a player is holding a specific item. Uses `execute if items entity @s <slot> <item>` per tick. - `item = \"namespace:item_id\"` - `slot = Mainhand | Offhand` (defaults to `Mainhand`) - `custom_data = \"{key:1b}\"` — match `minecraft:custom_data` component",
    minecraft = "- `slot = Mainhand | Offhand` (defaults to `Mainhand`) - `custom_data = \"{key:1b}\"` — match `minecraft:custom_data` component",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event(item = \"minecraft:shield\", slot = Offhand)]\nfn shield(event: sand::event::Event<sand::events::HoldingItemEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires every tick a player is **holding** a specific item.
///
/// Uses `execute if items entity @s <slot> <item>` per tick.
///
/// # Required filter
///
/// - `item = "namespace:item_id"`
///
/// # Optional filters
///
/// - `slot = Mainhand | Offhand` (defaults to `Mainhand`)
/// - `custom_data = "{key:1b}"` — match `minecraft:custom_data` component
///
/// # Example
///
/// ```rust,ignore
/// static BLOCKING: Flag = Flag::new("blocking");
///
/// #[on_event(item = "minecraft:diamond_sword")]
/// pub fn holding_diamond_sword(event: Event<HoldingItemEvent>) {
///     cmd::particle(Particle::Crit, event.player());
/// }
///
/// #[on_event(item = "minecraft:shield", slot = Offhand)]
/// pub fn holding_shield_offhand(event: Event<HoldingItemEvent>) {
///     BLOCKING.enable(event.player());
/// }
/// ```
pub struct HoldingItemEvent;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::CurrentlyWearingEvent",
    module = "sand::events",
    summary = "Fires every tick a player is wearing a specific item in an armor slot.",
    context = "Fires every tick a player is wearing a specific item in an armor slot. Uses `execute if items entity @s armor.<slot> <item>` per tick. - `slot = Head | Chest | Legs | Feet` - `item = \"namespace:item_id\"` - `custom_data = \"{key:1b}\"` — match `minecraft:custom_data` component",
    minecraft = "- `custom_data = \"{key:1b}\"` — match `minecraft:custom_data` component",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event(slot = Head, item = \"minecraft:diamond_helmet\")]\nfn helmet(event: sand::event::Event<sand::events::CurrentlyWearingEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires every tick a player is **wearing** a specific item in an armor slot.
///
/// Uses `execute if items entity @s armor.<slot> <item>` per tick.
///
/// # Required filters
///
/// - `slot = Head | Chest | Legs | Feet`
/// - `item = "namespace:item_id"`
///
/// # Optional filters
///
/// - `custom_data = "{key:1b}"` — match `minecraft:custom_data` component
///
/// # Example
///
/// ```rust,ignore
/// #[on_event(slot = Head, item = "minecraft:diamond_helmet")]
/// pub fn wearing_diamond_helmet(event: Event<CurrentlyWearingEvent>) {
///     cmd::particle(Particle::Enchant, event.player());
/// }
/// ```
pub struct CurrentlyWearingEvent;

// ════════════════════════════════════════════════════════════════════════════
// ── Comprehensive built-in event library ────────────────────────────────────
// ════════════════════════════════════════════════════════════════════════════
//
// All events below implement [`SandEvent`] and can be used directly with
// `#[on_event]`. Most map 1:1 to a Minecraft advancement trigger so they fire
// once per trigger occurrence and revoke themselves (unless noted).
// For filter-level customisation (e.g. specific item/entity), implement your
// own type with [`SandEvent`] using the same trigger and supply conditions.

// ── Kill / combat ─────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::EntityKillEvent",
    aliases = ["sand::event::vanilla::EntityKill"],
    module = "sand::events",
    summary = "Fires when the player kills any entity. Maps to `minecraft:player_killed_entity` with no conditions. For entity-type filters, use a custom [`SandEvent`] with the [`sand::component::AdvancementTrigger::PlayerKilledEntity`] trigger.",
    context = "Fires when the player kills any entity. Maps to `minecraft:player_killed_entity` with no conditions. For entity-type filters, use a custom [`SandEvent`] with the [`sand::component::AdvancementTrigger::PlayerKilledEntity`] trigger. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:player_killed_entity` with no conditions. For entity-type filters, use a custom [`SandEvent`] with the [`sand::component::AdvancementTrigger::PlayerKilledEntity`] trigger.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn kill(event: sand::event::Event<sand::events::EntityKillEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player kills any entity.
///
/// Maps to `minecraft:player_killed_entity` with no conditions.
/// For entity-type filters, use a custom [`SandEvent`] with the
/// [`crate::AdvancementTrigger::PlayerKilledEntity`] trigger.
///
/// # Example
/// ```rust,ignore
/// static TOTAL_KILLS: ScoreVar<i32> = ScoreVar::new("total_kills");
///
/// #[on_event]
/// pub fn on_kill(event: Event<EntityKillEvent>) {
///     TOTAL_KILLS.add(event.player(), 1);
/// }
/// ```
pub struct EntityKillEvent;
impl SandEvent for EntityKillEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::PlayerKilledEntity {
            entity: None,
            killing_blow: None,
        })
    }
    fn participants() -> crate::participant::EventParticipantPlan {
        crate::participant::EventParticipantPlan::new().observe_weapon()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerKillEvent",
    aliases = ["sand::event::vanilla::PlayerKill"],
    module = "sand::events",
    summary = "Fires when any entity kills the player. Maps to `minecraft:entity_killed_player` with no conditions.",
    context = "Fires when any entity kills the player. Maps to `minecraft:entity_killed_player` with no conditions. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:entity_killed_player` with no conditions.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn killed(event: sand::event::Event<sand::events::PlayerKillEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when any entity kills the player.
///
/// Maps to `minecraft:entity_killed_player` with no conditions.
///
/// # Example
/// ```rust,ignore
/// #[on_event]
/// pub fn on_killed(event: Event<PlayerKillEvent>) {
///     cmd::tellraw(
///         event.player(),
///         Text::new("You were slain!").red(),
///     );
/// }
/// ```
pub struct PlayerKillEvent;
impl SandEvent for PlayerKillEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::EntityKilledPlayer {
            entity: None,
            killing_blow: None,
        })
    }
    fn participants() -> crate::participant::EventParticipantPlan {
        crate::participant::EventParticipantPlan::new().observe_correlated_killer()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerDamageEntityEvent",
    aliases = ["sand::event::vanilla::PlayerDamagesEntity"],
    module = "sand::events",
    summary = "Fires when the player deals damage to any entity.",
    context = "Fires when the player deals damage to any entity. Maps to `minecraft:player_hurt_entity`.",
    minecraft = "Maps to `minecraft:player_hurt_entity`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn hit(event: sand::event::DamageEvent<sand::events::PlayerDamageEntityEvent>) { let _damage = event.reflect_damage(); sand::command::raw(\"say @s hit an entity\") }",
)]
/// Fires when the player deals damage to any entity.
///
/// Maps to `minecraft:player_hurt_entity`.
pub struct PlayerDamageEntityEvent;
impl SandEvent for PlayerDamageEntityEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::PlayerHurtEntity {
            entity: None,
            damage: None,
        })
    }
    fn participants() -> crate::participant::EventParticipantPlan {
        crate::participant::EventParticipantPlan::new().observe_weapon()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::EntityDamagePlayerEvent",
    aliases = ["sand::event::vanilla::EntityDamagesPlayer"],
    module = "sand::events",
    summary = "Fires when any entity deals damage to the player.",
    context = "Fires when any entity deals damage to the player. Maps to `minecraft:entity_hurt_player`.",
    minecraft = "Maps to `minecraft:entity_hurt_player`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn hurt(event: sand::event::DamageEvent<sand::events::EntityDamagePlayerEvent>) { let _damage = event.reflect_damage(); sand::command::raw(\"say @s was hurt\") }",
)]
/// Fires when any entity deals damage to the player.
///
/// Maps to `minecraft:entity_hurt_player`.
pub struct EntityDamagePlayerEvent;
impl SandEvent for EntityDamagePlayerEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::EntityHurtPlayer {
            entity: None,
            damage: None,
        })
    }
    fn participants() -> crate::participant::EventParticipantPlan {
        crate::participant::EventParticipantPlan::new().observe_correlated_attacker()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ShotCrossbowEvent",
    aliases = ["sand::event::vanilla::CrossbowShot"],
    module = "sand::events",
    summary = "Fires when the player shoots a crossbow.",
    context = "Fires when the player shoots a crossbow. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Uses minecraft:shot_crossbow and re-arms its generated advancement after dispatch.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn shot(event: sand::event::Event<sand::events::ShotCrossbowEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player shoots a crossbow.
pub struct ShotCrossbowEvent;
impl SandEvent for ShotCrossbowEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ShotCrossbow {
            item: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ChanneledLightningEvent",
    module = "sand::events",
    summary = "Fires when the player channels a trident's lightning at an entity.",
    context = "Fires when the player channels a trident's lightning at an entity. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Uses minecraft:channeled_lightning advancement criteria.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn lightning(event: sand::event::Event<sand::events::ChanneledLightningEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player channels a trident's lightning at an entity.
pub struct ChanneledLightningEvent;
impl SandEvent for ChanneledLightningEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ChanneledLightning {
            victims: None,
        })
    }
}

// ── Items ─────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ItemConsumeEvent",
    aliases = ["sand::event::vanilla::AnyItemConsumed"],
    module = "sand::events",
    summary = "Fires when the player consumes any item (food, potion, etc.).",
    context = "Fires when the player consumes any item (food, potion, etc.). Maps to `minecraft:consume_item`.",
    minecraft = "Maps to `minecraft:consume_item`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn consumed(event: sand::event::Event<sand::events::ItemConsumeEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player consumes any item (food, potion, etc.).
///
/// Maps to `minecraft:consume_item`.
///
/// # Example
/// ```rust,ignore
/// #[on_event]
/// pub fn on_eat(event: Event<ItemConsumeEvent>) {
///     cmd::say("Yum!");
/// }
/// ```
pub struct ItemConsumeEvent;
impl SandEvent for ItemConsumeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ConsumeItem { item: None })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ItemCraftEvent",
    aliases = ["sand::event::vanilla::AnyItemCrafted"],
    module = "sand::events",
    summary = "Compatibility marker for the removed `minecraft:crafted_item` trigger. Target-aware export rejects it with a migration diagnostic because current vanilla's `minecraft:recipe_crafted` requires a concrete recipe ID.",
    context = "Compatibility marker for the removed `minecraft:crafted_item` trigger. Target-aware export rejects it with a migration diagnostic because current vanilla's `minecraft:recipe_crafted` requires a concrete recipe ID. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Uses minecraft:recipe_crafted with Sand's broad built-in criterion.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn crafted(event: sand::event::Event<sand::events::ItemCraftEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Compatibility marker for the removed `minecraft:crafted_item` trigger.
/// Target-aware export rejects it with a migration diagnostic because current
/// vanilla's `minecraft:recipe_crafted` requires a concrete recipe ID.
pub struct ItemCraftEvent;
impl SandEvent for ItemCraftEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::CraftedItem { item: None })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ItemEnchantEvent",
    aliases = ["sand::event::vanilla::AnyItemEnchanted"],
    module = "sand::events",
    summary = "Fires when the player enchants any item. Maps to `minecraft:enchanted_item`.",
    context = "Fires when the player enchants any item. Maps to `minecraft:enchanted_item`. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:enchanted_item`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn enchanted(event: sand::event::Event<sand::events::ItemEnchantEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player enchants any item.
///
/// Maps to `minecraft:enchanted_item`.
pub struct ItemEnchantEvent;
impl SandEvent for ItemEnchantEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::EnchantedItem {
            item: None,
            levels: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::BucketFillEvent",
    module = "sand::events",
    summary = "Fires when the player fills any bucket. Maps to `minecraft:filled_bucket`.",
    context = "Fires when the player fills any bucket. Maps to `minecraft:filled_bucket`. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:filled_bucket`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn filled(event: sand::event::Event<sand::events::BucketFillEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player fills any bucket.
///
/// Maps to `minecraft:filled_bucket`.
pub struct BucketFillEvent;
impl SandEvent for BucketFillEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::FilledBucket {
            item: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::BucketEmptyEvent",
    module = "sand::events",
    summary = "Legacy compatibility marker for bucket-empty detection.",
    context = "Legacy compatibility marker for bucket-empty detection. The historical `minecraft:emptied_bucket` ID is absent from Sand's verified vanilla registries. Export fails with a migration diagnostic rather than silently emitting an event that never loads. There is no exact current advancement-trigger replacement; use an explicitly documented polling/correlation strategy when approximate detection is acceptable.",
    minecraft = "The historical `minecraft:emptied_bucket` ID is absent from Sand's verified vanilla registries. Export fails with a migration diagnostic rather than silently emitting an event that never loads. There is no exact current advancement-trigger replacement; use an explicitly documented polling/correlation strategy when approximate detection is acceptable.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn emptied(event: sand::event::Event<sand::events::BucketEmptyEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Legacy compatibility marker for bucket-empty detection.
///
/// The historical `minecraft:emptied_bucket` ID is absent from Sand's
/// verified vanilla registries. Export fails with a migration diagnostic
/// rather than silently emitting an event that never loads. There is no exact
/// current advancement-trigger replacement; use an explicitly documented
/// polling/correlation strategy when approximate detection is acceptable.
pub struct BucketEmptyEvent;
impl SandEvent for BucketEmptyEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::EmptiedBucket {
            item: None,
            location: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::FishingEvent",
    module = "sand::events",
    summary = "Fires when the player uses a fishing rod and it hooks something.",
    context = "Fires when the player uses a fishing rod and it hooks something. Maps to `minecraft:fishing_rod_hooked`.",
    minecraft = "Maps to `minecraft:fishing_rod_hooked`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn fish(event: sand::event::Event<sand::events::FishingEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player uses a fishing rod and it hooks something.
///
/// Maps to `minecraft:fishing_rod_hooked`.
pub struct FishingEvent;
impl SandEvent for FishingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::FishingRodHooked {
            rod: None,
            entity: None,
            item: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ItemPickedUpEvent",
    module = "sand::events",
    summary = "Fires when the player picks up a thrown item. Maps to `minecraft:thrown_item_picked_up_by_player`. Use the typed trigger variant ending in `ByEntity` for non-player pickup criteria.",
    context = "Fires when the player picks up a thrown item. Maps to `minecraft:thrown_item_picked_up_by_player`. Use the typed trigger variant ending in `ByEntity` for non-player pickup criteria. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:thrown_item_picked_up_by_player`. Use the typed trigger variant ending in `ByEntity` for non-player pickup criteria.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn pickup(event: sand::event::Event<sand::events::ItemPickedUpEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player picks up a thrown item.
///
/// Maps to `minecraft:thrown_item_picked_up_by_player`. Use the typed trigger
/// variant ending in `ByEntity` for non-player pickup criteria.
pub struct ItemPickedUpEvent;
impl SandEvent for ItemPickedUpEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(
            crate::AdvancementTrigger::ThrownItemPickedUpByPlayer {
                item: None,
                entity: None,
            },
        )
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ItemDurabilityChangeEvent",
    module = "sand::events",
    summary = "Fires when an item in the player's inventory loses durability.",
    context = "Fires when an item in the player's inventory loses durability. Maps to `minecraft:item_durability_changed`.",
    minecraft = "Maps to `minecraft:item_durability_changed`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn durability(event: sand::event::Event<sand::events::ItemDurabilityChangeEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when an item in the player's inventory loses durability.
///
/// Maps to `minecraft:item_durability_changed`.
pub struct ItemDurabilityChangeEvent;
impl SandEvent for ItemDurabilityChangeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ItemDurabilityChanged {
            item: None,
            delta: None,
            durability: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::BrewPotionEvent",
    aliases = ["sand::event::vanilla::PotionBrewed"],
    module = "sand::events",
    summary = "Fires when the player brews a potion. Maps to `minecraft:brewed_potion`.",
    context = "Fires when the player brews a potion. Maps to `minecraft:brewed_potion`. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:brewed_potion`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn brewed(event: sand::event::Event<sand::events::BrewPotionEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player brews a potion.
///
/// Maps to `minecraft:brewed_potion`.
pub struct BrewPotionEvent;
impl SandEvent for BrewPotionEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::brewed_any_potion())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::TotemActivateEvent",
    module = "sand::events",
    summary = "Fires when the player activates a totem of undying.",
    context = "Fires when the player activates a totem of undying. Maps to `minecraft:used_totem`.",
    minecraft = "Maps to `minecraft:used_totem`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn saved(event: sand::event::Event<sand::events::TotemActivateEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player activates a totem of undying.
///
/// Maps to `minecraft:used_totem`.
pub struct TotemActivateEvent;
impl SandEvent for TotemActivateEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::UsedTotem { item: None })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::RecipeUnlockEvent",
    module = "sand::events",
    summary = "Fires when the player unlocks a recipe. Maps to `minecraft:recipe_unlocked` with no recipe filter.",
    context = "Fires when the player unlocks a recipe. Maps to `minecraft:recipe_unlocked` with no recipe filter. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:recipe_unlocked` with no recipe filter.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn recipe(event: sand::event::Event<sand::events::RecipeUnlockEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player unlocks a recipe.
///
/// Maps to `minecraft:recipe_unlocked` with no recipe filter.
pub struct RecipeUnlockEvent;
impl SandEvent for RecipeUnlockEvent {
    fn dispatch() -> SandEventDispatch {
        // Use Custom because RecipeUnlocked requires a specific recipe string;
        // the no-filter version just fires for any recipe unlock.
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::Custom {
            trigger: "minecraft:recipe_unlocked".into(),
            conditions: None,
        })
    }
}

// ── World / blocks ────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::BlockPlaceEvent",
    aliases = ["sand::event::vanilla::AnyBlockPlaced"],
    module = "sand::events",
    summary = "Fires when the player places any block. Maps to `minecraft:placed_block` with no filters.",
    context = "Fires when the player places any block. Maps to `minecraft:placed_block` with no filters. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:placed_block` with no filters.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn placed(event: sand::event::Event<sand::events::BlockPlaceEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player places any block.
///
/// Maps to `minecraft:placed_block` with no filters.
///
/// # Example
/// ```rust,ignore
/// static BLOCKS_PLACED: ScoreVar<i32> = ScoreVar::new("blocks_placed");
///
/// #[on_event]
/// pub fn on_place(event: Event<BlockPlaceEvent>) {
///     BLOCKS_PLACED.add(event.player(), 1);
/// }
/// ```
pub struct BlockPlaceEvent;
impl SandEvent for BlockPlaceEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::placed_block(
            None, None, None, None,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::EnterBlockEvent",
    module = "sand::events",
    summary = "Fires when the player enters a block (e.g. water, honey).",
    context = "Fires when the player enters a block (e.g. water, honey). Maps to `minecraft:enter_block` with no block filter.",
    minecraft = "Maps to `minecraft:enter_block` with no block filter.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn entered(event: sand::event::Event<sand::events::EnterBlockEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player enters a block (e.g. water, honey).
///
/// Maps to `minecraft:enter_block` with no block filter.
pub struct EnterBlockEvent;
impl SandEvent for EnterBlockEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::enter_block(None, None))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::SlideDownBlockEvent",
    module = "sand::events",
    summary = "Fires when the player slides down a block (e.g. honey block wall).",
    context = "Fires when the player slides down a block (e.g. honey block wall). Maps to `minecraft:slide_down_block`.",
    minecraft = "Maps to `minecraft:slide_down_block`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn slid(event: sand::event::Event<sand::events::SlideDownBlockEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player slides down a block (e.g. honey block wall).
///
/// Maps to `minecraft:slide_down_block`.
pub struct SlideDownBlockEvent;
impl SandEvent for SlideDownBlockEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::slide_down_block(None))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::TargetHitEvent",
    module = "sand::events",
    summary = "Fires when a target block is hit by a projectile near the player.",
    context = "Fires when a target block is hit by a projectile near the player. Maps to `minecraft:target_hit`.",
    minecraft = "Maps to `minecraft:target_hit`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn target(event: sand::event::Event<sand::events::TargetHitEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when a target block is hit by a projectile near the player.
///
/// Maps to `minecraft:target_hit`.
pub struct TargetHitEvent;
impl SandEvent for TargetHitEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::TargetHit {
            signal_strength: None,
            projectile: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::BeeNestDestroyedEvent",
    module = "sand::events",
    summary = "Fires when the player destroys a bee nest or beehive.",
    context = "Fires when the player destroys a bee nest or beehive. Maps to `minecraft:bee_nest_destroyed`.",
    minecraft = "Maps to `minecraft:bee_nest_destroyed`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn bees(event: sand::event::Event<sand::events::BeeNestDestroyedEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player destroys a bee nest or beehive.
///
/// Maps to `minecraft:bee_nest_destroyed`.
pub struct BeeNestDestroyedEvent;
impl SandEvent for BeeNestDestroyedEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::bee_nest_destroyed(
            None, None, None,
        ))
    }
}

// ── Player state ──────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ChangeDimensionEvent",
    aliases = ["sand::event::vanilla::DimensionChanged"],
    module = "sand::events",
    summary = "Fires when the player changes dimension (e.g. entering the Nether or End).",
    context = "Fires when the player changes dimension (e.g. entering the Nether or End). Maps to `minecraft:changed_dimension`.",
    minecraft = "Maps to `minecraft:changed_dimension`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn dimension(event: sand::event::Event<sand::events::ChangeDimensionEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player changes dimension (e.g. entering the Nether or End).
///
/// Maps to `minecraft:changed_dimension`.
///
/// # Example
/// ```rust,ignore
/// #[on_event]
/// pub fn on_change_dim(event: Event<ChangeDimensionEvent>) {
///     cmd::say("Dimension change!");
/// }
/// ```
pub struct ChangeDimensionEvent;
impl SandEvent for ChangeDimensionEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::changed_dimension(
            None, None,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerSleepEvent",
    module = "sand::events",
    summary = "Fires when the player sleeps in a bed. Maps to `minecraft:slept_in_bed`.",
    context = "Fires when the player sleeps in a bed. Maps to `minecraft:slept_in_bed`. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:slept_in_bed`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn slept(event: sand::event::Event<sand::events::PlayerSleepEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player sleeps in a bed.
///
/// Maps to `minecraft:slept_in_bed`.
pub struct PlayerSleepEvent;
impl SandEvent for PlayerSleepEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::SleptInBed {
            location: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::FallFromHeightEvent",
    module = "sand::events",
    summary = "Fires when the player falls from a height and lands.",
    context = "Fires when the player falls from a height and lands. Maps to `minecraft:fall_from_height`.",
    minecraft = "Maps to `minecraft:fall_from_height`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn fell(event: sand::event::Event<sand::events::FallFromHeightEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player falls from a height and lands.
///
/// Maps to `minecraft:fall_from_height`.
///
/// # Example
/// ```rust,ignore
/// #[on_event]
/// pub fn on_fall(event: Event<FallFromHeightEvent>) {
///     cmd::playsound(
///         ResourceLocation::new("minecraft", "entity.player.hurt").unwrap(),
///         event.player(),
///     );
/// }
/// ```
pub struct FallFromHeightEvent;
impl SandEvent for FallFromHeightEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::FallFromHeight {
            distance: None,
            start_position: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerLevelUpEvent",
    aliases = ["sand::event::vanilla::PlayerLevelsUp"],
    module = "sand::events",
    summary = "Fires when a player's XP level increases (gains one or more levels in a tick).",
    context = "Fires when a player's XP level increases (gains one or more levels in a tick). The supported author-facing identity is `sand::events::PlayerLevelUpEvent`. Implemented as a Sand-generated tick-backed system — not an advancement. Vanilla Minecraft has no `minecraft:leveled_up` advancement trigger. Sand generates four scoreboard objectives: - `__sand_xp_lvl`   — current XP level - `__sand_xp_prev`  — previous tick's XP level - `__sand_xp_delta` — current − previous - `__sand_xp_seen`  — join-safety flag (prevents false fire on first tick)",
    minecraft = "Implemented as a Sand-generated tick-backed system — not an advancement. Vanilla Minecraft has no `minecraft:leveled_up` advancement trigger.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn leveled(event: sand::event::Event<sand::events::PlayerLevelUpEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when a player's XP level increases (gains one or more levels in a tick).
///
/// The supported author-facing identity is `sand::events::PlayerLevelUpEvent`.
///
/// Implemented as a Sand-generated tick-backed system — not an advancement.
/// Vanilla Minecraft has no `minecraft:leveled_up` advancement trigger.
///
/// Sand generates four scoreboard objectives:
/// - `__sand_xp_lvl`   — current XP level
/// - `__sand_xp_prev`  — previous tick's XP level
/// - `__sand_xp_delta` — current − previous
/// - `__sand_xp_seen`  — join-safety flag (prevents false fire on first tick)
///
/// The handler fires once per player per tick where their level increased. Level
/// decreases and same-level ticks do not fire. The first tick after a player
/// joins only initialises the baseline and does not fire.
///
/// # Example
///
/// ```rust,ignore
/// use sand_core::events::PlayerLevelUpEvent;
/// use sand_core::prelude::*;
/// use sand_macros::on_event;
///
/// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
///
/// #[on_event]
/// pub fn on_level_up(event: Event<PlayerLevelUpEvent>) {
///     MANA.add(event.player(), 10);
/// }
/// ```
pub struct PlayerLevelUpEvent;

/// Sand-internal score objectives used by the XP level-up tick system.
///
/// These are named exactly so the component generator and `Event<PlayerLevelUpEvent>`
/// helpers agree on the same objective names. All names are ≤16 characters.
pub(crate) static SAND_XP_LVL: crate::state::score::ScoreVar<i32> =
    crate::state::score::ScoreVar::new("__sand_xp_lvl");
pub(crate) static SAND_XP_PREV: crate::state::score::ScoreVar<i32> =
    crate::state::score::ScoreVar::new("__sand_xp_prev");
pub(crate) static SAND_XP_DELTA: crate::state::score::ScoreVar<i32> =
    crate::state::score::ScoreVar::new("__sand_xp_delta");

impl PlayerLevelUpEvent {
    /// Returns a [`ScoreRef`] for the player's current XP level this tick.
    ///
    /// The objective `__sand_xp_lvl` is populated each tick by
    /// `experience query @s levels`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[on_event]
    /// pub fn on_level_up(event: Event<PlayerLevelUpEvent>) {
    ///     let lvl = PlayerLevelUpEvent::current_level("@s");
    /// }
    /// ```
    ///
    /// [`ScoreRef`]: crate::state::score::ScoreRef
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::PlayerLevelUpEvent::current_level",
        aliases = ["sand::event::vanilla::PlayerLevelsUp::current_level"],
        module = "sand::events",
        kind = "method",
        summary = "Returns a [`ScoreRef`] for the player's current XP level this tick.",
        context = "Returns a [`ScoreRef`] for the player's current XP level this tick. The objective `__sand_xp_lvl` is populated each tick by `experience query @s levels`. [`ScoreRef`]: sand::state::ScoreRef",
        minecraft = "Sand synchronizes the current experience level during its generated tick observation.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(selector = "`selector` provides the Minecraft target selection used to return a [`ScoreRef`] for the player's current XP level this tick."),
        returns = "Returns a [`ScoreRef`] for the player's current XP level this tick.",
        example = "pub fn on_level_up(event: Event<PlayerLevelUpEvent>) {\nlet lvl = PlayerLevelUpEvent::current_level(\"@s\");\n}",
    )]
    pub fn current_level(selector: &str) -> crate::state::score::ScoreRef<'static, i32> {
        SAND_XP_LVL.of(selector)
    }

    /// Returns a [`ScoreRef`] for the player's XP level on the previous tick.
    ///
    /// The objective `__sand_xp_prev` holds the level from the preceding tick.
    ///
    /// [`ScoreRef`]: crate::state::score::ScoreRef
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::PlayerLevelUpEvent::previous_level",
        aliases = ["sand::event::vanilla::PlayerLevelsUp::previous_level"],
        module = "sand::events",
        kind = "method",
        summary = "Returns a [`ScoreRef`] for the player's XP level on the previous tick.",
        context = "Returns a [`ScoreRef`] for the player's XP level on the previous tick. The objective `__sand_xp_prev` holds the level from the preceding tick. [`ScoreRef`]: sand::state::ScoreRef",
        minecraft = "Sand snapshots the player's experience level before its generated transition check.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(selector = "`selector` provides the Minecraft target selection used to return a [`ScoreRef`] for the player's XP level on the previous tick."),
        returns = "Returns a [`ScoreRef`] for the player's XP level on the previous tick.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: & str)  {\n    let previous_level = sand::events::PlayerLevelUpEvent::previous_level(selector);\n}",
    )]
    pub fn previous_level(selector: &str) -> crate::state::score::ScoreRef<'static, i32> {
        SAND_XP_PREV.of(selector)
    }

    /// Returns a [`ScoreRef`] for the level delta this tick (current − previous).
    ///
    /// The objective `__sand_xp_delta` is always ≥ 1 when a handler fires,
    /// since the handler only runs when the delta is positive.
    ///
    /// [`ScoreRef`]: crate::state::score::ScoreRef
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::PlayerLevelUpEvent::level_delta",
        aliases = ["sand::event::vanilla::PlayerLevelsUp::level_delta"],
        module = "sand::events",
        kind = "method",
        summary = "Returns a [`ScoreRef`] for the level delta this tick (current − previous).",
        context = "Returns a [`ScoreRef`] for the level delta this tick (current − previous). The objective `__sand_xp_delta` is always ≥ 1 when a handler fires, since the handler only runs when the delta is positive. [`ScoreRef`]: sand::state::ScoreRef",
        minecraft = "Sand derives it from the synchronized previous and current level scores.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        params(selector = "`selector` provides the Minecraft target selection used to return a [`ScoreRef`] for the level delta this tick (current − previous)."),
        returns = "Returns a [`ScoreRef`] for the level delta this tick (current − previous).",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: & str)  {\n    let level_delta = sand::events::PlayerLevelUpEvent::level_delta(selector);\n}",
    )]
    pub fn level_delta(selector: &str) -> crate::state::score::ScoreRef<'static, i32> {
        SAND_XP_DELTA.of(selector)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::EffectsChangedEvent",
    module = "sand::events",
    summary = "Fires when the player's status effects change. Maps to `minecraft:effects_changed`.",
    context = "Fires when the player's status effects change. Maps to `minecraft:effects_changed`. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:effects_changed`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn effects(event: sand::event::Event<sand::events::EffectsChangedEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player's status effects change.
///
/// Maps to `minecraft:effects_changed`.
pub struct EffectsChangedEvent;
impl SandEvent for EffectsChangedEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::effects_changed_any(None))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::StartRidingEvent",
    module = "sand::events",
    summary = "Fires when the player starts riding an entity (horse, boat, etc.).",
    context = "Fires when the player starts riding an entity (horse, boat, etc.). Maps to `minecraft:started_riding`.",
    minecraft = "Maps to `minecraft:started_riding`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn ride(event: sand::event::Event<sand::events::StartRidingEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player starts riding an entity (horse, boat, etc.).
///
/// Maps to `minecraft:started_riding`.
pub struct StartRidingEvent;
impl SandEvent for StartRidingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::StartedRiding)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::UseEnderEyeEvent",
    module = "sand::events",
    summary = "Fires when the player uses an ender eye (to locate a stronghold).",
    context = "Fires when the player uses an ender eye (to locate a stronghold). Maps to `minecraft:used_ender_eye`.",
    minecraft = "Maps to `minecraft:used_ender_eye`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn eye(event: sand::event::Event<sand::events::UseEnderEyeEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player uses an ender eye (to locate a stronghold).
///
/// Maps to `minecraft:used_ender_eye`.
pub struct UseEnderEyeEvent;
impl SandEvent for UseEnderEyeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::UsedEnderEye {
            distance: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::TameAnimalEvent",
    aliases = ["sand::event::vanilla::AnimalTamed"],
    module = "sand::events",
    summary = "Fires when the player tames an animal. Maps to `minecraft:tame_animal`.",
    context = "Fires when the player tames an animal. Maps to `minecraft:tame_animal`. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:tame_animal`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn tame(event: sand::event::Event<sand::events::TameAnimalEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player tames an animal.
///
/// Maps to `minecraft:tame_animal`.
pub struct TameAnimalEvent;
impl SandEvent for TameAnimalEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::TamedAnimal {
            entity: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::BreedAnimalsEvent",
    aliases = ["sand::event::vanilla::AnimalsBreed"],
    module = "sand::events",
    summary = "Fires when the player breeds two animals. Maps to `minecraft:bred_animals`.",
    context = "Fires when the player breeds two animals. Maps to `minecraft:bred_animals`. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:bred_animals`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn breed(event: sand::event::Event<sand::events::BreedAnimalsEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player breeds two animals.
///
/// Maps to `minecraft:bred_animals`.
pub struct BreedAnimalsEvent;
impl SandEvent for BreedAnimalsEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::BredAnimals {
            parent: None,
            partner: None,
            child: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::SummonEntityEvent",
    aliases = ["sand::event::vanilla::EntitySummoned"],
    module = "sand::events",
    summary = "Fires when the player summons an entity (e.g. Iron Golem, Snow Golem, Wither).",
    context = "Fires when the player summons an entity (e.g. Iron Golem, Snow Golem, Wither). Maps to `minecraft:summoned_entity`.",
    minecraft = "Maps to `minecraft:summoned_entity`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn summon(event: sand::event::Event<sand::events::SummonEntityEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player summons an entity (e.g. Iron Golem, Snow Golem, Wither).
///
/// Maps to `minecraft:summoned_entity`.
pub struct SummonEntityEvent;
impl SandEvent for SummonEntityEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::SummonedEntity {
            entity: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::InteractWithEntityEvent",
    module = "sand::events",
    summary = "Fires when the player interacts with any entity (right-click).",
    context = "Fires when the player interacts with any entity (right-click). Maps to `minecraft:player_interacted_with_entity`.",
    minecraft = "Maps to `minecraft:player_interacted_with_entity`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn interact(event: sand::event::Event<sand::events::InteractWithEntityEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player interacts with any entity (right-click).
///
/// Maps to `minecraft:player_interacted_with_entity`.
pub struct InteractWithEntityEvent;
impl SandEvent for InteractWithEntityEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(
            crate::AdvancementTrigger::PlayerInteractedWithEntity {
                item: None,
                entity: None,
            },
        )
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::VillagerTradeEvent",
    module = "sand::events",
    summary = "Fires when the player trades with a villager. Maps to `minecraft:villager_trade`.",
    context = "Fires when the player trades with a villager. Maps to `minecraft:villager_trade`. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:villager_trade`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn trade(event: sand::event::Event<sand::events::VillagerTradeEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player trades with a villager.
///
/// Maps to `minecraft:villager_trade`.
pub struct VillagerTradeEvent;
impl SandEvent for VillagerTradeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::VillagerTrade {
            item: None,
            villager: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ConstructBeaconEvent",
    module = "sand::events",
    summary = "Fires when the player constructs or upgrades a beacon.",
    context = "Fires when the player constructs or upgrades a beacon. Maps to `minecraft:construct_beacon`.",
    minecraft = "Maps to `minecraft:construct_beacon`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn beacon(event: sand::event::Event<sand::events::ConstructBeaconEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player constructs or upgrades a beacon.
///
/// Maps to `minecraft:construct_beacon`.
pub struct ConstructBeaconEvent;
impl SandEvent for ConstructBeaconEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ConstructBeacon {
            level: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::CureZombieVillagerEvent",
    module = "sand::events",
    summary = "Fires when the player cures a zombie villager. Maps to `minecraft:cured_zombie_villager`.",
    context = "Fires when the player cures a zombie villager. Maps to `minecraft:cured_zombie_villager`. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Maps to `minecraft:cured_zombie_villager`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn cure(event: sand::event::Event<sand::events::CureZombieVillagerEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player cures a zombie villager.
///
/// Maps to `minecraft:cured_zombie_villager`.
pub struct CureZombieVillagerEvent;
impl SandEvent for CureZombieVillagerEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::CuredZombieVillager {
            villager: None,
            zombie: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::LootContainerOpenEvent",
    module = "sand::events",
    summary = "Fires when the player opens a container that generates loot.",
    context = "Fires when the player opens a container that generates loot. Maps to `minecraft:player_generates_container_loot`.",
    minecraft = "Maps to `minecraft:player_generates_container_loot`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn loot(event: sand::event::Event<sand::events::LootContainerOpenEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player opens a container that generates loot.
///
/// Maps to `minecraft:player_generates_container_loot`.
pub struct LootContainerOpenEvent;
impl SandEvent for LootContainerOpenEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(
            crate::AdvancementTrigger::PlayerGeneratesContainerLoot { loot_table: None },
        )
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::HeroOfTheVillageEvent",
    module = "sand::events",
    summary = "Fires when the player achieves Hero of the Village.",
    context = "Fires when the player achieves Hero of the Village. Maps to `minecraft:hero_of_the_village`. Fires once per raid victory.",
    minecraft = "Maps to `minecraft:hero_of_the_village`. Fires once per raid victory.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn hero(event: sand::event::Event<sand::events::HeroOfTheVillageEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when the player achieves Hero of the Village.
///
/// Maps to `minecraft:hero_of_the_village`. Fires once per raid victory.
pub struct HeroOfTheVillageEvent;
impl SandEvent for HeroOfTheVillageEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::HeroOfTheVillage {
            location: None,
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::LightningStrikeEvent",
    module = "sand::events",
    summary = "Fires when a lightning bolt strikes near the player.",
    context = "Fires when a lightning bolt strikes near the player. Maps to `minecraft:lightning_strike`.",
    minecraft = "Maps to `minecraft:lightning_strike`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn strike(event: sand::event::Event<sand::events::LightningStrikeEvent>) { sand::command::raw(\"say @s event fired\"); }",
)]
/// Fires when a lightning bolt strikes near the player.
///
/// Maps to `minecraft:lightning_strike`.
pub struct LightningStrikeEvent;
impl SandEvent for LightningStrikeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::LightningStrike {
            lightning: None,
            bystander: None,
        })
    }
}

// ════════════════════════════════════════════════════════════════════════════
// ── AdvancementEvent impls for all advancement-backed events ──────────────
// ════════════════════════════════════════════════════════════════════════════
//
// These allow using the built-in event types with `Event<E>` and the typed
// trigger builders from `sand_core::event::trigger`.

macro_rules! adv_event {
    ($ty:ty) => {
        impl crate::event::AdvancementEvent for $ty {
            type Trigger = crate::AdvancementTrigger;
            fn trigger() -> Self::Trigger {
                let dispatch: SandEventDispatch = <$ty as SandEvent>::dispatch().into();
                dispatch.into_trigger().unwrap()
            }
        }
    };
    // Same as above, plus a declared participant plan (#230) — the export
    // pipeline applies it automatically to this event's generated body; see
    // `crate::event::AdvancementEvent::participants`.
    ($ty:ty, participants: $plan:expr) => {
        impl crate::event::AdvancementEvent for $ty {
            type Trigger = crate::AdvancementTrigger;
            fn trigger() -> Self::Trigger {
                let dispatch: SandEventDispatch = <$ty as SandEvent>::dispatch().into();
                dispatch.into_trigger().unwrap()
            }
            fn participants() -> crate::participant::EventParticipantPlan {
                $plan
            }
        }
    };
}

impl SandEventDispatch {
    /// Extract the advancement trigger from this dispatch, panicking if it's
    /// a tick-condition dispatch.
    fn into_trigger(self) -> Option<crate::AdvancementTrigger> {
        match self.normalize() {
            NormalizedEventDispatch::Advancement(t) => Some(t),
            NormalizedEventDispatch::Tick(_)
            | NormalizedEventDispatch::Chain(_)
            | NormalizedEventDispatch::Tracked(_) => None,
        }
    }
}

// Combat participant plans (#230): `@s` is the player subject for every one
// of these advancement-backed events. For EntityKillEvent/PlayerDamageEntityEvent,
// the player *is* the causing entity (already exact via `.player()`), so the
// only useful declared participant is the weapon in their mainhand at the
// moment of the trigger (`ItemLocation::PlayerMainHand`, exact snapshot —
// see `EventParticipantPlan::observe_weapon`). For PlayerKillEvent/
// EntityDamagePlayerEvent, the player is the *victim*, so the causing
// entity is only reachable through `execute on attacker` — the existing
// Phase 9 correlated-attacker backend — under the Killer/Attacker role
// respectively. Victim/DirectAttacker/InteractedEntity/Projectile/
// ProjectileOwner/Ammunition are intentionally not declared here: no
// evidence-backed backend exists for them on these event families (see
// `docs/testing/participant-role-evidence.md`) — `event.entity(role)`/
// `event.item(role)` honestly resolve `Unavailable(NotApplicable)` for any
// undeclared role rather than guessing.
adv_event!(
    EntityKillEvent,
    participants: crate::participant::EventParticipantPlan::new().observe_weapon()
);
adv_event!(
    PlayerKillEvent,
    participants: crate::participant::EventParticipantPlan::new().observe_correlated_killer()
);
adv_event!(
    PlayerDamageEntityEvent,
    participants: crate::participant::EventParticipantPlan::new().observe_weapon()
);
adv_event!(
    EntityDamagePlayerEvent,
    participants: crate::participant::EventParticipantPlan::new().observe_correlated_attacker()
);
impl crate::event::DamageAdvancementEvent for PlayerDamageEntityEvent {}
impl crate::event::DamageAdvancementEvent for EntityDamagePlayerEvent {}
adv_event!(ShotCrossbowEvent);
adv_event!(ChanneledLightningEvent);
adv_event!(ItemConsumeEvent);
adv_event!(ItemCraftEvent);
adv_event!(ItemEnchantEvent);
adv_event!(BucketFillEvent);
adv_event!(BucketEmptyEvent);
adv_event!(FishingEvent);
adv_event!(ItemPickedUpEvent);
adv_event!(ItemDurabilityChangeEvent);
adv_event!(BrewPotionEvent);
adv_event!(TotemActivateEvent);
adv_event!(RecipeUnlockEvent);
adv_event!(BlockPlaceEvent);
adv_event!(EnterBlockEvent);
adv_event!(SlideDownBlockEvent);
adv_event!(TargetHitEvent);
adv_event!(BeeNestDestroyedEvent);
adv_event!(ChangeDimensionEvent);
adv_event!(PlayerSleepEvent);
adv_event!(FallFromHeightEvent);
// PlayerLevelUpEvent uses XpLevelUp dispatch — not an advancement trigger.
// The AdvancementEvent impl here is a placeholder so Event<PlayerLevelUpEvent>
// satisfies the Event<E: AdvancementEvent> bound. The macro special-cases
// PlayerLevelUpEvent / PlayerLevelsUp and emits EventDispatch::XpLevelUp instead
// of calling this trigger.
impl crate::event::AdvancementEvent for PlayerLevelUpEvent {
    type Trigger = crate::AdvancementTrigger;
    fn trigger() -> Self::Trigger {
        // This trigger is never emitted — the macro bypasses AdvancementEvent::trigger()
        // for PlayerLevelUpEvent and emits EventDispatch::XpLevelUp instead.
        crate::AdvancementTrigger::Tick
    }
}
adv_event!(EffectsChangedEvent);
adv_event!(StartRidingEvent);
adv_event!(UseEnderEyeEvent);
adv_event!(TameAnimalEvent);
adv_event!(BreedAnimalsEvent);
adv_event!(SummonEntityEvent);
adv_event!(InteractWithEntityEvent);
adv_event!(VillagerTradeEvent);
adv_event!(ConstructBeaconEvent);
adv_event!(CureZombieVillagerEvent);
adv_event!(LootContainerOpenEvent);
adv_event!(HeroOfTheVillageEvent);
adv_event!(LightningStrikeEvent);

// ── Tick-poll events ──────────────────────────────────────────────────────
//
// These fire every tick the condition is true, checked as each online player.
// They use `TickCondition` dispatch — no advancement file is generated.
//
// These use Sand-owned entity predicates. Predicate flags are stable datapack
// schema, unlike raw player NBT selector fields.

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerStartSneakingEvent",
    aliases = ["sand::event::vanilla::PlayerStartsSneaking"],
    module = "sand::events",
    summary = "Fires once when a player changes from not sneaking to sneaking.",
    context = "Fires once when a player changes from not sneaking to sneaking. This is tick-polled from vanilla's `flags.is_sneaking` entity predicate. The first observed state establishes a baseline and does not fire.",
    minecraft = "Sand compares a tracked player-sneaking predicate across ticks and fires only on false-to-true transitions.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn start_sneak(_: sand::events::PlayerStartSneakingEvent) { sand::command::raw(\"say Sneak\"); }",
)]
/// Fires once when a player changes from not sneaking to sneaking.
///
/// This is tick-polled from vanilla's `flags.is_sneaking` entity predicate.
/// The first observed state establishes a baseline and does not fire.
pub struct PlayerStartSneakingEvent;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerStopSneakingEvent",
    aliases = ["sand::event::vanilla::PlayerStopsSneaking"],
    module = "sand::events",
    summary = "Fires once when a player changes from sneaking to not sneaking.",
    context = "Fires once when a player changes from sneaking to not sneaking. Uses the same shared tracker as [`PlayerStartSneakingEvent`].",
    minecraft = "Sand's tracked sneaking predicate fires only on true-to-false transitions.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn stop_sneak(_: sand::events::PlayerStopSneakingEvent) { sand::command::raw(\"say Stop\"); }",
)]
/// Fires once when a player changes from sneaking to not sneaking.
///
/// Uses the same shared tracker as [`PlayerStartSneakingEvent`].
pub struct PlayerStopSneakingEvent;

/// Shared current-state source used by both sneaking transitions and
/// persistent composition. Kept public only for proc-macro expansion.
#[doc(hidden)]
pub(crate) const PLAYER_SNEAKING_TRACKED_SOURCE: crate::TrackedSource =
    crate::TrackedSource::BooleanCondition {
        description: "vanilla entity predicate flags.is_sneaking",
        condition: "predicate __sand_local:__sand/player_sneaking",
    };

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerSneakEvent",
    aliases = ["sand::event::vanilla::PlayerSneaking"],
    module = "sand::events",
    summary = "Fires every tick the player is sneaking / crouching (Shift held).",
    context = "Fires every tick the player is sneaking / crouching (Shift held). Uses a generated `flags.is_sneaking` predicate.",
    minecraft = "Sand evaluates its player-sneaking predicate for each online player every tick.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn sneaking(_: sand::events::PlayerSneakEvent) { sand::command::raw(\"say Sneaking\"); }",
)]
/// Fires every tick the player is sneaking / crouching (Shift held).
///
/// Uses a generated `flags.is_sneaking` predicate.
///
/// # Example
/// ```rust,ignore
/// use sand_core::events::PlayerSneakEvent;
/// use sand_core::prelude::*;
/// use sand_macros::on_event;
///
/// #[on_event]
/// pub fn while_sneaking(event: PlayerSneakEvent) {
///     cmd::particle(Particle::Smoke, event.player());
/// }
/// ```
pub struct PlayerSneakEvent;
impl SandEvent for PlayerSneakEvent {
    fn dispatch() -> SandEventDispatch {
        let crate::TrackedSource::BooleanCondition { condition, .. } =
            PLAYER_SNEAKING_TRACKED_SOURCE
        else {
            unreachable!("the shared sneaking source is boolean")
        };
        SandEventDispatch::TickCondition(condition.into())
    }
}
impl PersistentSandEvent for PlayerSneakEvent {
    fn persistent_condition() -> PersistentEventCondition {
        let crate::TrackedSource::BooleanCondition { condition, .. } =
            PLAYER_SNEAKING_TRACKED_SOURCE
        else {
            unreachable!("the shared sneaking source is boolean")
        };
        let predicate = condition
            .strip_prefix("predicate ")
            .expect("the shared sneaking source is a predicate condition");
        PersistentEventCondition::players(crate::condition::Condition::predicate_raw(predicate))
    }
}

/// Shared current-state source for sprinting transitions and persistent
/// composition. Kept public only for proc-macro expansion.
#[doc(hidden)]
pub(crate) const PLAYER_SPRINTING_TRACKED_SOURCE: crate::TrackedSource =
    crate::TrackedSource::BooleanCondition {
        description: "vanilla entity predicate flags.is_sprinting",
        condition: "predicate __sand_local:__sand/player_sprinting",
    };

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerSprintEvent",
    aliases = ["sand::event::vanilla::PlayerSprinting"],
    module = "sand::events",
    summary = "Fires every tick the player is sprinting. Uses a generated `flags.is_sprinting` predicate.",
    context = "Fires every tick the player is sprinting. Uses a generated `flags.is_sprinting` predicate. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Sand evaluates its tracked sprinting predicate for each online player every tick.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn sprinting(_: sand::events::PlayerSprintEvent) { sand::command::raw(\"say Sprint\"); }",
)]
/// Fires every tick the player is sprinting.
///
/// Uses a generated `flags.is_sprinting` predicate.
pub struct PlayerSprintEvent;
impl SandEvent for PlayerSprintEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("predicate __sand_local:__sand/player_sprinting".into())
    }
}
impl PersistentSandEvent for PlayerSprintEvent {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(crate::condition::Condition::predicate_raw(
            "__sand_local:__sand/player_sprinting",
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerStartSprintingEvent",
    module = "sand::events",
    summary = "Fires once when a player changes from not sprinting to sprinting.",
    context = "Fires once when a player changes from not sprinting to sprinting. Shares the `player_sprinting` tracker with [`PlayerStopSprintingEvent`] — multiple handlers of either event reuse one generated provider. The first observed state establishes a baseline and does not fire.",
    minecraft = "Sand emits a false-to-true transition from its per-player sprinting tracker.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn start_sprint(_: sand::events::PlayerStartSprintingEvent) { sand::command::raw(\"say Sprint\"); }",
)]
/// Fires once when a player changes from not sprinting to sprinting.
///
/// Shares the `player_sprinting` tracker with [`PlayerStopSprintingEvent`] —
/// multiple handlers of either event reuse one generated provider. The first
/// observed state establishes a baseline and does not fire.
pub struct PlayerStartSprintingEvent;
impl SandEvent for PlayerStartSprintingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_sprinting",
            PLAYER_SPRINTING_TRACKED_SOURCE,
            crate::TransitionKind::BecameTrue,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerStopSprintingEvent",
    module = "sand::events",
    summary = "Fires once when a player changes from sprinting to not sprinting.",
    context = "Fires once when a player changes from sprinting to not sprinting. Uses the same shared tracker as [`PlayerStartSprintingEvent`].",
    minecraft = "Sand emits a true-to-false transition from its per-player sprinting tracker.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn stop_sprint(_: sand::events::PlayerStopSprintingEvent) { sand::command::raw(\"say Stop\"); }",
)]
/// Fires once when a player changes from sprinting to not sprinting.
///
/// Uses the same shared tracker as [`PlayerStartSprintingEvent`].
pub struct PlayerStopSprintingEvent;
impl SandEvent for PlayerStopSprintingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_sprinting",
            PLAYER_SPRINTING_TRACKED_SOURCE,
            crate::TransitionKind::BecameFalse,
        ))
    }
}

/// Shared current-state source for swimming transitions and persistent
/// composition. Kept public only for proc-macro expansion.
#[doc(hidden)]
pub(crate) const PLAYER_SWIMMING_TRACKED_SOURCE: crate::TrackedSource =
    crate::TrackedSource::BooleanCondition {
        description: "vanilla entity predicate flags.is_swimming",
        condition: "predicate __sand_local:__sand/player_swimming",
    };

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerSwimmingEvent",
    aliases = ["sand::event::vanilla::PlayerSwimming"],
    module = "sand::events",
    summary = "Fires every tick the player is swimming (swimming animation active, 1.13+).",
    context = "Fires every tick the player is swimming (swimming animation active, 1.13+). Uses a generated `flags.is_swimming` predicate.",
    minecraft = "Sand evaluates the swimming entity predicate as each online player.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn swimming(_: sand::events::PlayerSwimmingEvent) { sand::command::raw(\"say Swim\"); }",
)]
/// Fires every tick the player is swimming (swimming animation active, 1.13+).
///
/// Uses a generated `flags.is_swimming` predicate.
pub struct PlayerSwimmingEvent;
impl SandEvent for PlayerSwimmingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("predicate __sand_local:__sand/player_swimming".into())
    }
}
impl PersistentSandEvent for PlayerSwimmingEvent {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(crate::condition::Condition::predicate_raw(
            "__sand_local:__sand/player_swimming",
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerStartSwimmingEvent",
    module = "sand::events",
    summary = "Fires once when a player changes from not swimming to swimming.",
    context = "Fires once when a player changes from not swimming to swimming. Shares the `player_swimming` tracker with [`PlayerStopSwimmingEvent`]. The first observed state establishes a baseline and does not fire.",
    minecraft = "Sand's tracked swimming predicate fires only on false-to-true transitions.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn start_swim(_: sand::events::PlayerStartSwimmingEvent) { sand::command::raw(\"say Swim\"); }",
)]
/// Fires once when a player changes from not swimming to swimming.
///
/// Shares the `player_swimming` tracker with [`PlayerStopSwimmingEvent`].
/// The first observed state establishes a baseline and does not fire.
pub struct PlayerStartSwimmingEvent;
impl SandEvent for PlayerStartSwimmingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_swimming",
            PLAYER_SWIMMING_TRACKED_SOURCE,
            crate::TransitionKind::BecameTrue,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerStopSwimmingEvent",
    module = "sand::events",
    summary = "Fires once when a player changes from swimming to not swimming.",
    context = "Fires once when a player changes from swimming to not swimming. Uses the same shared tracker as [`PlayerStartSwimmingEvent`].",
    minecraft = "Sand's tracked swimming predicate fires only on true-to-false transitions.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn stop_swim(_: sand::events::PlayerStopSwimmingEvent) { sand::command::raw(\"say Stop\"); }",
)]
/// Fires once when a player changes from swimming to not swimming.
///
/// Uses the same shared tracker as [`PlayerStartSwimmingEvent`].
pub struct PlayerStopSwimmingEvent;
impl SandEvent for PlayerStopSwimmingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_swimming",
            PLAYER_SWIMMING_TRACKED_SOURCE,
            crate::TransitionKind::BecameFalse,
        ))
    }
}

/// Shared current-state source for flying transitions and persistent
/// composition. Kept public only for proc-macro expansion.
#[doc(hidden)]
pub(crate) const PLAYER_FLYING_TRACKED_SOURCE: crate::TrackedSource =
    crate::TrackedSource::BooleanCondition {
        description: "vanilla entity NBT abilities.flying",
        condition: "entity @s[nbt={abilities:{flying:1b}}]",
    };

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerFlyingEvent",
    module = "sand::events",
    summary = "Fires every tick the player is actively flying (Creative/Spectator flight).",
    context = "Fires every tick the player is actively flying (Creative/Spectator flight). Uses `entity @s[nbt={abilities:{flying:1b}}]`.",
    minecraft = "Uses `entity @s[nbt={abilities:{flying:1b}}]`.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn flying(_: sand::events::PlayerFlyingEvent) { sand::command::raw(\"say Fly\"); }",
)]
/// Fires every tick the player is actively flying (Creative/Spectator flight).
///
/// Uses `entity @s[nbt={abilities:{flying:1b}}]`.
pub struct PlayerFlyingEvent;
impl SandEvent for PlayerFlyingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[nbt={abilities:{flying:1b}}]".into())
    }
}
impl PersistentSandEvent for PlayerFlyingEvent {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(crate::condition::Condition::entity_raw(
            "@s[nbt={abilities:{flying:1b}}]",
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerStartFlyingEvent",
    module = "sand::events",
    summary = "Fires once when a player starts actively flying (Creative/Spectator flight).",
    context = "Fires once when a player starts actively flying (Creative/Spectator flight). Shares the `player_flying` tracker with [`PlayerStopFlyingEvent`]. The first observed state establishes a baseline and does not fire — a player who is already flying at join/reload does not spuriously fire this event.",
    minecraft = "Sand tracks the player flying predicate and fires on false-to-true transitions.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn start_flying(_: sand::events::PlayerStartFlyingEvent) { sand::command::raw(\"say Fly\"); }",
)]
/// Fires once when a player starts actively flying (Creative/Spectator flight).
///
/// Shares the `player_flying` tracker with [`PlayerStopFlyingEvent`]. The
/// first observed state establishes a baseline and does not fire — a player
/// who is already flying at join/reload does not spuriously fire this event.
pub struct PlayerStartFlyingEvent;
impl SandEvent for PlayerStartFlyingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_flying",
            PLAYER_FLYING_TRACKED_SOURCE,
            crate::TransitionKind::BecameTrue,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerStopFlyingEvent",
    module = "sand::events",
    summary = "Fires once when a player stops actively flying. Uses the same shared tracker as [`PlayerStartFlyingEvent`].",
    context = "Fires once when a player stops actively flying. Uses the same shared tracker as [`PlayerStartFlyingEvent`]. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Sand tracks the player flying predicate and fires on true-to-false transitions.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn stop_flying(_: sand::events::PlayerStopFlyingEvent) { sand::command::raw(\"say Stop\"); }",
)]
/// Fires once when a player stops actively flying.
///
/// Uses the same shared tracker as [`PlayerStartFlyingEvent`].
pub struct PlayerStopFlyingEvent;
impl SandEvent for PlayerStopFlyingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_flying",
            PLAYER_FLYING_TRACKED_SOURCE,
            crate::TransitionKind::BecameFalse,
        ))
    }
}

/// Shared current-state source for on-fire transitions and persistent
/// composition. Kept public only for proc-macro expansion.
#[doc(hidden)]
pub(crate) const PLAYER_ON_FIRE_TRACKED_SOURCE: crate::TrackedSource =
    crate::TrackedSource::BooleanCondition {
        description: "vanilla entity predicate flags.is_on_fire",
        condition: "predicate __sand_local:__sand/player_on_fire",
    };

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerOnFireEvent",
    aliases = ["sand::event::vanilla::PlayerOnFire"],
    module = "sand::events",
    summary = "Fires every tick the player is on fire. Uses a generated `flags.is_on_fire` predicate.",
    context = "Fires every tick the player is on fire. Uses a generated `flags.is_on_fire` predicate. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Sand evaluates its player-on-fire predicate for each online player.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn burning(_: sand::events::PlayerOnFireEvent) { sand::command::raw(\"say Fire\"); }",
)]
/// Fires every tick the player is on fire.
///
/// Uses a generated `flags.is_on_fire` predicate.
pub struct PlayerOnFireEvent;
impl SandEvent for PlayerOnFireEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("predicate __sand_local:__sand/player_on_fire".into())
    }
}
impl PersistentSandEvent for PlayerOnFireEvent {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(crate::condition::Condition::predicate_raw(
            "__sand_local:__sand/player_on_fire",
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerCaughtFireEvent",
    module = "sand::events",
    summary = "Fires once when a player catches fire. Shares the `player_on_fire` tracker with [`PlayerExtinguishedEvent`]. The first observed state establishes a baseline and does not fire.",
    context = "Fires once when a player catches fire. Shares the `player_on_fire` tracker with [`PlayerExtinguishedEvent`]. The first observed state establishes a baseline and does not fire. Freezing and drowning start/stop events are intentionally not provided: unlike on-fire (a stable `flags.is_on_fire` entity predicate flag), vanilla Java exposes freezing only through the raw `ticks_frozen`/`ticks_frozen_max` NBT ratio and drowning only through the raw `Air` NBT stat, neither of which has a corresponding boolean entity predicate flag or scoreboard criterion as of Minecraft Java 26.2. Reading them would require a tick-sampled `data get entity @s ...` derivation with an author-chosen threshold, which is an inferred approximation, not an exact transition — exposing it as `PlayerStartedFreezingEvent` would overstate its reliability. See the testing documentation for the evidence trail; this is tracked as explicit follow-up scope, not a gap left silently unaddressed.",
    minecraft = "Freezing and drowning start/stop events are intentionally not provided: unlike on-fire (a stable `flags.is_on_fire` entity predicate flag), vanilla Java exposes freezing only through the raw `ticks_frozen`/`ticks_frozen_max` NBT ratio and drowning only through the raw `Air` NBT stat, neither of which has a corresponding boolean entity predicate flag or scoreboard criterion as of Minecraft Java 26.2. Reading them would require a tick-sampled `data get entity @s ...` derivation with an author-chosen threshold, which is an inferred approximation, not an exact transition — exposing it as `PlayerStartedFreezingEvent` would overstate its reliability. See the testing documentation for the evidence trail; this is tracked as explicit follow-up scope, not a gap left silently unaddressed.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn caught_fire(_: sand::events::PlayerCaughtFireEvent) { sand::command::raw(\"say Fire\"); }",
)]
/// Fires once when a player catches fire.
///
/// Shares the `player_on_fire` tracker with [`PlayerExtinguishedEvent`]. The
/// first observed state establishes a baseline and does not fire.
///
/// Freezing and drowning start/stop events are intentionally **not**
/// provided: unlike on-fire (a stable `flags.is_on_fire` entity predicate
/// flag), vanilla Java exposes freezing only through the raw
/// `ticks_frozen`/`ticks_frozen_max` NBT ratio and drowning only through the
/// raw `Air` NBT stat, neither of which has a corresponding boolean entity
/// predicate flag or scoreboard criterion as of Minecraft Java 26.2. Reading
/// them would require a tick-sampled `data get entity @s ...` derivation
/// with an author-chosen threshold, which is an inferred approximation, not
/// an exact transition — exposing it as `PlayerStartedFreezingEvent` would
/// overstate its reliability. See the testing documentation for the
/// evidence trail; this is tracked as explicit follow-up scope, not a gap
/// left silently unaddressed.
pub struct PlayerCaughtFireEvent;
impl SandEvent for PlayerCaughtFireEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_on_fire",
            PLAYER_ON_FIRE_TRACKED_SOURCE,
            crate::TransitionKind::BecameTrue,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerExtinguishedEvent",
    module = "sand::events",
    summary = "Fires once when a player stops being on fire. Uses the same shared tracker as [`PlayerCaughtFireEvent`].",
    context = "Fires once when a player stops being on fire. Uses the same shared tracker as [`PlayerCaughtFireEvent`]. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Sand's tracked fire predicate fires only on true-to-false transitions.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn extinguished(_: sand::events::PlayerExtinguishedEvent) { sand::command::raw(\"say Safe\"); }",
)]
/// Fires once when a player stops being on fire.
///
/// Uses the same shared tracker as [`PlayerCaughtFireEvent`].
pub struct PlayerExtinguishedEvent;
impl SandEvent for PlayerExtinguishedEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_on_fire",
            PLAYER_ON_FIRE_TRACKED_SOURCE,
            crate::TransitionKind::BecameFalse,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerInCreativeEvent",
    module = "sand::events",
    summary = "Fires every tick the player is in a Creative-mode gamemode.",
    context = "Fires every tick the player is in a Creative-mode gamemode. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Sand evaluates an entity gamemode=creative condition per online player.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn creative(_: sand::events::PlayerInCreativeEvent) { sand::command::raw(\"say Creative\"); }",
)]
/// Fires every tick the player is in a Creative-mode gamemode.
pub struct PlayerInCreativeEvent;
impl SandEvent for PlayerInCreativeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=creative]".into())
    }
}
impl PersistentSandEvent for PlayerInCreativeEvent {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(crate::condition::Condition::entity_raw(
            "@s[gamemode=creative]",
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerInAdventureEvent",
    module = "sand::events",
    summary = "Fires every tick the player is in Adventure mode.",
    context = "Fires every tick the player is in Adventure mode. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Sand evaluates an entity gamemode=adventure condition per online player.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn adventure(_: sand::events::PlayerInAdventureEvent) { sand::command::raw(\"say Adventure\"); }",
)]
/// Fires every tick the player is in Adventure mode.
pub struct PlayerInAdventureEvent;
impl SandEvent for PlayerInAdventureEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=adventure]".into())
    }
}
impl PersistentSandEvent for PlayerInAdventureEvent {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(crate::condition::Condition::entity_raw(
            "@s[gamemode=adventure]",
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerInSpectatorEvent",
    module = "sand::events",
    summary = "Fires every tick the player is in Spectator mode.",
    context = "Fires every tick the player is in Spectator mode. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Sand evaluates an entity gamemode=spectator condition per online player.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn spectator(_: sand::events::PlayerInSpectatorEvent) { sand::command::raw(\"say Spectator\"); }",
)]
/// Fires every tick the player is in Spectator mode.
pub struct PlayerInSpectatorEvent;
impl SandEvent for PlayerInSpectatorEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=spectator]".into())
    }
}
impl PersistentSandEvent for PlayerInSpectatorEvent {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(crate::condition::Condition::entity_raw(
            "@s[gamemode=spectator]",
        ))
    }
}

// ── Gamemode transitions (#49) ────────────────────────────────────────────
//
// One boolean tracker per gamemode (`entity @s[gamemode=<mode>]`), each
// shared by its Entered/Exited pair — four trackers total regardless of how
// many handlers subscribe. A single `PlayerGamemodeChangedEvent` carrying a
// typed previous/current payload is intentionally not provided: the current
// event-handler-context model (#230) has no mechanism for exposing an
// enum-typed "previous state" value inside a handler body honestly — adding
// one here would mean simulating a payload callers cannot safely read.
// Typed enter/exit pairs give the same information without that gap.

macro_rules! gamemode_transition {
    ($mode:literal, $tracker:literal, $enter:ident, $exit:ident) => {
        #[doc = concat!("Fires once when a player switches into ", $mode, " mode.")]
        pub struct $enter;
        ::sand_api_contract::inventory::submit! {
            ::sand_api_contract::ApiRegistration {
                canonical_path: concat!("sand::events::", stringify!($enter)),
                aliases: &[],
                canonical_module: "sand::events",
                kind: ::sand_api_contract::ApiKind::Struct,
                signature: concat!("pub struct ", stringify!($enter), ";"),
                summary: concat!("Marks the transition into Minecraft's ", $mode, " game mode."),
                context: "This generated zero-sized event marker lets an on_event handler react once per player transition without polling or maintaining its own prior-state tracker.",
                minecraft: concat!("Tracks each player against @s[gamemode=", $mode, "] and dispatches when the condition becomes true."),
                use_when: &["Handling the moment a player enters this game mode"],
                avoid_when: &["Checking a player's current game mode continuously"],
                parameters: &[],
                returns: None,
                example: concat!("use sand::events::", stringify!($enter), ";"),
                availability: &[],
            }
        }
        impl SandEvent for $enter {
            fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
                    $tracker,
                    crate::TrackedSource::BooleanCondition {
                        description: concat!("vanilla gamemode selector: ", $mode),
                        condition: concat!("entity @s[gamemode=", $mode, "]"),
                    },
                    crate::TransitionKind::BecameTrue,
                ))
            }
        }

        #[doc = concat!("Fires once when a player switches out of ", $mode, " mode.")]
        #[doc = concat!("Uses the same shared tracker as [`", stringify!($enter), "`].")]
        pub struct $exit;
        ::sand_api_contract::inventory::submit! {
            ::sand_api_contract::ApiRegistration {
                canonical_path: concat!("sand::events::", stringify!($exit)),
                aliases: &[],
                canonical_module: "sand::events",
                kind: ::sand_api_contract::ApiKind::Struct,
                signature: concat!("pub struct ", stringify!($exit), ";"),
                summary: concat!("Marks the transition out of Minecraft's ", $mode, " game mode."),
                context: "This generated zero-sized event marker shares the enter marker's tracker so a handler runs once when a player leaves this mode.",
                minecraft: concat!("Tracks each player against @s[gamemode=", $mode, "] and dispatches when the condition becomes false."),
                use_when: &["Handling the moment a player leaves this game mode"],
                avoid_when: &["Checking a player's current game mode continuously"],
                parameters: &[],
                returns: None,
                example: concat!("use sand::events::", stringify!($exit), ";"),
                availability: &[],
            }
        }
        impl SandEvent for $exit {
            fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
                    $tracker,
                    crate::TrackedSource::BooleanCondition {
                        description: concat!("vanilla gamemode selector: ", $mode),
                        condition: concat!("entity @s[gamemode=", $mode, "]"),
                    },
                    crate::TransitionKind::BecameFalse,
                ))
            }
        }
    };
}

gamemode_transition!(
    "survival",
    "player_gm_survival",
    PlayerEnteredSurvivalEvent,
    PlayerExitedSurvivalEvent
);
gamemode_transition!(
    "creative",
    "player_gm_creative",
    PlayerEnteredCreativeEvent,
    PlayerExitedCreativeEvent
);
gamemode_transition!(
    "adventure",
    "player_gm_adventure",
    PlayerEnteredAdventureEvent,
    PlayerExitedAdventureEvent
);
gamemode_transition!(
    "spectator",
    "player_gm_spectator",
    PlayerEnteredSpectatorEvent,
    PlayerExitedSpectatorEvent
);

// ── Health transitions (#49) ──────────────────────────────────────────────
//
// One shared per-player health provider backed by vanilla's `health`
// scoreboard criterion. The `health` criterion samples the player's current
// health **rounded down to an integer, 0-20 by default (more with the Health
// Boost/absorption-independent max health attribute), and does NOT include
// absorption hearts** — absorption is a separate, decaying overlay tracked by
// vanilla outside the `health` stat. Sand auto-declares the backing
// objective (`sand_health`) once at load time; it is not something callers
// need to pre-declare.

/// Shared current-state source for health-change transitions. Kept public
/// only for proc-macro expansion.
#[doc(hidden)]
pub(crate) const PLAYER_HEALTH_TRACKED_SOURCE: crate::TrackedSource = crate::TrackedSource::Score {
    description: "vanilla health scoreboard criterion (integer, excludes absorption)",
    objective: "sand_health",
    criterion: "health",
};

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerHealthChangedEvent",
    module = "sand::events",
    summary = "Fires once whenever a player's health value changes (gain or loss).",
    context = "Fires once whenever a player's health value changes (gain or loss). Backed by vanilla's `health` scoreboard criterion — an integer value, 0-20 by default, that does not include absorption hearts. The first tick after join/reload establishes a baseline and does not fire.",
    minecraft = "Backed by vanilla's `health` scoreboard criterion — an integer value, 0-20 by default, that does not include absorption hearts. The first tick after join/reload establishes a baseline and does not fire.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn health(_: sand::events::PlayerHealthChangedEvent) { sand::command::raw(\"say Health\"); }",
)]
/// Fires once whenever a player's health value changes (gain or loss).
///
/// Backed by vanilla's `health` scoreboard criterion — an integer value,
/// 0-20 by default, that does **not** include absorption hearts. The first
/// tick after join/reload establishes a baseline and does not fire.
pub struct PlayerHealthChangedEvent;
impl SandEvent for PlayerHealthChangedEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_health",
            PLAYER_HEALTH_TRACKED_SOURCE,
            crate::TransitionKind::ScoreChanged,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerHealthLostEvent",
    module = "sand::events",
    summary = "Fires once whenever a player's health value decreases.",
    context = "Fires once whenever a player's health value decreases. Shares the `player_health` tracker with [`PlayerHealthChangedEvent`] and [`PlayerHealthGainedEvent`].",
    minecraft = "Sand compares generated current and previous health score baselines.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn lost(_: sand::events::PlayerHealthLostEvent) { sand::command::raw(\"say Hurt\"); }",
)]
/// Fires once whenever a player's health value decreases.
///
/// Shares the `player_health` tracker with [`PlayerHealthChangedEvent`] and
/// [`PlayerHealthGainedEvent`].
pub struct PlayerHealthLostEvent;
impl SandEvent for PlayerHealthLostEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_health",
            PLAYER_HEALTH_TRACKED_SOURCE,
            crate::TransitionKind::ScoreDecreased,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerHealthGainedEvent",
    module = "sand::events",
    summary = "Fires once whenever a player's health value increases.",
    context = "Fires once whenever a player's health value increases. Shares the `player_health` tracker with [`PlayerHealthChangedEvent`] and [`PlayerHealthLostEvent`].",
    minecraft = "Sand compares generated current and previous health score baselines.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn gained(_: sand::events::PlayerHealthGainedEvent) { sand::command::raw(\"say Healed\"); }",
)]
/// Fires once whenever a player's health value increases.
///
/// Shares the `player_health` tracker with [`PlayerHealthChangedEvent`] and
/// [`PlayerHealthLostEvent`].
pub struct PlayerHealthGainedEvent;
impl SandEvent for PlayerHealthGainedEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_health",
            PLAYER_HEALTH_TRACKED_SOURCE,
            crate::TransitionKind::ScoreIncreased,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerLowHealthEvent",
    module = "sand::events",
    summary = "Fires once when a player's health drops to or below a threshold, and its counterpart [`PlayerRecoveredHealthEvent`] fires once when it rises back above that threshold.",
    context = "Fires once when a player's health drops to or below a threshold, and its counterpart [`PlayerRecoveredHealthEvent`] fires once when it rises back above that threshold. `HALF_HEARTS` is the threshold in half-hearts (vanilla's `health` criterion unit) — e.g. `PlayerLowHealthEvent<6>` fires at 3 hearts (6 half-hearts) or below. All instantiations share the single `player_low_health` tracker and the same `sand_health` objective as the change/gain/loss events above, so a pack using both never generates a second observer. Exactly one threshold value may be used per exported pack: mixing two different `HALF_HEARTS` values is a tracker-identity conflict the exporter rejects with a clear diagnostic (Sand cannot honestly share one previous/current baseline for two different boolean thresholds under one tracker id).",
    minecraft = "All instantiations share the single `player_low_health` tracker and the same `sand_health` objective as the change/gain/loss events above, so a pack using both never generates a second observer. Exactly one threshold value may be used per exported pack: mixing two different `HALF_HEARTS` values is a tracker-identity conflict the exporter rejects with a clear diagnostic (Sand cannot honestly share one previous/current baseline for two different boolean thresholds under one tracker id).",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn low(_: sand::event::Event<sand::events::PlayerLowHealthEvent<6>>) { sand::command::raw(\"say Low health\"); }",
)]
/// Fires once when a player's health drops to or below a threshold, and its
/// counterpart [`PlayerRecoveredHealthEvent`] fires once when it rises back
/// above that threshold.
///
/// `HALF_HEARTS` is the threshold in half-hearts (vanilla's `health`
/// criterion unit) — e.g. `PlayerLowHealthEvent<6>` fires at 3 hearts (6
/// half-hearts) or below.
///
/// All instantiations share the single `player_low_health` tracker and the
/// same `sand_health` objective as the change/gain/loss events above, so a
/// pack using both never generates a second observer. Exactly one threshold
/// value may be used per exported pack: mixing two different `HALF_HEARTS`
/// values is a tracker-identity conflict the exporter rejects with a clear
/// diagnostic (Sand cannot honestly share one previous/current baseline for
/// two different boolean thresholds under one tracker id).
///
/// ```rust,ignore
/// use sand_core::events::PlayerLowHealthEvent;
/// use sand_core::prelude::*;
/// use sand_macros::on_event;
///
/// #[on_event]
/// pub fn low_health_warning(event: Event<PlayerLowHealthEvent<6>>) {
///     cmd::say("Low health!");
/// }
/// ```
pub struct PlayerLowHealthEvent<const HALF_HEARTS: i32>;
impl<const HALF_HEARTS: i32> SandEvent for PlayerLowHealthEvent<HALF_HEARTS> {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_low_health",
            crate::TrackedSource::ScoreThreshold {
                description: "vanilla health scoreboard criterion, low-health threshold",
                objective: "sand_health",
                criterion: "health",
                comparator: crate::ScoreThresholdComparator::AtOrBelow(HALF_HEARTS),
            },
            crate::TransitionKind::BecameTrue,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::PlayerRecoveredHealthEvent",
    module = "sand::events",
    summary = "Fires once when a player's health rises back above [`PlayerLowHealthEvent`]'s threshold. `HALF_HEARTS` must match the corresponding `PlayerLowHealthEvent<HALF_HEARTS>` exactly — they share one tracker.",
    context = "Fires once when a player's health rises back above [`PlayerLowHealthEvent`]'s threshold. `HALF_HEARTS` must match the corresponding `PlayerLowHealthEvent<HALF_HEARTS>` exactly — they share one tracker. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Shares PlayerLowHealthEvent's generated health tracker, so both types must use the same half-heart threshold.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn recovered(_: sand::event::Event<sand::events::PlayerRecoveredHealthEvent<6>>) { sand::command::raw(\"say Recovered\"); }",
)]
/// Fires once when a player's health rises back above
/// [`PlayerLowHealthEvent`]'s threshold. `HALF_HEARTS` must match the
/// corresponding `PlayerLowHealthEvent<HALF_HEARTS>` exactly — they share
/// one tracker.
pub struct PlayerRecoveredHealthEvent<const HALF_HEARTS: i32>;
impl<const HALF_HEARTS: i32> SandEvent for PlayerRecoveredHealthEvent<HALF_HEARTS> {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            "player_low_health",
            crate::TrackedSource::ScoreThreshold {
                description: "vanilla health scoreboard criterion, low-health threshold",
                objective: "sand_health",
                criterion: "health",
                comparator: crate::ScoreThresholdComparator::AtOrBelow(HALF_HEARTS),
            },
            crate::TransitionKind::BecameFalse,
        ))
    }
}

// ── Status effect transitions (#49) ───────────────────────────────────────
//
// One reusable generic pair, `EffectStarted<E>`/`EffectStopped<E>`, backed
// by a per-effect Sand-owned `minecraft:entity_properties` predicate
// (`{"effects": {"<id>": {}}}`) — the same detection family already used for
// `flags.*` state (sneaking/sprinting/on-fire), just keyed on active effects
// instead of entity flags. Only effects with a registered `#[on_event]`
// handler generate a predicate + tracker; the other markers cost nothing.

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::StatusEffectMarker",
    module = "sand::events",
    summary = "Implemented by zero-sized status-effect marker types (e.g. [`Speed`]) so [`EffectStarted<E>`]/[`EffectStopped<E>`] can be generic over which vanilla effect they observe, without a hand-written detection type per effect.",
    context = "Implemented by zero-sized status-effect marker types (e.g. [`Speed`]) so [`EffectStarted<E>`]/[`EffectStopped<E>`] can be generic over which vanilla effect they observe, without a hand-written detection type per effect. Not intended for manual implementation — implemented internally by `status_effect_marker!` for every supported effect.",
    minecraft = "Sand uses its effect ID and predicate condition to build a shared transition tracker.",
    use_when = ["Defining, composing, or handling a typed Sand event"],
    avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
    example = "use sand::events::StatusEffectMarker;",
)]
/// Implemented by zero-sized status-effect marker types (e.g. [`Speed`]) so
/// [`EffectStarted<E>`]/[`EffectStopped<E>`] can be generic over which
/// vanilla effect they observe, without a hand-written detection type per
/// effect.
///
/// Not intended for manual implementation — implemented internally by
/// `status_effect_marker!` for every supported effect.
pub trait StatusEffectMarker: 'static {
    #[doc = "The vanilla status-effect resource identifier."]
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::StatusEffectMarker::EFFECT_ID",
        module = "sand::events",
        kind = "associated_const",
        summary = "The vanilla status-effect resource identifier.",
        context = "The vanilla status-effect resource identifier. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "It names the effect used by generated entity-properties predicates.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        example = "use sand::events::StatusEffectMarker;",
    )]
    #[doc(hidden)]
    const EFFECT_ID: &'static str;
    #[doc = "The stable Sand tracker identity for this status effect."]
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::StatusEffectMarker::TRACKER_ID",
        module = "sand::events",
        kind = "associated_const",
        summary = "The stable Sand tracker identity for this status effect.",
        context = "The stable Sand tracker identity for this status effect. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Handlers for the same effect share its generated previous/current state.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        example = "use sand::events::StatusEffectMarker;",
    )]
    #[doc(hidden)]
    const TRACKER_ID: &'static str;
    #[doc = "The typed condition fragment that observes this status effect."]
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::StatusEffectMarker::CONDITION",
        module = "sand::events",
        kind = "associated_const",
        summary = "The typed condition fragment that observes this status effect.",
        context = "The typed condition fragment that observes this status effect. This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
        minecraft = "Sand uses it to test the executing player's active effects.",
        use_when = ["Defining, composing, or handling a typed Sand event"],
        avoid_when = ["Inspecting generated advancement or event-graph implementation state"],
        example = "use sand::events::StatusEffectMarker;",
    )]
    #[doc(hidden)]
    const CONDITION: &'static str;
}

macro_rules! status_effect_marker {
    ($ty:ident, $id:literal, $tracker:literal, $condition:literal) => {
        #[doc = concat!("Marker type for the vanilla `", $id, "` status effect.")]
        #[doc = ""]
        #[doc = concat!("Use with [`EffectStarted<", stringify!($ty), ">`] / [`EffectStopped<", stringify!($ty), ">`].")]
        pub struct $ty;
        ::sand_api_contract::inventory::submit! {
            ::sand_api_contract::ApiRegistration {
                canonical_path: concat!("sand::events::", stringify!($ty)),
                aliases: &[],
                canonical_module: "sand::events",
                kind: ::sand_api_contract::ApiKind::Struct,
                signature: concat!("pub struct ", stringify!($ty), ";"),
                summary: concat!("Marks Minecraft's `", $id, "` status effect for typed transition events."),
                context: "This generated zero-sized marker selects one vanilla effect for EffectStarted and EffectStopped without requiring stringly typed effect names.",
                minecraft: concat!("Uses a generated entity-properties predicate for ", $id, " and a shared transition tracker."),
                use_when: &["Subscribing to this effect starting or stopping on a player"],
                avoid_when: &["Applying, removing, or representing an arbitrary status effect"],
                parameters: &[],
                returns: None,
                example: concat!("use sand::events::{EffectStarted, ", stringify!($ty), "};\n\nfn typed_event(_: EffectStarted<", stringify!($ty), ">) {}"),
                availability: &[],
            }
        }
        impl StatusEffectMarker for $ty {
            const EFFECT_ID: &'static str = $id;
            const TRACKER_ID: &'static str = $tracker;
            const CONDITION: &'static str = $condition;
        }
    };
}

status_effect_marker!(
    Poison,
    "minecraft:poison",
    "effect_poison",
    "predicate __sand_local:__sand/effect_poison"
);
status_effect_marker!(
    Wither,
    "minecraft:wither",
    "effect_wither",
    "predicate __sand_local:__sand/effect_wither"
);
status_effect_marker!(
    Regeneration,
    "minecraft:regeneration",
    "effect_regeneration",
    "predicate __sand_local:__sand/effect_regeneration"
);
status_effect_marker!(
    FireResistance,
    "minecraft:fire_resistance",
    "effect_fire_resist",
    "predicate __sand_local:__sand/effect_fire_resist"
);
status_effect_marker!(
    Strength,
    "minecraft:strength",
    "effect_strength",
    "predicate __sand_local:__sand/effect_strength"
);
status_effect_marker!(
    Weakness,
    "minecraft:weakness",
    "effect_weakness",
    "predicate __sand_local:__sand/effect_weakness"
);
status_effect_marker!(
    Speed,
    "minecraft:speed",
    "effect_speed",
    "predicate __sand_local:__sand/effect_speed"
);
status_effect_marker!(
    Slowness,
    "minecraft:slowness",
    "effect_slowness",
    "predicate __sand_local:__sand/effect_slowness"
);
status_effect_marker!(
    Resistance,
    "minecraft:resistance",
    "effect_resistance",
    "predicate __sand_local:__sand/effect_resistance"
);
status_effect_marker!(
    Absorption,
    "minecraft:absorption",
    "effect_absorption",
    "predicate __sand_local:__sand/effect_absorption"
);
status_effect_marker!(
    Hunger,
    "minecraft:hunger",
    "effect_hunger",
    "predicate __sand_local:__sand/effect_hunger"
);
status_effect_marker!(
    MiningFatigue,
    "minecraft:mining_fatigue",
    "effect_mining_fatigue",
    "predicate __sand_local:__sand/effect_mining_fatigue"
);
status_effect_marker!(
    Nausea,
    "minecraft:nausea",
    "effect_nausea",
    "predicate __sand_local:__sand/effect_nausea"
);
status_effect_marker!(
    Blindness,
    "minecraft:blindness",
    "effect_blindness",
    "predicate __sand_local:__sand/effect_blindness"
);
status_effect_marker!(
    Levitation,
    "minecraft:levitation",
    "effect_levitation",
    "predicate __sand_local:__sand/effect_levitation"
);
status_effect_marker!(
    Glowing,
    "minecraft:glowing",
    "effect_glowing",
    "predicate __sand_local:__sand/effect_glowing"
);
status_effect_marker!(
    Invisibility,
    "minecraft:invisibility",
    "effect_invisibility",
    "predicate __sand_local:__sand/effect_invisibility"
);

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::EffectStarted",
    module = "sand::events",
    summary = "Fires once when a player gains status effect `E` (was not active, is now active). See [`StatusEffectMarker`] for the supported markers (e.g. [`Speed`], [`Poison`]).",
    context = "Fires once when a player gains status effect `E` (was not active, is now active). See [`StatusEffectMarker`] for the supported markers (e.g. [`Speed`], [`Poison`]). This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Sand emits an entity-properties effect predicate and a false-to-true transition tracker for the selected effect.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn speed(_: sand::event::Event<sand::events::EffectStarted<sand::events::Speed>>) { sand::command::raw(\"say Speed\"); }",
)]
/// Fires once when a player gains status effect `E` (was not active, is now
/// active). See [`StatusEffectMarker`] for the supported markers (e.g.
/// [`Speed`], [`Poison`]).
///
/// ```rust,ignore
/// use sand_core::events::{EffectStarted, Speed};
/// use sand_core::prelude::*;
/// use sand_macros::on_event;
///
/// #[on_event]
/// pub fn on_speed_start(event: Event<EffectStarted<Speed>>) {
///     cmd::say("Speed boost!");
/// }
/// ```
pub struct EffectStarted<E: StatusEffectMarker>(std::marker::PhantomData<E>);
impl<E: StatusEffectMarker> SandEvent for EffectStarted<E> {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            E::TRACKER_ID,
            crate::TrackedSource::BooleanCondition {
                description: "vanilla entity_properties effects predicate",
                condition: E::CONDITION,
            },
            crate::TransitionKind::BecameTrue,
        ))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::EffectStopped",
    module = "sand::events",
    summary = "Fires once when a player loses status effect `E` (was active, is now not active — either it expired or was removed). Shares the tracker with [`EffectStarted<E>`].",
    context = "Fires once when a player loses status effect `E` (was active, is now not active — either it expired or was removed). Shares the tracker with [`EffectStarted<E>`]. This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
    minecraft = "Sand emits a true-to-false transition from the same generated effect tracker used by EffectStarted.",
    use_when = ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
    avoid_when = ["Representing mutable event data; read typed handler context or declared participants instead"],
    example = "#[sand::on_event]\nfn speed_end(_: sand::event::Event<sand::events::EffectStopped<sand::events::Speed>>) { sand::command::raw(\"say Speed ended\"); }",
)]
/// Fires once when a player loses status effect `E` (was active, is now not
/// active — either it expired or was removed). Shares the tracker with
/// [`EffectStarted<E>`].
pub struct EffectStopped<E: StatusEffectMarker>(std::marker::PhantomData<E>);
impl<E: StatusEffectMarker> SandEvent for EffectStopped<E> {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tracked(crate::TrackedTransition::new(
            E::TRACKER_ID,
            crate::TrackedSource::BooleanCondition {
                description: "vanilla entity_properties effects predicate",
                condition: E::CONDITION,
            },
            crate::TransitionKind::BecameFalse,
        ))
    }
}

// ── Doc-coverage registry ────────────────────────────────────────────────────
//
// Every public built-in event type exported from this module must appear in
// this list AND in `book/src/reference/event-trigger-matrix.md`. Workspace
// tests verify matrix coverage. When adding a new public event, append its type
// name here and add a row to the matrix.
//
// `SandEvent` and `SandEventDispatch` are excluded: they are traits/enums,
// not callable event types.
#[allow(dead_code)]
pub(crate) const BUILTIN_EVENT_NAMES: &[&str] = &[
    // Session
    "OnJoinEvent",
    "FirstJoinEvent",
    "OnDeathEvent",
    "OnRespawnEvent",
    // Equipment
    "ArmorEquipEvent",
    "ArmorUnequipEvent",
    "HoldingItemEvent",
    "CurrentlyWearingEvent",
    // Kill / combat
    "EntityKillEvent",
    "PlayerKillEvent",
    "PlayerDamageEntityEvent",
    "EntityDamagePlayerEvent",
    "ShotCrossbowEvent",
    "ChanneledLightningEvent",
    // Items
    "ItemConsumeEvent",
    "ItemCraftEvent",
    "ItemEnchantEvent",
    "BucketFillEvent",
    "BucketEmptyEvent",
    "FishingEvent",
    "ItemPickedUpEvent",
    "ItemDurabilityChangeEvent",
    "BrewPotionEvent",
    "TotemActivateEvent",
    "RecipeUnlockEvent",
    // Block / world
    "BlockPlaceEvent",
    "EnterBlockEvent",
    "SlideDownBlockEvent",
    "TargetHitEvent",
    "BeeNestDestroyedEvent",
    // Player state
    "ChangeDimensionEvent",
    "PlayerSleepEvent",
    "FallFromHeightEvent",
    "PlayerLevelUpEvent",
    "EffectsChangedEvent",
    "StartRidingEvent",
    "UseEnderEyeEvent",
    "HeroOfTheVillageEvent",
    "LightningStrikeEvent",
    // Entity / interaction
    "TameAnimalEvent",
    "BreedAnimalsEvent",
    "SummonEntityEvent",
    "InteractWithEntityEvent",
    "VillagerTradeEvent",
    "ConstructBeaconEvent",
    "CureZombieVillagerEvent",
    "LootContainerOpenEvent",
    // Tick-poll / continuous state
    "PlayerStartSneakingEvent",
    "PlayerStopSneakingEvent",
    "PlayerSneakEvent",
    "PlayerSprintEvent",
    "PlayerStartSprintingEvent",
    "PlayerStopSprintingEvent",
    "PlayerSwimmingEvent",
    "PlayerStartSwimmingEvent",
    "PlayerStopSwimmingEvent",
    "PlayerFlyingEvent",
    "PlayerStartFlyingEvent",
    "PlayerStopFlyingEvent",
    "PlayerOnFireEvent",
    "PlayerCaughtFireEvent",
    "PlayerExtinguishedEvent",
    "PlayerInCreativeEvent",
    "PlayerInAdventureEvent",
    "PlayerInSpectatorEvent",
    "PlayerEnteredSurvivalEvent",
    "PlayerExitedSurvivalEvent",
    "PlayerEnteredCreativeEvent",
    "PlayerExitedCreativeEvent",
    "PlayerEnteredAdventureEvent",
    "PlayerExitedAdventureEvent",
    "PlayerEnteredSpectatorEvent",
    "PlayerExitedSpectatorEvent",
    "PlayerHealthChangedEvent",
    "PlayerHealthLostEvent",
    "PlayerHealthGainedEvent",
    "PlayerLowHealthEvent",
    "PlayerRecoveredHealthEvent",
    "EffectStarted",
    "EffectStopped",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::AdvancementEvent;

    #[test]
    fn semantic_dispatch_builders_produce_opaque_dispatch_values() {
        let tick: SandEventDispatch = SandEventDispatch::tick().as_players().into();
        assert!(matches!(tick.normalize(), NormalizedEventDispatch::Tick(_)));

        let advancement = SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::Tick);
        assert!(matches!(
            advancement.normalize(),
            NormalizedEventDispatch::Advancement(crate::AdvancementTrigger::Tick)
        ));
    }

    #[test]
    fn player_level_up_event_is_not_deprecated() {
        // Compile-time: just instantiating the type confirms no deprecated attr.
        let _: PlayerLevelUpEvent = PlayerLevelUpEvent;
    }

    #[test]
    fn player_level_up_event_implements_advancement_event() {
        // The placeholder trigger must be Tick (safe; never emitted for XpLevelUp).
        let trigger = PlayerLevelUpEvent::trigger();
        // Tick trigger serializes to "minecraft:tick" — not "minecraft:leveled_up".
        let id = trigger.trigger_id();
        assert_ne!(id, "minecraft:leveled_up");
        assert_eq!(id, "minecraft:tick");
    }

    #[test]
    fn xp_score_vars_have_valid_names() {
        // Objective names must be ≤ 16 chars for Minecraft.
        assert!(SAND_XP_LVL.objective_name().len() <= 16);
        assert!(SAND_XP_PREV.objective_name().len() <= 16);
        assert!(SAND_XP_DELTA.objective_name().len() <= 16);
    }

    #[test]
    fn xp_objective_names_are_stable() {
        assert_eq!(SAND_XP_LVL.objective_name(), "__sand_xp_lvl");
        assert_eq!(SAND_XP_PREV.objective_name(), "__sand_xp_prev");
        assert_eq!(SAND_XP_DELTA.objective_name(), "__sand_xp_delta");
    }

    #[test]
    fn helper_current_level_generates_score_ref() {
        let score_ref = PlayerLevelUpEvent::current_level("@s");
        let operand = score_ref.operand();
        assert_eq!(operand.selector, "@s");
        assert_eq!(operand.objective, "__sand_xp_lvl");
    }

    #[test]
    fn helper_previous_level_generates_score_ref() {
        let score_ref = PlayerLevelUpEvent::previous_level("@s");
        let operand = score_ref.operand();
        assert_eq!(operand.selector, "@s");
        assert_eq!(operand.objective, "__sand_xp_prev");
    }

    #[test]
    fn helper_level_delta_generates_score_ref() {
        let score_ref = PlayerLevelUpEvent::level_delta("@s");
        let operand = score_ref.operand();
        assert_eq!(operand.selector, "@s");
        assert_eq!(operand.objective, "__sand_xp_delta");
    }

    #[test]
    fn player_levels_up_alias_is_same_type() {
        // crate::event::vanilla::PlayerLevelsUp is just a type alias — verify it
        // has the same helper methods available.
        let score_ref = crate::event::vanilla::PlayerLevelsUp::current_level("@s");
        let operand = score_ref.operand();
        assert_eq!(operand.objective, "__sand_xp_lvl");
    }

    #[test]
    fn builtin_event_names_has_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for name in super::BUILTIN_EVENT_NAMES {
            assert!(
                seen.insert(*name),
                "Duplicate entry in BUILTIN_EVENT_NAMES: {name}"
            );
        }
    }

    #[test]
    fn builtin_event_names_is_non_empty() {
        assert!(!super::BUILTIN_EVENT_NAMES.is_empty());
    }

    #[test]
    fn all_builtin_events_are_covered_in_the_reference_matrix() {
        let matrix_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../book/src/reference/event-trigger-matrix.md");
        let matrix = std::fs::read_to_string(&matrix_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", matrix_path.display()));
        let missing = super::BUILTIN_EVENT_NAMES
            .iter()
            .copied()
            .filter(|name| !matrix.contains(name))
            .collect::<Vec<_>>();
        assert!(
            missing.is_empty(),
            "built-in events missing from {}: {missing:?}",
            matrix_path.display()
        );
    }

    // ── TickEventDispatch / EventSetup lifecycle ──────────────────────────────

    #[test]
    fn tick_dispatch_when_renders_single_plan() {
        let d = SandEventDispatch::tick()
            .as_players()
            .when(crate::condition::Condition::raw(
                "score @s sync_jumps < @s jumps",
            ));
        assert_eq!(
            d.execution_plans(),
            TickExecutionPlans::Plans(vec![vec!["if score @s sync_jumps < @s jumps".to_string()]])
        );
    }

    #[test]
    fn tick_dispatch_when_and_unless_are_ordered_and_anded() {
        let d = SandEventDispatch::tick()
            .as_players()
            .when(crate::condition::Condition::raw(
                "score @s sync_jumps < @s jumps",
            ))
            .unless(crate::condition::Condition::raw(
                "score @s is_dead matches 1",
            ));
        let plans = d.execution_plans();
        let TickExecutionPlans::Plans(plans) = plans else {
            panic!("expected Plans");
        };
        assert_eq!(plans.len(), 1);
        let clauses = &plans[0];
        assert_eq!(clauses.len(), 2);
        // `when` clause must precede `unless` clause.
        assert_eq!(clauses[0], "if score @s sync_jumps < @s jumps");
        assert_eq!(clauses[1], "unless score @s is_dead matches 1");
    }

    #[test]
    fn tick_dispatch_if_alias_matches_when() {
        let a = SandEventDispatch::tick()
            .if_(crate::condition::Condition::raw("score @s a matches 1"))
            .execution_plans();
        let b = SandEventDispatch::tick()
            .when(crate::condition::Condition::raw("score @s a matches 1"))
            .execution_plans();
        assert_eq!(a, b);
    }

    #[test]
    fn tick_dispatch_no_conditions_is_explicitly_unconditional() {
        let d = SandEventDispatch::tick().as_players();
        assert_eq!(d.execution_plans(), TickExecutionPlans::Unconditional);
        assert!(d.execution_plans().is_unconditional());
    }

    #[test]
    fn tick_dispatch_every_tick_is_unconditional() {
        let d = SandEventDispatch::tick().as_players().every_tick();
        assert_eq!(d.execution_plans(), TickExecutionPlans::Unconditional);
    }

    #[test]
    fn tick_dispatch_unless_only_is_not_unconditional() {
        // A dispatch with only `.unless(...)` must still render a real
        // condition, not collapse to Unconditional.
        let d = SandEventDispatch::tick()
            .as_players()
            .unless(crate::condition::Condition::raw("score @s busy matches 1"));
        let plans = d.execution_plans();
        assert!(!plans.is_unconditional());
        assert_eq!(
            plans,
            TickExecutionPlans::Plans(vec![vec!["unless score @s busy matches 1".to_string()]])
        );
    }

    #[test]
    fn tick_dispatch_or_condition_yields_multiple_plans() {
        let d = SandEventDispatch::tick().as_players().when(
            crate::condition::Condition::raw("score @s a matches 1")
                .or(crate::condition::Condition::raw("score @s b matches 1")),
        );
        let plans = d.execution_plans();
        assert_eq!(
            plans,
            TickExecutionPlans::Plans(vec![
                vec!["if score @s a matches 1".to_string()],
                vec!["if score @s b matches 1".to_string()],
            ])
        );
    }

    #[test]
    fn tick_dispatch_empty_any_condition_yields_zero_plans_not_unconditional() {
        // A `when(Condition::any([]))` is a declared-but-unsatisfiable
        // condition (vacuous OR) — it must render as `Plans(vec![])` (never
        // fires), which is distinct from `Unconditional` (always fires).
        let d = SandEventDispatch::tick()
            .as_players()
            .when(crate::condition::Condition::any([]));
        let plans = d.execution_plans();
        assert!(!plans.is_unconditional());
        assert_eq!(plans, TickExecutionPlans::Plans(vec![]));
        assert!(plans.plans().is_empty());
    }

    #[test]
    fn dispatch_tick_builder_converts_into_sand_event_dispatch() {
        struct Jump;
        impl SandEvent for Jump {
            fn dispatch() -> SandEventDispatch {
                SandEventDispatch::tick()
                    .as_players()
                    .when(crate::condition::Condition::raw(
                        "score @s sync_jumps < @s jumps",
                    ))
                    .into()
            }
        }
        let dispatch: SandEventDispatch = Jump::dispatch();
        match dispatch.normalize() {
            NormalizedEventDispatch::Tick(t) => {
                assert_eq!(
                    t.execution_plans(),
                    TickExecutionPlans::Plans(vec![vec![
                        "if score @s sync_jumps < @s jumps".to_string()
                    ]])
                );
            }
            NormalizedEventDispatch::Advancement(_) => panic!("expected Tick"),
            NormalizedEventDispatch::Chain(_) => panic!("expected Tick"),
            NormalizedEventDispatch::Tracked(_) => panic!("expected Tick"),
        }
    }

    #[test]
    fn legacy_tick_condition_normalizes_to_structured_tick() {
        let dispatch = SandEventDispatch::TickCondition("entity @s[tag=ready]".into());
        match dispatch.normalize() {
            NormalizedEventDispatch::Tick(t) => {
                assert_eq!(
                    t.execution_plans(),
                    TickExecutionPlans::Plans(vec![vec!["if entity @s[tag=ready]".to_string()]])
                );
            }
            NormalizedEventDispatch::Advancement(_) => panic!("expected Tick"),
            NormalizedEventDispatch::Chain(_) => panic!("expected Tick"),
            NormalizedEventDispatch::Tracked(_) => panic!("expected Tick"),
        }
    }

    #[test]
    fn legacy_advancement_trigger_normalizes_unchanged() {
        let dispatch = SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::Tick);
        match dispatch.normalize() {
            NormalizedEventDispatch::Advancement(t) => {
                assert_eq!(t.trigger_id(), "minecraft:tick");
            }
            NormalizedEventDispatch::Tick(_) => panic!("expected Advancement"),
            NormalizedEventDispatch::Chain(_) => panic!("expected Advancement"),
            NormalizedEventDispatch::Tracked(_) => panic!("expected Advancement"),
        }
    }

    #[test]
    fn event_setup_default_is_empty() {
        let setup = EventSetup::none();
        assert!(setup.objectives.is_empty());
        assert!(setup.pre_observation.is_empty());
        assert!(setup.post_observation.is_empty());
    }

    #[test]
    fn event_setup_is_empty_covers_every_field() {
        assert!(EventSetup::none().is_empty());
        assert!(
            !EventSetup {
                objectives: vec!["x".into()],
                pre_observation: vec![],
                post_observation: vec![],
            }
            .is_empty()
        );
        assert!(
            !EventSetup {
                objectives: vec![],
                pre_observation: vec!["x".into()],
                post_observation: vec![],
            }
            .is_empty()
        );
        assert!(
            !EventSetup {
                objectives: vec![],
                pre_observation: vec![],
                post_observation: vec!["x".into()],
            }
            .is_empty()
        );
    }

    #[test]
    fn event_setup_first_non_empty_category_is_none_when_empty_and_prioritized_when_mixed() {
        assert_eq!(EventSetup::none().first_non_empty_category(), None);
        assert_eq!(
            EventSetup {
                objectives: vec!["x".into()],
                pre_observation: vec!["y".into()],
                post_observation: vec!["z".into()],
            }
            .first_non_empty_category(),
            Some("objectives")
        );
        assert_eq!(
            EventSetup {
                objectives: vec![],
                pre_observation: vec!["y".into()],
                post_observation: vec!["z".into()],
            }
            .first_non_empty_category(),
            Some("pre_observation")
        );
        assert_eq!(
            EventSetup {
                objectives: vec![],
                pre_observation: vec![],
                post_observation: vec!["z".into()],
            }
            .first_non_empty_category(),
            Some("post_observation")
        );
    }

    #[test]
    fn tick_window_rejects_zero() {
        assert_eq!(TickWindow::new(0), Err(TickWindowError::Zero));
    }

    #[test]
    fn tick_window_rejects_above_max() {
        assert_eq!(
            TickWindow::new(TickWindow::MAX_TICKS + 1),
            Err(TickWindowError::TooLarge {
                requested: TickWindow::MAX_TICKS + 1,
                max: TickWindow::MAX_TICKS,
            })
        );
    }

    #[test]
    fn tick_window_accepts_min_and_max() {
        assert_eq!(TickWindow::new(1).unwrap().ticks(), 1);
        assert_eq!(
            TickWindow::new(TickWindow::MAX_TICKS).unwrap().ticks(),
            TickWindow::MAX_TICKS
        );
    }

    #[test]
    fn tick_window_error_messages_are_actionable() {
        assert!(
            TickWindowError::Zero
                .to_string()
                .contains("at least 1 tick")
        );
        let too_large = TickWindowError::TooLarge {
            requested: 99_999,
            max: TickWindow::MAX_TICKS,
        };
        assert!(too_large.to_string().contains("99999"));
        assert!(
            too_large
                .to_string()
                .contains(&TickWindow::MAX_TICKS.to_string())
        );
    }

    #[test]
    fn tick_scope_has_player_subject_is_deterministic_and_never_reflective() {
        // Both scopes that can back a graph parent guarantee a player
        // subject; neither inspects handler code or runtime state to decide
        // this — the fact is a pure function of the enum variant.
        assert!(TickScope::Players.has_player_subject());
        assert!(TickScope::AdvancementPlayer.has_player_subject());
        assert_eq!(TickScope::default(), TickScope::Players);
    }
}
