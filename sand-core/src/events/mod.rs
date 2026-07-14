//! Built-in Sand event types and the legacy [`SandEvent`] trait for custom
//! tick-poll or compatibility events.
//!
//! New custom advancement-backed events should implement
//! [`AdvancementEvent`](crate::event::AdvancementEvent) and use
//! [`Event<T>`](crate::event::Event) as the handler parameter:
//!
//! ```rust,ignore
//! use sand_core::prelude::*;
//! use sand_core::event::trigger::ConsumeItemTrigger;
//! use sand_components::ItemPredicate;
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
//! use sand_components::ItemPredicate;
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
//! [`SandEvent::setup`]), and generic event families with distinct, stable
//! per-monomorphization identity. Implement [`AdvancementEvent`](crate::event::AdvancementEvent)
//! instead when your event maps to exactly one vanilla advancement trigger and
//! needs no owned lifecycle — that is the lighter-weight, common case.
//!
//! ```rust,ignore
//! use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
//! use sand_core::prelude::*;
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
//!                 "scoreboard players operation @a sync_jumps = @a jumps".into(),
//!             ],
//!         }
//!     }
//! }
//!
//! #[event]
//! pub fn on_jump(event: PlayerJumpEvent) {
//!     cmd::say("Jumped!");
//! }
//! ```
//!
//! Simple advancement-backed or single-fragment tick-poll `SandEvent` impls
//! remain supported via [`SandEventDispatch::AdvancementTrigger`] and
//! [`SandEventDispatch::TickCondition`] — both lower into the same normalized
//! IR as [`SandEventDispatch::tick()`] (see [`SandEventDispatch::normalize`]).

// ── Custom event API ──────────────────────────────────────────────────────────

/// Execution scope for a structured [`TickEventDispatch`].
///
/// Currently only per-player polling is supported; more scopes (e.g. arbitrary
/// entity queries) are a natural extension point for #240-style composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TickScope {
    /// Evaluated as each online player (`execute as @a ... at @s run ...`).
    #[default]
    Players,
}

/// Lifecycle resources a [`SandEvent`] owns: objectives to create at load time,
/// commands to run before each observation, and commands to run after a
/// successful observation (e.g. synchronizing a delta-tracking score).
///
/// Returned by [`SandEvent::setup`]. When multiple `#[event]` handlers
/// subscribe to the same event type, Sand deduplicates the setup so
/// objectives and detector/synchronization functions are emitted once.
#[derive(Debug, Clone, Default)]
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

/// Structured, typed tick-poll dispatch definition.
///
/// Built via [`SandEventDispatch::tick`]. Conditions are composed from the
/// same [`Condition`](crate::condition::Condition) IR used throughout Sand
/// (score comparisons, flags, predicates, entity checks, and the explicit
/// [`Condition::raw`] escape hatch) rather than hand-formatted strings.
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
#[derive(Debug, Clone, Default)]
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
    /// or `None` if no conditions were declared.
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

    /// Render this dispatch's combined condition to a single `if/unless …`
    /// clause-list fragment (without the leading `execute` keyword), suitable
    /// for splicing into an `execute <clauses> at @s run …` command.
    ///
    /// Returns `None` if no conditions were declared, or if the combined
    /// condition expands into more than one OR-alternative execute plan
    /// (i.e. a top-level `Any`/`.unless` combination that cannot collapse
    /// into a single chained clause list) — that case is not yet supported by
    /// the tick-dispatch lifecycle codegen.
    pub fn render_clauses(&self) -> Option<String> {
        let combined = self.combined_condition()?;
        let plans = combined.to_execute_plans(false);
        if plans.len() != 1 {
            return None;
        }
        Some(plans[0].join(" "))
    }
}

impl From<TickEventDispatch> for SandEventDispatch {
    fn from(tick: TickEventDispatch) -> Self {
        SandEventDispatch::Tick(tick)
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
}

/// Normalized internal representation of a [`SandEventDispatch`], used by the
/// export pipeline and by tests asserting on lowering behavior.
///
/// Every `SandEventDispatch` variant — including the legacy `AdvancementTrigger`
/// and `TickCondition` compatibility constructors — lowers into one of these
/// two shapes, so the exporter has a single normalized IR to consume rather
/// than juggling multiple representations.
#[allow(clippy::large_enum_variant)]
pub enum NormalizedEventDispatch {
    /// Advancement-backed dispatch.
    Advancement(crate::AdvancementTrigger),
    /// Tick-poll dispatch, always in the structured [`TickEventDispatch`] shape.
    Tick(TickEventDispatch),
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

    /// Lower this dispatch into the normalized internal IR.
    ///
    /// - `AdvancementTrigger(t)` → `Advancement(t)` unchanged.
    /// - `TickCondition(s)` → `Tick(...)` with `s` carried as a single
    ///   [`Condition::raw`](crate::condition::Condition::raw) `when` clause.
    /// - `Tick(t)` → `Tick(t)` unchanged.
    pub fn normalize(self) -> NormalizedEventDispatch {
        match self {
            SandEventDispatch::AdvancementTrigger(t) => NormalizedEventDispatch::Advancement(t),
            SandEventDispatch::TickCondition(s) => NormalizedEventDispatch::Tick(
                TickEventDispatch::default().when(crate::condition::Condition::raw(s)),
            ),
            SandEventDispatch::Tick(t) => NormalizedEventDispatch::Tick(t),
        }
    }
}

/// Implement this trait on your own type to define a custom Sand event.
///
/// Your type is used as the phantom parameter in an `#[event]` handler
/// function. Sand inspects [`dispatch`](Self::dispatch) at build time to
/// emit the appropriate datapack files.
///
/// # Example
///
/// ```rust,ignore
/// use sand_core::events::{SandEvent, SandEventDispatch};
/// use sand_core::prelude::*;
/// use sand_core::AdvancementTrigger;
///
/// /// Fires when a player picks up any item.
/// pub struct ItemPickupEvent;
///
/// impl SandEvent for ItemPickupEvent {
///     fn dispatch() -> SandEventDispatch {
///         SandEventDispatch::AdvancementTrigger(
///             AdvancementTrigger::PickedUpItem { item: None }
///         )
///     }
/// }
///
/// #[event]
/// pub fn on_item_pickup(event: ItemPickupEvent) {
///     cmd::say("Picked something up!");
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
    /// Sand deduplicates setup by the event's generated identity so
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

/// Fires when the player crafts any item.
///
/// Maps to `minecraft:crafted_item`.
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

/// Fires when the player empties any bucket. (Added in MC 1.17.)
///
/// Maps to `minecraft:emptied_bucket`.
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

/// Fires when a thrown item is picked up by any entity.
///
/// Maps to `minecraft:thrown_item_picked_up`.
pub struct ItemPickedUpEvent;
impl SandEvent for ItemPickedUpEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ThrownItemPickedUp {
            item: None,
            entity: None,
        })
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

/// Fires every tick the player is sneaking / crouching (Shift held).
///
/// Uses a generated `flags.is_sneaking` predicate.
///
/// # Example
/// ```rust,ignore
/// #[event]
/// pub fn while_sneaking(event: PlayerSneakEvent) {
///     cmd::particle(Particle::Smoke, event.player());
/// }
/// ```
pub struct PlayerSneakEvent;
impl SandEvent for PlayerSneakEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("predicate __sand_local:__sand/player_sneaking".into())
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

/// Fires every tick the player is swimming (swimming animation active, 1.13+).
///
/// Uses a generated `flags.is_swimming` predicate.
pub struct PlayerSwimmingEvent;
impl SandEvent for PlayerSwimmingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("predicate __sand_local:__sand/player_swimming".into())
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

/// Fires every tick the player is on fire.
///
/// Uses a generated `flags.is_on_fire` predicate.
pub struct PlayerOnFireEvent;
impl SandEvent for PlayerOnFireEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("predicate __sand_local:__sand/player_on_fire".into())
    }
}

/// Fires every tick the player is in a Creative-mode gamemode.
pub struct PlayerInCreativeEvent;
impl SandEvent for PlayerInCreativeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=creative]".into())
    }
}

/// Fires every tick the player is in Adventure mode.
pub struct PlayerInAdventureEvent;
impl SandEvent for PlayerInAdventureEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=adventure]".into())
    }
}

/// Fires every tick the player is in Spectator mode.
pub struct PlayerInSpectatorEvent;
impl SandEvent for PlayerInSpectatorEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=spectator]".into())
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
    fn tick_dispatch_when_renders_single_clause() {
        let d = SandEventDispatch::tick()
            .as_players()
            .when(crate::condition::Condition::raw(
                "score @s sync_jumps < @s jumps",
            ));
        assert_eq!(
            d.render_clauses().as_deref(),
            Some("if score @s sync_jumps < @s jumps")
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
        let rendered = d.render_clauses().unwrap();
        assert!(rendered.contains("if score @s sync_jumps < @s jumps"));
        assert!(rendered.contains("unless score @s is_dead matches 1"));
        // `when` clause must precede `unless` clause.
        assert!(
            rendered.find("if score @s sync_jumps").unwrap()
                < rendered.find("unless score @s is_dead").unwrap()
        );
    }

    #[test]
    fn tick_dispatch_if_alias_matches_when() {
        let a = SandEventDispatch::tick()
            .if_(crate::condition::Condition::raw("score @s a matches 1"))
            .render_clauses();
        let b = SandEventDispatch::tick()
            .when(crate::condition::Condition::raw("score @s a matches 1"))
            .render_clauses();
        assert_eq!(a, b);
    }

    #[test]
    fn tick_dispatch_no_conditions_renders_none() {
        let d = SandEventDispatch::tick().as_players();
        assert_eq!(d.render_clauses(), None);
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
                    t.render_clauses().as_deref(),
                    Some("if score @s sync_jumps < @s jumps")
                );
            }
            NormalizedEventDispatch::Advancement(_) => panic!("expected Tick"),
        }
    }

    #[test]
    fn legacy_tick_condition_normalizes_to_structured_tick() {
        let dispatch = SandEventDispatch::TickCondition("entity @s[tag=ready]".into());
        match dispatch.normalize() {
            NormalizedEventDispatch::Tick(t) => {
                assert_eq!(
                    t.render_clauses().as_deref(),
                    Some("if entity @s[tag=ready]")
                );
            }
            NormalizedEventDispatch::Advancement(_) => panic!("expected Tick"),
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
        }
    }

    #[test]
    fn event_setup_default_is_empty() {
        let setup = EventSetup::none();
        assert!(setup.objectives.is_empty());
        assert!(setup.pre_observation.is_empty());
        assert!(setup.post_observation.is_empty());
    }
}
