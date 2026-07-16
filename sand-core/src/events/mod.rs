//! Built-in event types and the advanced [`SandEvent`] custom-event trait.
//!
//! New custom advancement-backed events should implement
//! [`AdvancementEvent`](crate::event::AdvancementEvent) and use
//! [`Event<T>`](crate::event::Event) as the handler parameter:
//!
//! ```rust,ignore
//! use sand_core::prelude::*;
//! use sand_core::event::trigger::ConsumeItemTrigger;
//! use sand_macros::event;
//!
//! pub struct AteGoldenAppleEvent;
//!
//! impl AdvancementEvent for AteGoldenAppleEvent {
//!     type Trigger = ConsumeItemTrigger;
//!     fn trigger() -> Self::Trigger {
//!         ConsumeItemTrigger::new().item(ItemPredicate::id("minecraft:golden_apple"))
//!     }
//! }
//!
//! #[event]
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
//! | [`OnRespawnEvent`] | Tick after respawning from death | — |
//! | [`ArmorEquipEvent`] | Item equipped in an equipment slot | `slot` |
//! | [`ArmorUnequipEvent`] | Item removed from an equipment slot | `slot` |
//! | [`HoldingItemEvent`] | Holding item (every tick) | `item` |
//! | [`CurrentlyWearingEvent`] | Wearing item in armor slot (every tick) | `slot`, `item` |
//!
//! # Usage
//!
//! Use the `#[event]` attribute macro from `sand_macros` on a free-standing
//! function. The primary handler parameter is `Event<T>` where `T` implements
//! [`AdvancementEvent`](crate::event::AdvancementEvent):
//!
//! ```rust,ignore
//! use sand_macros::event;
//! use sand_core::prelude::*;
//! use sand_core::events::{OnJoinEvent, OnDeathEvent, ArmorEquipEvent};
//!
//! static TOTAL_DEATHS: ScoreVar<i32> = ScoreVar::new("total_deaths");
//!
//! #[event]
//! pub fn on_join(event: Event<OnJoinEvent>) {
//!     cmd::tellraw(
//!         Selector::self_(),
//!         Text::new("Welcome!").gold(),
//!     );
//! }
//!
//! #[event]
//! pub fn on_death(event: Event<OnDeathEvent>) {
//!     TOTAL_DEATHS.add(event.player(), 1);
//! }
//!
//! // Slot filter required; item is optional
//! #[event(slot = Head, item = "minecraft:diamond_helmet")]
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
//! use sand_macros::event;
//!
//! pub struct AteGoldenAppleEvent;
//!
//! impl AdvancementEvent for AteGoldenAppleEvent {
//!     type Trigger = ConsumeItemTrigger;
//!     fn trigger() -> Self::Trigger {
//!         ConsumeItemTrigger::new().item(ItemPredicate::id("minecraft:golden_apple"))
//!     }
//! }
//!
//! #[event]
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
//! use sand_macros::event;
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
//! #[event]
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
//! [`SandEventDispatch::TickCondition`] — both lower into the same normalized
//! IR as [`SandEventDispatch::tick()`] (see [`SandEventDispatch::normalize`]).

/// Event dependency graph construction for same-cycle chained dispatch (#240).
pub mod graph;

// ── Custom event API ──────────────────────────────────────────────────────────

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
    pub fn has_player_subject(self) -> bool {
        matches!(self, Self::Players | Self::AdvancementPlayer)
    }
}

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
    pub fn players(condition: impl Into<crate::condition::Condition>) -> Self {
        Self {
            scope: TickScope::Players,
            condition: condition.into(),
        }
    }

    /// The execution scope required by this condition.
    pub fn scope(&self) -> TickScope {
        self.scope
    }
}

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
    fn persistent_condition() -> PersistentEventCondition;
}

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
    /// The smallest representable window: current-cycle occurrence only.
    pub const MIN_TICKS: u32 = 1;
    /// The largest representable window (20 minutes at 20 ticks/second).
    ///
    /// Bounded correlation is meant for short cross-tick coordination
    /// windows, not long-lived session state — use durable per-player state
    /// (e.g. `sand_core::state`) instead.
    pub const MAX_TICKS: u32 = 24_000;

    /// Validate `ticks` as a bounded correlation window.
    ///
    /// Returns [`TickWindowError::Zero`] for `0` and
    /// [`TickWindowError::TooLarge`] above [`TickWindow::MAX_TICKS`].
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
    pub fn ticks(self) -> u32 {
        self.0
    }
}

/// [`TickWindow::new`] validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickWindowError {
    /// A window must cover at least the current cycle (`N >= 1`).
    Zero,
    /// `requested` exceeds [`TickWindow::MAX_TICKS`].
    TooLarge { requested: u32, max: u32 },
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
pub struct BoundedEventDependency {
    #[doc(hidden)]
    pub event_type_id: fn() -> std::any::TypeId,
    #[doc(hidden)]
    pub event_type_name: fn() -> &'static str,
    #[doc(hidden)]
    pub event_dispatch: fn() -> SandEventDispatch,
    #[doc(hidden)]
    pub event_setup: fn() -> EventSetup,
    #[doc(hidden)]
    pub window: TickWindow,
}

/// One typed persistent-state dependency attached to a chained event.
pub struct PersistentEventDependency {
    #[doc(hidden)]
    pub event_type_id: fn() -> std::any::TypeId,
    #[doc(hidden)]
    pub event_type_name: fn() -> &'static str,
    #[doc(hidden)]
    pub event_dispatch: fn() -> SandEventDispatch,
    #[doc(hidden)]
    pub event_setup: fn() -> EventSetup,
    #[doc(hidden)]
    pub make_condition: fn() -> PersistentEventCondition,
}

/// One typed same-cycle event occurrence dependency.
#[derive(Clone, Copy)]
pub struct SameCycleEventDependency {
    #[doc(hidden)]
    pub event_type_id: fn() -> std::any::TypeId,
    #[doc(hidden)]
    pub event_type_name: fn() -> &'static str,
    #[doc(hidden)]
    pub event_dispatch: fn() -> SandEventDispatch,
    #[doc(hidden)]
    pub event_setup: fn() -> EventSetup,
    /// Whether this parent's advancement is revoked after firing —
    /// [`SandEvent::revoke`]. Only meaningful when the parent resolves to
    /// advancement-backed dispatch (#240 Phase 6); ignored for tick-backed
    /// parents, which have no advancement to revoke.
    #[doc(hidden)]
    pub event_revoke: fn() -> bool,
}

impl SameCycleEventDependency {
    fn of<E: SandEvent + 'static>() -> Self {
        Self {
            event_type_id: std::any::TypeId::of::<E>,
            event_type_name: std::any::type_name::<E>,
            event_dispatch: || E::dispatch().into(),
            event_setup: E::setup,
            event_revoke: E::revoke,
        }
    }
}

/// One explicit same-cycle occurrence clause in a composed event definition.
pub enum SameCycleEventRequirement {
    /// One concrete parent must have fired.
    After(SameCycleEventDependency),
    /// At least one parent in the group must have fired.
    AfterAny(Vec<SameCycleEventDependency>),
    /// Every parent in the group must have fired.
    AfterAll(Vec<SameCycleEventDependency>),
}

mod event_group_private {
    pub trait Sealed {}
}

/// A typed tuple of two through eight concrete [`SandEvent`] parent types.
///
/// This trait is implemented by Sand for supported tuple arities and is not
/// intended for manual implementation.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a supported same-cycle event group",
    label = "expected a tuple of 2 through 8 concrete `SandEvent` types",
    note = "use `after::<E>()` for one parent, or `after_any::<(A, B)>()` / `after_all::<(A, B)>()` for 2 through 8 parents"
)]
pub trait SameCycleEventGroup: event_group_private::Sealed {
    #[doc(hidden)]
    fn dependencies() -> Vec<SameCycleEventDependency>;
}

macro_rules! impl_same_cycle_event_group {
    ($($event:ident),+ $(,)?) => {
        impl<$($event: SandEvent + 'static),+> event_group_private::Sealed for ($($event,)+) {}

        impl<$($event: SandEvent + 'static),+> SameCycleEventGroup for ($($event,)+) {
            fn dependencies() -> Vec<SameCycleEventDependency> {
                vec![$(SameCycleEventDependency::of::<$event>()),+]
            }
        }
    };
}

impl_same_cycle_event_group!(A, B);
impl_same_cycle_event_group!(A, B, C);
impl_same_cycle_event_group!(A, B, C, D);
impl_same_cycle_event_group!(A, B, C, D, E);
impl_same_cycle_event_group!(A, B, C, D, E, F);
impl_same_cycle_event_group!(A, B, C, D, E, F, G);
impl_same_cycle_event_group!(A, B, C, D, E, F, G, H);

/// Lifecycle resources a [`SandEvent`] owns: objectives to create at load time,
/// commands to run before each observation, and commands to run after a
/// successful observation (e.g. synchronizing a delta-tracking score).
///
/// Returned by [`SandEvent::setup`]. When multiple `#[event]` handlers
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
    pub fn none() -> Self {
        Self::default()
    }
}

/// Explicit result of expanding a [`TickEventDispatch`]'s conditions into
/// concrete `execute` clause plans.
///
/// This exists specifically so "no conditions were declared" and "the
/// condition expands into more than one OR-alternative execute plan" can
/// never be conflated into a single `None` — every caller must handle both
/// cases explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TickExecutionPlans {
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

impl TickExecutionPlans {
    /// `true` if this is [`Unconditional`](Self::Unconditional).
    pub fn is_unconditional(&self) -> bool {
        matches!(self, Self::Unconditional)
    }

    /// The OR-alternative plans, or an empty slice for
    /// [`Unconditional`](Self::Unconditional).
    pub fn plans(&self) -> &[Vec<String>] {
        match self {
            Self::Unconditional => &[],
            Self::Plans(p) => p,
        }
    }
}

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
    pub scope: TickScope,
    /// Positive conditions — all must hold (ANDed).
    pub when: Vec<crate::condition::Condition>,
    /// Negative conditions — none may hold (ANDed as `unless`).
    pub unless: Vec<crate::condition::Condition>,
}

impl TickEventDispatch {
    /// Evaluate as each online player. Currently the only supported scope;
    /// present for API clarity and forward-compatibility with future scopes.
    pub fn as_players(mut self) -> Self {
        self.scope = TickScope::Players;
        self
    }

    /// Add a positive condition — the event fires only while this holds.
    ///
    /// Multiple calls are ANDed together.
    pub fn when(mut self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.when.push(condition.into());
        self
    }

    /// Ergonomic alias for [`when`](Self::when).
    pub fn if_(self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.when(condition)
    }

    /// Add a negative condition — the event does not fire while this holds.
    ///
    /// Multiple calls are ANDed together (i.e. every `unless` condition must
    /// fail to hold).
    pub fn unless(mut self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.unless.push(condition.into());
        self
    }

    /// No-op cadence marker: the event is checked every tick.
    ///
    /// Present so dispatch definitions can be explicit about cadence; there
    /// is currently no other supported cadence.
    pub fn every_tick(self) -> Self {
        self
    }

    /// Combine `when`/`unless` into a single [`Condition`](crate::condition::Condition),
    /// or `None` if no conditions were declared (dispatch is unconditional).
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
    pub fn execution_plans(&self) -> TickExecutionPlans {
        match self.combined_condition() {
            None => TickExecutionPlans::Unconditional,
            Some(combined) => TickExecutionPlans::Plans(combined.to_execute_plans(false)),
        }
    }
}

impl From<TickEventDispatch> for SandEventDispatch {
    fn from(tick: TickEventDispatch) -> Self {
        SandEventDispatch::Tick(tick)
    }
}

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
/// See the `#[event]` macro, which supplies these factories automatically
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
    pub occurrence: Vec<SameCycleEventRequirement>,
    /// Persistent current-state requirements, kept distinct from the
    /// same-cycle occurrence parent and from ordinary anonymous conditions.
    pub persistent: Vec<PersistentEventDependency>,
    /// Bounded cross-tick correlation requirements. Distinct from `occurrence`
    /// (same-cycle only) and `persistent` (current state, no occurrence). See
    /// [`ChainEventDispatch::within`].
    pub bounded: Vec<BoundedEventDependency>,
    /// Positive conditions — all must hold (ANDed) for this child to fire
    /// once its occurrence requirements are satisfied.
    pub when: Vec<crate::condition::Condition>,
    /// Negative conditions — none may hold.
    pub unless: Vec<crate::condition::Condition>,
}

impl ChainEventDispatch {
    /// Require one additional event to have fired for the same subject during
    /// the current event cycle.
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
    pub fn after_any<G: SameCycleEventGroup>(mut self) -> Self {
        self.occurrence
            .push(SameCycleEventRequirement::AfterAny(G::dependencies()));
        self
    }

    /// Require every event in `G` to have fired for the same subject during
    /// the current event cycle.
    ///
    /// `G` is a tuple of two through eight concrete [`SandEvent`] types.
    /// Multiple `after_all` groups in one definition are rejected at export.
    pub fn after_all<G: SameCycleEventGroup>(mut self) -> Self {
        self.occurrence
            .push(SameCycleEventRequirement::AfterAll(G::dependencies()));
        self
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
    pub fn while_<E: PersistentSandEvent + 'static>(mut self) -> Self {
        self.persistent.push(PersistentEventDependency {
            event_type_id: std::any::TypeId::of::<E>,
            event_type_name: std::any::type_name::<E>,
            event_dispatch: || E::dispatch().into(),
            event_setup: E::setup,
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
    pub fn within<E: SandEvent + 'static>(mut self, window: TickWindow) -> Self {
        self.bounded.push(BoundedEventDependency {
            event_type_id: std::any::TypeId::of::<E>,
            event_type_name: std::any::type_name::<E>,
            event_dispatch: || E::dispatch().into(),
            event_setup: E::setup,
            window,
        });
        self
    }

    /// Add a positive condition — the child fires only while this holds, in
    /// addition to the parent having fired this cycle.
    ///
    /// Multiple calls are ANDed together.
    pub fn when(mut self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.when.push(condition.into());
        self
    }

    /// Ergonomic alias for [`when`](Self::when).
    pub fn if_(self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.when(condition)
    }

    /// Add a negative condition — the child does not fire while this holds.
    ///
    /// Multiple calls are ANDed together (i.e. every `unless` condition must
    /// fail to hold).
    pub fn unless(mut self, condition: impl Into<crate::condition::Condition>) -> Self {
        self.unless.push(condition.into());
        self
    }

    /// Combine `when`/`unless` into a single [`Condition`](crate::condition::Condition),
    /// or `None` if no conditions were declared (the child fires
    /// unconditionally whenever its parent fires).
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

    /// Expand this child's conditions into explicit [`TickExecutionPlans`],
    /// same shape as [`TickEventDispatch::execution_plans`].
    pub fn execution_plans(&self) -> TickExecutionPlans {
        match self.combined_condition() {
            None => TickExecutionPlans::Unconditional,
            Some(combined) => TickExecutionPlans::Plans(combined.to_execute_plans(false)),
        }
    }
}

impl From<ChainEventDispatch> for SandEventDispatch {
    fn from(chain: ChainEventDispatch) -> Self {
        SandEventDispatch::Chain(chain)
    }
}

/// How a custom [`SandEvent`] is dispatched at runtime.
///
/// Returned by [`SandEvent::dispatch`]. Sand inspects this at build time to
/// generate the correct detection mechanism (advancement JSON or tick loop).
#[allow(clippy::large_enum_variant)]
pub enum SandEventDispatch {
    /// The event fires when the given advancement trigger criteria are met.
    ///
    /// Sand generates an advancement JSON file and wires the handler function
    /// as its reward. The advancement is revoked after firing (by default) so
    /// it can trigger again next time.
    AdvancementTrigger(crate::AdvancementTrigger),

    /// The event fires every tick an `execute if <condition>` is satisfied,
    /// evaluated as each online player.
    ///
    /// The string must be a valid Minecraft `execute if` sub-command, e.g.:
    ///
    /// - `"items entity @s mainhand minecraft:diamond_sword"` — holding a sword
    /// - `"score @s my_flag matches 1"` — scoreboard flag is set
    /// - `"predicate my_pack:some_predicate"` — custom predicate
    ///
    /// This is the simple, single-fragment form. Prefer
    /// [`SandEventDispatch::tick`] for typed conditions built from Sand's
    /// [`Condition`](crate::condition::Condition) IR, or when the event needs
    /// owned lifecycle resources via [`SandEvent::setup`].
    TickCondition(String),

    /// Structured, typed tick-poll dispatch. See [`TickEventDispatch`].
    Tick(TickEventDispatch),

    /// Structured, same-cycle chained dispatch. See [`ChainEventDispatch`]
    /// and [`SandEventDispatch::chain`].
    Chain(ChainEventDispatch),
}

/// Normalized internal representation of a [`SandEventDispatch`], used by the
/// export pipeline and by tests asserting on lowering behavior.
///
/// Every `SandEventDispatch` variant — including the legacy `AdvancementTrigger`
/// and `TickCondition` compatibility constructors — lowers into one of these
/// shapes, so the exporter has a single normalized IR to consume rather
/// than juggling multiple representations.
#[allow(clippy::large_enum_variant)]
pub enum NormalizedEventDispatch {
    /// Advancement-backed dispatch.
    Advancement(crate::AdvancementTrigger),
    /// Tick-poll dispatch, always in the structured [`TickEventDispatch`] shape.
    Tick(TickEventDispatch),
    /// Same-cycle chained dispatch. See [`ChainEventDispatch`].
    Chain(ChainEventDispatch),
}

impl SandEventDispatch {
    /// Construct a structured, typed tick-poll dispatch builder.
    ///
    /// ```rust,ignore
    /// SandEventDispatch::tick()
    ///     .as_players()
    ///     .when(SYNC_JUMPS.of("@s").lt_score(JUMPS.of("@s")))
    /// ```
    pub fn tick() -> TickEventDispatch {
        TickEventDispatch::default()
    }

    /// Construct a structured, same-cycle chained dispatch builder.
    ///
    /// Declares that this event evaluates only from `Parent`'s successful
    /// dispatch cycle, inheriting its execution subject and position, rather
    /// than independently re-detecting `Parent`'s condition. `Parent` need
    /// not have any direct `#[event]` handler of its own — only a `SandEvent`
    /// impl.
    ///
    /// ```rust,ignore
    /// SandEventDispatch::chain::<PlayerJumpEvent>()
    ///     .when(Condition::raw("block ~ ~-1 ~ minecraft:white_wool"))
    /// ```
    pub fn chain<P: SandEvent + 'static>() -> ChainEventDispatch {
        Self::compose().after::<P>()
    }

    /// Construct a same-cycle composition builder without choosing a parent.
    ///
    /// Add at least one [`ChainEventDispatch::after`],
    /// [`ChainEventDispatch::after_any`], or
    /// [`ChainEventDispatch::after_all`] clause before returning it from
    /// [`SandEvent::dispatch`]. Empty compositions are rejected at export.
    pub fn compose() -> ChainEventDispatch {
        ChainEventDispatch {
            occurrence: Vec::new(),
            persistent: Vec::new(),
            bounded: Vec::new(),
            when: Vec::new(),
            unless: Vec::new(),
        }
    }

    /// Start a typed any-parent same-cycle composition.
    pub fn after_any<G: SameCycleEventGroup>() -> ChainEventDispatch {
        Self::compose().after_any::<G>()
    }

    /// Start a typed all-parent same-cycle composition.
    pub fn after_all<G: SameCycleEventGroup>() -> ChainEventDispatch {
        Self::compose().after_all::<G>()
    }

    /// Lower this dispatch into the normalized internal IR.
    ///
    /// - `AdvancementTrigger(t)` → `Advancement(t)` unchanged.
    /// - `TickCondition(s)` → `Tick(...)` with `s` carried as a single
    ///   [`Condition::raw`](crate::condition::Condition::raw) `when` clause.
    /// - `Tick(t)` → `Tick(t)` unchanged.
    /// - `Chain(c)` → `Chain(c)` unchanged.
    pub fn normalize(self) -> NormalizedEventDispatch {
        match self {
            SandEventDispatch::AdvancementTrigger(t) => NormalizedEventDispatch::Advancement(t),
            SandEventDispatch::TickCondition(s) => NormalizedEventDispatch::Tick(
                TickEventDispatch::default().when(crate::condition::Condition::raw(s)),
            ),
            SandEventDispatch::Tick(t) => NormalizedEventDispatch::Tick(t),
            SandEventDispatch::Chain(c) => NormalizedEventDispatch::Chain(c),
        }
    }
}

/// Implement this trait on your own type to define a custom Sand event.
///
/// Your concrete type is the single parameter of a custom `#[event]` handler.
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
/// use sand_macros::event;
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
/// #[event]
/// pub fn on_ready(_event: PlayerReady) {
///     cmd::say("Ready!");
/// }
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` is used as a bare `#[event]` handler parameter but does not implement `SandEvent`",
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
    fn dispatch() -> impl Into<SandEventDispatch>;

    /// Lifecycle resources this event owns: objectives, pre-observation, and
    /// post-observation commands.
    ///
    /// Defaults to [`EventSetup::none`]. Override for events that need to
    /// create scoreboard objectives or run commands around detection — see
    /// [`EventSetup`] for the ordering guarantee (detection always runs
    /// before `post_observation`).
    ///
    /// When several `#[event]` handlers subscribe to the same event type,
    /// Sand deduplicates setup by the event's in-process type identity so
    /// objectives and detector functions are only emitted once.
    fn setup() -> EventSetup {
        EventSetup::none()
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
    fn revoke() -> bool {
        true
    }
}

// ── Built-in event marker types ───────────────────────────────────────────────

/// Fires on the first tick after a server start, `/reload`, or when a new
/// player joins mid-session.
///
/// The preferred short name is [`sand_core::event::vanilla::OnJoin`](crate::event::vanilla::OnJoin).
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
/// #[event]
/// pub fn on_join(event: Event<OnJoinEvent>) {
///     cmd::tellraw(
///         Selector::self_(),
///         Text::new("Welcome back!").gold(),
///     );
/// }
/// ```
pub struct OnJoinEvent;

/// Fires the very first time a player ever joins. Never fires again.
///
/// The preferred short name is [`sand_core::event::vanilla::FirstJoin`](crate::event::vanilla::FirstJoin).
///
/// Implemented as an `Advancement + Tick` trigger **without** revocation.
/// Once the advancement is granted it stays, so the event fires exactly once
/// per player across all sessions.
///
/// # Example
///
/// ```rust,ignore
/// #[event]
/// pub fn first_join(event: Event<FirstJoinEvent>) {
///     cmd::tellraw(
///         Selector::self_(),
///         Text::new("Welcome for the very first time!").aqua(),
///     );
///     cmd::give(Selector::self_(), "minecraft:diamond").count(3);
/// }
/// ```
pub struct FirstJoinEvent;

/// Fires on the tick a player dies (any cause: mob, fall, void, `/kill`, …).
///
/// The preferred short name is [`sand_core::event::vanilla::OnDeath`](crate::event::vanilla::OnDeath).
///
/// Implemented via the `deathCount` scoreboard criterion. The handler runs as
/// `@s` = the dying player.
///
/// # Example
///
/// ```rust,ignore
/// static TOTAL_DEATHS: ScoreVar<i32> = ScoreVar::new("total_deaths");
///
/// #[event]
/// pub fn on_death(event: Event<OnDeathEvent>) {
///     TOTAL_DEATHS.add(event.player(), 1);
/// }
/// ```
pub struct OnDeathEvent;

/// Fires on the tick after a player respawns from death.
///
/// The preferred short name is [`sand_core::event::vanilla::OnRespawn`](crate::event::vanilla::OnRespawn).
///
/// Sand tags each dying player with `__sand_was_dead` during the death check.
/// Each tick, any player with that tag who is no longer in spectator mode
/// (i.e. has respawned) triggers this event, then the tag is removed.
///
/// # Example
///
/// ```rust,ignore
/// #[event]
/// pub fn on_respawn(event: Event<OnRespawnEvent>) {
///     cmd::tellraw(
///         Selector::self_(),
///         Text::new("You respawned!").green(),
///     );
/// }
/// ```
pub struct OnRespawnEvent;

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
/// #[event(slot = Feet)]
/// pub fn any_boots_equipped(event: Event<ArmorEquipEvent>) {
///     cmd::say("Boots equipped!");
/// }
///
/// // Specific item with custom NBT
/// #[event(slot = Feet, item = "minecraft:leather_boots", custom_data = "{mana_boots:1b}")]
/// pub fn mana_boots_equipped(event: Event<ArmorEquipEvent>) {
///     MANA_REGEN.enable(event.player());
/// }
/// ```
pub struct ArmorEquipEvent;

/// Fires on the tick a player **removes** an item from an equipment slot.
///
/// Same filter syntax as [`ArmorEquipEvent`].
///
/// # Example
///
/// ```rust,ignore
/// static MANA_REGEN: Flag = Flag::new("mana_regen");
///
/// #[event(slot = Feet, item = "minecraft:leather_boots", custom_data = "{mana_boots:1b}")]
/// pub fn mana_boots_removed(event: Event<ArmorUnequipEvent>) {
///     MANA_REGEN.disable(event.player());
/// }
/// ```
pub struct ArmorUnequipEvent;

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
/// #[event(item = "minecraft:diamond_sword")]
/// pub fn holding_diamond_sword(event: Event<HoldingItemEvent>) {
///     cmd::particle(Particle::Crit, event.player());
/// }
///
/// #[event(item = "minecraft:shield", slot = Offhand)]
/// pub fn holding_shield_offhand(event: Event<HoldingItemEvent>) {
///     BLOCKING.enable(event.player());
/// }
/// ```
pub struct HoldingItemEvent;

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
/// #[event(slot = Head, item = "minecraft:diamond_helmet")]
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
// `#[event]`. Most map 1:1 to a Minecraft advancement trigger so they fire
// once per trigger occurrence and revoke themselves (unless noted).
// For filter-level customisation (e.g. specific item/entity), implement your
// own type with [`SandEvent`] using the same trigger and supply conditions.

// ── Kill / combat ─────────────────────────────────────────────────────────

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
/// #[event]
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
}

/// Fires when any entity kills the player.
///
/// Maps to `minecraft:entity_killed_player` with no conditions.
///
/// # Example
/// ```rust,ignore
/// #[event]
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
}

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
}

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
}

/// Fires when the player shoots a crossbow.
pub struct ShotCrossbowEvent;
impl SandEvent for ShotCrossbowEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ShotCrossbow {
            item: None,
        })
    }
}

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

/// Fires when the player consumes any item (food, potion, etc.).
///
/// Maps to `minecraft:consume_item`.
///
/// # Example
/// ```rust,ignore
/// #[event]
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

/// Compatibility marker for the removed `minecraft:crafted_item` trigger.
/// Target-aware export rejects it with a migration diagnostic because current
/// vanilla's `minecraft:recipe_crafted` requires a concrete recipe ID.
pub struct ItemCraftEvent;
impl SandEvent for ItemCraftEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::CraftedItem { item: None })
    }
}

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

/// Fires when the player brews a potion.
///
/// Maps to `minecraft:brewed_potion`.
pub struct BrewPotionEvent;
impl SandEvent for BrewPotionEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::brewed_any_potion())
    }
}

/// Fires when the player activates a totem of undying.
///
/// Maps to `minecraft:used_totem`.
pub struct TotemActivateEvent;
impl SandEvent for TotemActivateEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::UsedTotem { item: None })
    }
}

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

/// Fires when the player places any block.
///
/// Maps to `minecraft:placed_block` with no filters.
///
/// # Example
/// ```rust,ignore
/// static BLOCKS_PLACED: ScoreVar<i32> = ScoreVar::new("blocks_placed");
///
/// #[event]
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

/// Fires when the player enters a block (e.g. water, honey).
///
/// Maps to `minecraft:enter_block` with no block filter.
pub struct EnterBlockEvent;
impl SandEvent for EnterBlockEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::enter_block(None, None))
    }
}

/// Fires when the player slides down a block (e.g. honey block wall).
///
/// Maps to `minecraft:slide_down_block`.
pub struct SlideDownBlockEvent;
impl SandEvent for SlideDownBlockEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::slide_down_block(None))
    }
}

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

/// Fires when the player changes dimension (e.g. entering the Nether or End).
///
/// Maps to `minecraft:changed_dimension`.
///
/// # Example
/// ```rust,ignore
/// #[event]
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

/// Fires when the player falls from a height and lands.
///
/// Maps to `minecraft:fall_from_height`.
///
/// # Example
/// ```rust,ignore
/// #[event]
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

/// Fires when a player's XP level increases (gains one or more levels in a tick).
///
/// The preferred short name is
/// [`sand_core::event::vanilla::PlayerLevelsUp`](crate::event::vanilla::PlayerLevelsUp).
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
/// use sand_core::event::vanilla::PlayerLevelsUp;
/// use sand_core::events::PlayerLevelUpEvent;
/// use sand_core::prelude::*;
/// use sand_macros::event;
///
/// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
///
/// #[event]
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
    /// #[event]
    /// pub fn on_level_up(event: Event<PlayerLevelUpEvent>) {
    ///     let lvl = PlayerLevelUpEvent::current_level("@s");
    /// }
    /// ```
    ///
    /// [`ScoreRef`]: crate::state::score::ScoreRef
    pub fn current_level(selector: &str) -> crate::state::score::ScoreRef<'static, i32> {
        SAND_XP_LVL.of(selector)
    }

    /// Returns a [`ScoreRef`] for the player's XP level on the previous tick.
    ///
    /// The objective `__sand_xp_prev` holds the level from the preceding tick.
    ///
    /// [`ScoreRef`]: crate::state::score::ScoreRef
    pub fn previous_level(selector: &str) -> crate::state::score::ScoreRef<'static, i32> {
        SAND_XP_PREV.of(selector)
    }

    /// Returns a [`ScoreRef`] for the level delta this tick (current − previous).
    ///
    /// The objective `__sand_xp_delta` is always ≥ 1 when a handler fires,
    /// since the handler only runs when the delta is positive.
    ///
    /// [`ScoreRef`]: crate::state::score::ScoreRef
    pub fn level_delta(selector: &str) -> crate::state::score::ScoreRef<'static, i32> {
        SAND_XP_DELTA.of(selector)
    }
}

/// Fires when the player's status effects change.
///
/// Maps to `minecraft:effects_changed`.
pub struct EffectsChangedEvent;
impl SandEvent for EffectsChangedEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::effects_changed_any(None))
    }
}

/// Fires when the player starts riding an entity (horse, boat, etc.).
///
/// Maps to `minecraft:started_riding`.
pub struct StartRidingEvent;
impl SandEvent for StartRidingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::StartedRiding)
    }
}

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
        impl crate::event::EventPlayer for $ty {}
    };
}

impl SandEventDispatch {
    /// Extract the advancement trigger from this dispatch, panicking if it's
    /// a tick-condition dispatch.
    fn into_trigger(self) -> Option<crate::AdvancementTrigger> {
        match self {
            SandEventDispatch::AdvancementTrigger(t) => Some(t),
            SandEventDispatch::TickCondition(_) => None,
            SandEventDispatch::Tick(_) => None,
            SandEventDispatch::Chain(_) => None,
        }
    }
}

adv_event!(EntityKillEvent);
adv_event!(PlayerKillEvent);
adv_event!(PlayerDamageEntityEvent);
adv_event!(EntityDamagePlayerEvent);
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
impl crate::event::EventPlayer for PlayerLevelUpEvent {}
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

/// Fires once when a player changes from not sneaking to sneaking.
///
/// This is tick-polled from vanilla's `flags.is_sneaking` entity predicate.
/// The first observed state establishes a baseline and does not fire.
pub struct PlayerStartSneakingEvent;

/// Fires once when a player changes from sneaking to not sneaking.
///
/// Uses the same shared tracker as [`PlayerStartSneakingEvent`].
pub struct PlayerStopSneakingEvent;

/// Shared current-state source used by both sneaking transitions and
/// persistent composition. Kept public only for proc-macro expansion.
#[doc(hidden)]
pub const PLAYER_SNEAKING_TRACKED_SOURCE: crate::TrackedSource =
    crate::TrackedSource::BooleanCondition {
        description: "vanilla entity predicate flags.is_sneaking",
        condition: "predicate __sand_local:__sand/player_sneaking",
    };

/// Fires every tick the player is sneaking / crouching (Shift held).
///
/// Uses a generated `flags.is_sneaking` predicate.
///
/// # Example
/// ```rust,ignore
/// use sand_core::events::PlayerSneakEvent;
/// use sand_core::prelude::*;
/// use sand_macros::event;
///
/// #[event]
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
        PersistentEventCondition::players(crate::condition::Condition::predicate(predicate))
    }
}

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
        PersistentEventCondition::players(crate::condition::Condition::predicate(
            "__sand_local:__sand/player_sprinting",
        ))
    }
}

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
        PersistentEventCondition::players(crate::condition::Condition::predicate(
            "__sand_local:__sand/player_swimming",
        ))
    }
}

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
        PersistentEventCondition::players(crate::condition::Condition::entity(
            "@s[nbt={abilities:{flying:1b}}]",
        ))
    }
}

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
        PersistentEventCondition::players(crate::condition::Condition::predicate(
            "__sand_local:__sand/player_on_fire",
        ))
    }
}

/// Fires every tick the player is in a Creative-mode gamemode.
pub struct PlayerInCreativeEvent;
impl SandEvent for PlayerInCreativeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=creative]".into())
    }
}
impl PersistentSandEvent for PlayerInCreativeEvent {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(crate::condition::Condition::entity(
            "@s[gamemode=creative]",
        ))
    }
}

/// Fires every tick the player is in Adventure mode.
pub struct PlayerInAdventureEvent;
impl SandEvent for PlayerInAdventureEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=adventure]".into())
    }
}
impl PersistentSandEvent for PlayerInAdventureEvent {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(crate::condition::Condition::entity(
            "@s[gamemode=adventure]",
        ))
    }
}

/// Fires every tick the player is in Spectator mode.
pub struct PlayerInSpectatorEvent;
impl SandEvent for PlayerInSpectatorEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=spectator]".into())
    }
}
impl PersistentSandEvent for PlayerInSpectatorEvent {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(crate::condition::Condition::entity(
            "@s[gamemode=spectator]",
        ))
    }
}

// ════════════════════════════════════════════════════════════════════════════
// ── EventPlayer impls for all event types ──────────────────────────────────
// ════════════════════════════════════════════════════════════════════════════
// (Advancement-backed types are covered by the adv_event! macro above.)

macro_rules! player_event {
    ($ty:ty) => {
        impl crate::event::EventPlayer for $ty {}
    };
}

player_event!(OnJoinEvent);
player_event!(FirstJoinEvent);
player_event!(OnDeathEvent);
player_event!(OnRespawnEvent);
player_event!(ArmorEquipEvent);
player_event!(ArmorUnequipEvent);
player_event!(HoldingItemEvent);
player_event!(CurrentlyWearingEvent);
player_event!(PlayerStartSneakingEvent);
player_event!(PlayerStopSneakingEvent);
player_event!(PlayerSneakEvent);
player_event!(PlayerSprintEvent);
player_event!(PlayerSwimmingEvent);
player_event!(PlayerFlyingEvent);
player_event!(PlayerOnFireEvent);
player_event!(PlayerInCreativeEvent);
player_event!(PlayerInAdventureEvent);
player_event!(PlayerInSpectatorEvent);

// ── Doc-coverage registry ────────────────────────────────────────────────────
//
// Every public built-in event type exported from this module must appear in
// this list AND in `book/src/reference/event-trigger-matrix.md`. Workspace
// tests verify matrix coverage. When adding a new public event, append its type
// name here and add a row to the matrix.
//
// `SandEvent` and `SandEventDispatch` are excluded: they are traits/enums,
// not callable event types.
pub const BUILTIN_EVENT_NAMES: &[&str] = &[
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
    "PlayerSwimmingEvent",
    "PlayerFlyingEvent",
    "PlayerOnFireEvent",
    "PlayerInCreativeEvent",
    "PlayerInAdventureEvent",
    "PlayerInSpectatorEvent",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::AdvancementEvent;

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
