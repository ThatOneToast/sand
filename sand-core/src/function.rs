/// Descriptor for a datapack function registered via `#[sand_macros::function]`.
///
/// All descriptors submitted with [`inventory::submit!`] are collected at
/// program startup and iterable via [`inventory::iter::<FunctionDescriptor>`].
///
/// # Fields
/// - `path` — the resource location *path* component (e.g. `"hello_world"`,
///   `"utils/tick"`). The namespace is applied by the caller at build time.
/// - `make` — a zero-argument factory function that returns the list of
///   command strings for this function. Using a factory enables both static
///   string literals and dynamic [`crate::Command`] builder values.
pub struct FunctionDescriptor {
    pub path: &'static str,
    pub make: fn() -> Vec<String>,
}

inventory::collect!(FunctionDescriptor);

/// Side table mapping `fn() -> Vec<String>` pointers to their registered
/// resource location path (namespace:path or bare path).
///
/// Automatically populated by `#[sand_macros::function]`. The path stored
/// is the full `"ns:path"` if given explicitly, or just the path component
/// for bare `#[function]` functions.
pub struct FunctionPointerEntry {
    /// The function pointer to match against.
    pub ptr: fn() -> Vec<String>,
    /// The resource location path as specified in the attribute,
    /// e.g. `"powers:ate_golden_apple"` or `"my_function"`.
    pub path: &'static str,
}
inventory::collect!(FunctionPointerEntry);

/// Side table mapping the unique type of a Rust function item to its registered
/// resource location path.
///
/// Rust function items do not automatically satisfy trait impls for bare fn
/// pointers in generic parameters, so this lets `cmd::call(local_function)`
/// resolve without requiring `as fn() -> Vec<String>` casts.
pub struct FunctionPointerTypeEntry {
    /// Returns the unique [`TypeId`](std::any::TypeId) of the function item.
    pub type_id: fn() -> std::any::TypeId,
    /// The resource location path as specified in the attribute.
    pub path: &'static str,
}
inventory::collect!(FunctionPointerTypeEntry);

/// Trait for types that can be resolved to a `function <id>` command string.
///
/// This enables `cmd::call(...)` to accept local function pointers,
/// [`FunctionRef`](crate::resource_ref::FunctionRef) values, [`ResourceLocation`](crate::ResourceLocation) values, and raw path strings.
///
/// # Implementors
///
/// | Type | Resolution |
/// |---|---|
/// | [`FunctionRef`](crate::resource_ref::FunctionRef) | Uses the ref's `Display` → `"function namespace:path"` |
/// | `&FunctionRef` | Same as above |
/// | [`ResourceLocation`](crate::ResourceLocation) | Uses the location's `Display` → `"function namespace:path"` |
/// | `&str` | Used as-is → `"function raw_path"` |
/// | `String` | Used as-is → `"function raw_path"` |
/// | `fn() -> Vec<String>` or function item | Looks up the registered path from `#[function]` inventory |
///
/// # Errors
///
/// An unregistered `fn() -> Vec<String>`  (not annotated with `#[function]`)
/// will panic with a clear message.
pub trait IntoFunctionRef {
    /// Resolve to a complete `function <id>` Minecraft command string.
    fn into_function_command(self) -> String;

    /// Resolve to just the `namespace:path` resource location string.
    fn into_function_id(self) -> String;
}

impl IntoFunctionRef for crate::resource_ref::FunctionRef {
    fn into_function_command(self) -> String {
        format!("function {self}")
    }
    fn into_function_id(self) -> String {
        self.to_string()
    }
}

impl IntoFunctionRef for &crate::resource_ref::FunctionRef {
    fn into_function_command(self) -> String {
        format!("function {self}")
    }
    fn into_function_id(self) -> String {
        self.to_string()
    }
}

impl IntoFunctionRef for crate::ResourceLocation {
    fn into_function_command(self) -> String {
        format!("function {self}")
    }
    fn into_function_id(self) -> String {
        self.to_string()
    }
}

impl IntoFunctionRef for &str {
    fn into_function_command(self) -> String {
        format!("function {self}")
    }
    fn into_function_id(self) -> String {
        self.to_string()
    }
}

impl IntoFunctionRef for String {
    fn into_function_command(self) -> String {
        format!("function {self}")
    }
    fn into_function_id(self) -> String {
        self.to_string()
    }
}

/// Sentinel namespace emitted for local function pointers whose namespace is
/// not yet known at compile time.  The export pipeline in
/// [`crate::component::export_components_json`] replaces this with the actual
/// pack namespace from `sand.toml`.
pub const SAND_LOCAL_NS: &str = "__sand_local";

fn command_for_path(path: &str) -> String {
    if path.contains(':') {
        format!("function {path}")
    } else {
        format!("function {SAND_LOCAL_NS}:{path}")
    }
}

fn id_for_path(path: &str) -> String {
    if path.contains(':') {
        path.to_string()
    } else {
        format!("{SAND_LOCAL_NS}:{path}")
    }
}

fn registered_path_for_function_value<F>(value: F) -> Option<&'static str>
where
    F: Copy + 'static,
{
    let type_id = std::any::TypeId::of::<F>();
    for entry in inventory::iter::<FunctionPointerTypeEntry>() {
        if (entry.type_id)() == type_id {
            return Some(entry.path);
        }
    }

    if std::mem::size_of::<F>() == std::mem::size_of::<fn() -> Vec<String>>() {
        let ptr = unsafe { *(&value as *const F).cast::<fn() -> Vec<String>>() };
        for entry in inventory::iter::<FunctionPointerEntry>() {
            if entry.ptr as usize == ptr as usize {
                return Some(entry.path);
            }
        }
    }

    None
}

impl<F> IntoFunctionRef for F
where
    F: Fn() -> Vec<String> + Copy + 'static,
{
    fn into_function_command(self) -> String {
        if let Some(path) = registered_path_for_function_value(self) {
            return command_for_path(path);
        }
        panic!(
            "unregistered function pointer: the function must be annotated with \
             #[function] or #[function(\"path\")] to be callable via cmd::call()"
        )
    }
    fn into_function_id(self) -> String {
        if let Some(path) = registered_path_for_function_value(self) {
            return id_for_path(path);
        }
        panic!(
            "unregistered function pointer: the function must be annotated with \
             #[function] or #[function(\"path\")] to be callable via cmd::call()"
        )
    }
}

/// Maps an event-type's [`TypeId`](std::any::TypeId) to the handler function
/// path registered by `#[event]`.
///
/// Used by [`crate::event::handle::EventHandle`] to derive advancement IDs for
/// `revoke()` and `grant()` without requiring a string argument.
///
/// Populated automatically by the `#[event]` macro for advancement-backed
/// events (`dispatch = "advancement"`).  Not emitted for tick-poll events.
pub struct EventPathEntry {
    pub type_id: std::any::TypeId,
    /// The handler function path component, e.g. `"on_ate_golden_apple"`.
    pub path: &'static str,
}
inventory::collect!(EventPathEntry);

/// Registry entry for a `#[component]`-annotated function.
///
/// The `make` fn pointer is a zero-argument function that constructs the
/// component and boxes it as a trait object. Registered at link time via
/// `inventory::submit!` — no user wiring needed.
pub struct ComponentFactory {
    /// Factory function that returns a boxed datapack component.
    pub make: fn() -> Box<dyn crate::DatapackComponent>,
}
inventory::collect!(ComponentFactory);

/// Registers a function as an entry in a Minecraft function tag.
///
/// Produced by `#[component(Tick)]`, `#[component(Load)]`, and
/// `#[component(Tag = "ns:name")]`. During `sand build` all descriptors for
/// the same `tag` are merged into a single tag JSON file:
/// entries keep first registration/export order, and duplicate function refs
/// are emitted only once.
///
/// | Variant | `tag` value | Output file |
/// |---|---|---|
/// | `Tick` | `"minecraft:tick"` | `data/minecraft/tags/function/tick.json` |
/// | `Load` | `"minecraft:load"` | `data/minecraft/tags/function/load.json` |
/// | `Tag = "ns:name"` | `"ns:name"` | `data/ns/tags/function/name.json` |
pub struct FunctionTagDescriptor {
    /// Full tag resource location, e.g. `"minecraft:tick"`.
    pub tag: &'static str,
    /// Function path component (namespace applied at build time), e.g. `"my_tick"`.
    pub function_path: &'static str,
}
inventory::collect!(FunctionTagDescriptor);

/// How a Sand event is dispatched at runtime.
///
/// Sand inspects this at build time and generates the appropriate mcfunction
/// wiring for each variant. Multiple events of the same variant type are
/// batched into a single aggregator function.
pub enum EventDispatch {
    /// Advancement-backed event.
    ///
    /// Sand generates an advancement JSON and the handler function is its
    /// reward. When `revoke` is `true`, `advancement revoke @s only <id>`
    /// is inserted at the top of the handler so it can fire again next time
    /// the trigger is met.
    Advancement {
        make_trigger: fn() -> crate::AdvancementTrigger,
        /// Returns `true` to revoke the advancement after firing, `false` for once-only.
        revoke: fn() -> bool,
        /// Optional typed guard condition.
        ///
        /// When `Some`, the entry function prepends one or more
        /// `execute unless <clause> run return 0` lines generated from the
        /// [`Condition`](crate::condition::Condition) via `execute_commands(true, "return 0")`.
        /// This correctly handles `Any` (OR) conditions as multiple guard lines.
        guard: Option<fn() -> Option<crate::condition::Condition>>,
    },

    /// All-deaths detection via the `deathCount` scoreboard criterion.
    ///
    /// Fires for every player death (mob, fall, fire, void, `/kill`, …).
    /// Sand generates a `__sand_death_check` tick function and a
    /// `__sand_death_init` load function (adds the `deathCount` objective).
    DeathTick,

    /// Fires on the first tick after a server start, `/reload`, or when a new
    /// player joins mid-session.
    ///
    /// Sand generates a `__sand_join_init` load function that creates the
    /// `__sand_join` scoreboard objective and resets all scores on every load,
    /// plus a `__sand_join_check` tick function that runs all handlers for
    /// any online player whose `__sand_join` score is not 1, then sets the
    /// score to 1.
    ///
    /// **Vanilla limitation:** mid-session disconnect → reconnect without a
    /// `/reload` does **not** re-fire because the player's score persists in
    /// `scoreboard.dat`. True per-login detection requires a mod or plugin.
    JoinTick,

    /// Fires on the first Sand tick that observes a player active after an
    /// observed death.
    ///
    /// Sand keeps a per-player phase score and gates completion on vanilla's
    /// `minecraft.custom:minecraft.time_since_death` statistic becoming
    /// positive after death reset it to zero. The respawn check runs before
    /// new deaths inside one generated coordinator, so one death observation
    /// cannot dispatch both `DeathTick` and `RespawnTick` in the same cycle.
    /// This is a tick-boundary observation, not the client respawn packet.
    RespawnTick,

    /// Tick-polled condition — fires every tick the condition is true.
    ///
    /// Sand generates a `__sand_tick_check` function that runs
    /// `execute as @a if <condition> at @s run function ns:path` once per
    /// handler, registered to `minecraft:tick`.
    ///
    /// `make_condition` should return a valid Minecraft `execute if`
    /// sub-command, e.g. `"items entity @s mainhand minecraft:diamond_sword"`.
    TickPoll { make_condition: fn() -> String },

    /// Fires when a player equips an item in an equipment slot.
    ///
    /// Sand uses a per-player entity tag to track previous slot state and
    /// detect equip transitions each tick.
    ArmorEquip {
        slot: ArmorSlot,
        /// Item ID filter — `None` matches any item.
        item_id: Option<&'static str>,
        /// SNBT for `minecraft:custom_data` matching, e.g. `"{my_item:1b}"`.
        custom_data_snbt: Option<&'static str>,
    },

    /// Fires when a player removes an item from an equipment slot.
    ///
    /// Same detection mechanism as [`ArmorEquip`](EventDispatch::ArmorEquip).
    ArmorUnequip {
        slot: ArmorSlot,
        item_id: Option<&'static str>,
        custom_data_snbt: Option<&'static str>,
    },

    /// Custom event dispatch for types implementing [`crate::events::SandEvent`].
    ///
    /// At build time, Sand calls `make_trigger()`, `make_condition()`,
    /// `make_tick()`, `make_chain()`, and `make_tracked()` to determine which
    /// dispatch path to use. Exactly one must return `Some`; the export
    /// pipeline rejects the export with a diagnostic naming the handler path
    /// and all five factory methods if zero or more than one return `Some`
    /// (#121).
    ///
    /// - `Some` from `make_trigger` only → advancement-backed dispatch
    /// - `Some` from `make_condition` only → tick-poll dispatch
    /// - `Some` from `make_tick` only → structured tick-lifecycle dispatch
    /// - `Some` from `make_chain` only → same-cycle chained dispatch (#240)
    /// - `Some` from `make_tracked` only → reusable tracked transition (#49)
    /// - zero or more than one `Some` → rejected at export time
    Custom {
        /// Returns `Some(AdvancementTrigger)` when using legacy `AdvancementTrigger` dispatch.
        make_trigger: fn() -> Option<crate::AdvancementTrigger>,
        /// Returns `Some(condition_string)` when using legacy `TickCondition` dispatch.
        make_condition: fn() -> Option<String>,
        /// Returns `Some(TickEventDispatch)` when using the structured, typed
        /// `SandEventDispatch::tick()` builder. Mutually exclusive with the
        /// other factories — exactly one returns `Some`.
        make_tick: fn() -> Option<crate::events::TickEventDispatch>,
        /// Returns `Some(ChainEventDispatch)` when using
        /// `SandEventDispatch::chain::<Parent>()`. Mutually exclusive with
        /// the other factories — exactly one returns `Some`.
        make_chain: fn() -> Option<crate::events::ChainEventDispatch>,
        /// Returns `Some(TrackedTransition)` when using
        /// `SandEventDispatch::Tracked(...)`. Mutually exclusive with the
        /// other factories — exactly one returns `Some`. Generic `SandEvent`
        /// types (e.g. `EffectStarted<Speed>`) reach the shared transition
        /// provider backend through this factory rather than macro-level
        /// name matching, so distinct monomorphizations dispatch correctly.
        make_tracked: fn() -> Option<TrackedTransition>,
        /// Whether to revoke the advancement after firing (advancement dispatch only).
        revoke: fn() -> bool,
        /// In-process grouping identity of the `SandEvent` type this handler
        /// subscribes to.
        ///
        /// `TypeId` is appropriate for distinguishing concrete types —
        /// including distinct generic monomorphizations such as
        /// `ElevatorUsed<GoUp>` vs `ElevatorUsed<GoDown>` — **within the
        /// current export process**, and is used to group/deduplicate setup
        /// and detector evaluation across multiple handlers of the same
        /// event. It is **not** a stable identifier across compiler versions
        /// or builds and must never be used to derive generated resource
        /// paths — see the sibling `event_type_name` field for that.
        event_type_id: fn() -> std::any::TypeId,
        /// Canonical concrete type name (`std::any::type_name::<T>()`) of the
        /// `SandEvent` type this handler subscribes to.
        ///
        /// Used as the input to a deterministic resource-key derivation for
        /// generated detector/setup/dispatch function paths, so the same
        /// concrete event type always produces the same generated paths
        /// regardless of how many handlers subscribe to it, their
        /// registration order, or inventory/link order. Distinct generic
        /// monomorphizations produce distinct canonical names and therefore
        /// distinct keys.
        event_type_name: fn() -> &'static str,
        /// Returns the event's lifecycle setup (objectives, pre/post-observation
        /// commands). Only meaningful for tick-poll dispatch; ignored for
        /// advancement dispatch.
        make_setup: fn() -> crate::events::EventSetup,
    },

    /// Tick-backed XP level-up detection.
    ///
    /// Sand generates four Sand-owned scoreboard objectives and a dedicated
    /// `__sand_xp_check` tick function. Handlers are called when a player's XP
    /// level increases. The first tick after load/join initialises previous-level
    /// state without firing. Level decreases do not fire the event.
    ///
    /// Objectives used (all ≤16 chars):
    /// - `__sand_xp_lvl`   — current XP level (refreshed every tick)
    /// - `__sand_xp_prev`  — previous XP level (last tick)
    /// - `__sand_xp_delta` — current − previous
    /// - `__sand_xp_seen`  — 0 until first tick; prevents spurious fire on join
    XpLevelUp,

    /// Reusable tick-polled previous/current transition detection.
    ///
    /// Handlers that name the same tracker and source share one generated
    /// baseline and tracker function. Setup and ticking are contributed to the
    /// automatic state lifecycle rather than a separate tick registry.
    Tracked(TrackedTransition),
}

/// The value sampled by a tracked transition event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackedSource {
    /// A vanilla `execute if` condition, without the leading `if` keyword.
    BooleanCondition {
        description: &'static str,
        condition: &'static str,
    },
    /// A per-player scoreboard value sampled from `@s`.
    ///
    /// `criterion` is the vanilla scoreboard criterion (e.g. `"health"`) used
    /// to auto-declare `objective` at load time — trackers never assume the
    /// objective already exists.
    Score {
        description: &'static str,
        objective: &'static str,
        criterion: &'static str,
    },
    /// A boolean threshold crossing derived from a per-player scoreboard
    /// value, e.g. "health at or below N".
    ///
    /// Reuses the same boolean transition machinery as
    /// [`TrackedSource::BooleanCondition`] — the comparison is rendered into
    /// an equivalent `score @s <objective> matches <range>` condition at
    /// export time — but keeps the numeric threshold as a plain value
    /// instead of requiring a pre-formatted `&'static str`, so const-generic
    /// or otherwise parameterized threshold events can be expressed without
    /// baking the number into a static string at the call site.
    ScoreThreshold {
        description: &'static str,
        objective: &'static str,
        criterion: &'static str,
        comparator: ScoreThresholdComparator,
    },
}

/// A one-sided integer comparison used by [`TrackedSource::ScoreThreshold`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreThresholdComparator {
    /// `score @s <objective> matches ..N` (value <= N).
    AtOrBelow(i32),
    /// `score @s <objective> matches N..` (value >= N).
    AtOrAbove(i32),
}

impl ScoreThresholdComparator {
    pub(crate) fn render(self, objective: &str) -> String {
        match self {
            Self::AtOrBelow(n) => format!("score @s {objective} matches ..{n}"),
            Self::AtOrAbove(n) => format!("score @s {objective} matches {n}.."),
        }
    }
}

impl TrackedSource {
    pub const fn description(self) -> &'static str {
        match self {
            Self::BooleanCondition { description, .. }
            | Self::Score { description, .. }
            | Self::ScoreThreshold { description, .. } => description,
        }
    }
}

/// Transition comparison applied to a tracked source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransitionKind {
    BecameTrue,
    BecameFalse,
    ScoreChanged,
    ScoreIncreased,
    ScoreDecreased,
}

impl TransitionKind {
    /// Pure comparison model used by tests and non-export tooling.
    pub const fn matches(self, previous: i32, current: i32, seen: bool) -> bool {
        if !seen {
            return false;
        }
        match self {
            Self::BecameTrue => previous == 0 && current != 0,
            Self::BecameFalse => previous != 0 && current == 0,
            Self::ScoreChanged => previous != current,
            Self::ScoreIncreased => current > previous,
            Self::ScoreDecreased => current < previous,
        }
    }
}

/// Immutable transition tracker attached to an [`EventDescriptor`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrackedTransition {
    pub tracker_id: &'static str,
    pub source: TrackedSource,
    pub kind: TransitionKind,
}

impl TrackedTransition {
    pub const fn new(
        tracker_id: &'static str,
        source: TrackedSource,
        kind: TransitionKind,
    ) -> Self {
        Self {
            tracker_id,
            source,
            kind,
        }
    }
}

/// Descriptor for a function registered via `#[sand_macros::event]`.
///
/// Collected via [`inventory::iter::<EventDescriptor>`] at export time.
///
/// # Fields
/// - `path` — function resource location path (no namespace), e.g. `"on_join"`
/// - `id_override` — optional full advancement ID override (advancement dispatch only)
/// - `make` — factory that returns the `Vec<String>` of mcfunction commands
/// - `dispatch` — advancement, special generated system, tick poll, or tracked transition
pub struct EventDescriptor {
    pub path: &'static str,
    pub id_override: Option<&'static str>,
    pub make: fn() -> Vec<String>,
    pub dispatch: EventDispatch,
}
inventory::collect!(EventDescriptor);

/// Descriptor for a function registered via `#[sand_macros::schedule]`.
///
/// Produces three `.mcfunction` files and injects tick/load handlers:
/// - `<path>.mcfunction` — the body called each interval
/// - `<path>_start.mcfunction` — initialises the scoreboard counters for `@s`
/// - `<path>_stop.mcfunction`  — cancels an active schedule for `@s`
///
/// # Scoreboard objectives (auto-created on load)
/// Objective names are derived from a stable FNV-1a hash of `path` so they
/// always fit within Minecraft's 16-character limit:
/// - `__ss_<hash>_t` (`dummy`) — ticks remaining; `0` = inactive
/// - `__ss_<hash>_p` (`dummy`) — phase countdown between executions (only
///   created when `every > 1`)
///
/// The generated tick function evaluates and mutates these objectives once per
/// active player under `execute as ...`, with the owner bound to `@s`.
///
/// # Usage
/// Start / stop the schedule at runtime by calling the generated functions:
/// ```mcfunction
/// # From another .mcfunction — run for the executing entity (@s)
/// function mypack:my_effect_start
/// function mypack:my_effect_stop
/// ```
pub struct ScheduleDescriptor {
    /// Resource location path (no namespace), e.g. `"my_effect"`.
    pub path: &'static str,
    /// Total duration in ticks. Counts down every tick regardless of `every`.
    pub total_ticks: u32,
    /// Execute the body every N ticks. `1` = every tick (default).
    pub every: u32,
    /// Factory that returns the command strings for the body function.
    pub make: fn() -> Vec<String>,
}
inventory::collect!(ScheduleDescriptor);

/// Which inventory slot to watch for [`ArmorEventDescriptor`] events.
///
/// Slot IDs match Minecraft's NBT slot bytes:
/// `Head=103b`, `Chest=102b`, `Legs=101b`, `Feet=100b`, `Offhand=-106b`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlot {
    /// Helmet slot. NBT: `Slot:103b`
    Head,
    /// Chestplate slot. NBT: `Slot:102b`
    Chest,
    /// Leggings slot. NBT: `Slot:101b`
    Legs,
    /// Boots slot. NBT: `Slot:100b`
    Feet,
    /// Offhand slot. NBT: `Slot:-106b`
    Offhand,
}

impl ArmorSlot {
    /// Get the NBT slot byte for this armor slot.
    pub fn slot_byte(self) -> i8 {
        match self {
            ArmorSlot::Head => 103,
            ArmorSlot::Chest => 102,
            ArmorSlot::Legs => 101,
            ArmorSlot::Feet => 100,
            ArmorSlot::Offhand => -106,
        }
    }

    /// Get the tag name segment for this armor slot (used in entity tag names).
    pub fn tag_name_segment(self) -> &'static str {
        match self {
            ArmorSlot::Head => "head",
            ArmorSlot::Chest => "chest",
            ArmorSlot::Legs => "legs",
            ArmorSlot::Feet => "feet",
            ArmorSlot::Offhand => "offhand",
        }
    }

    /// Slot name for `execute if items entity @s <slot>`.
    pub fn slot_name(self) -> &'static str {
        match self {
            ArmorSlot::Head => "armor.head",
            ArmorSlot::Chest => "armor.chest",
            ArmorSlot::Legs => "armor.legs",
            ArmorSlot::Feet => "armor.feet",
            ArmorSlot::Offhand => "weapon.offhand",
        }
    }
}

/// Whether the event fires on equip or unequip.
#[derive(Clone, Copy)]
pub enum ArmorEventKind {
    /// Fires on the tick the item appears in the watched slot.
    Equip,
    /// Fires on the tick the item is removed from the watched slot.
    Unequip,
}

/// Descriptor for `#[sand_macros::armor_event]` annotated functions.
///
/// At export time, all descriptors are combined into a single
/// `__sand_armor_check` mcfunction registered to `minecraft:tick`.
pub struct ArmorEventDescriptor {
    /// Function path (no namespace), e.g. `"on_boots_equip"`.
    pub path: &'static str,
    /// Factory that returns the mcfunction commands.
    pub make: fn() -> Vec<String>,
    /// Which slot to watch.
    pub slot: ArmorSlot,
    /// Equip or Unequip.
    pub kind: ArmorEventKind,
    /// Item ID filter, e.g. `"minecraft:leather_boots"`. `None` = any item.
    pub item_id: Option<&'static str>,
    /// SNBT for `minecraft:custom_data` matching, e.g. `"{mana_boots:true}"`.
    /// Generates: `components:{"minecraft:custom_data":<snbt>}` in the NBT selector.
    pub custom_data_snbt: Option<&'static str>,
}
inventory::collect!(ArmorEventDescriptor);

/// A temporary scoreboard objective automatically created on load.
///
/// Register with [`temp_score!`](crate::temp_score) and Sand will emit
/// `scoreboard objectives add <name> <criteria>` in the generated init
/// function — no manual load-function wiring needed.
///
/// # Example
/// ```rust,ignore
/// temp_score!(player_hp_tmp);           // dummy criterion
/// temp_score!(kill_count, "playerKillCount");
/// ```
pub struct TempScoreboard {
    /// The objective name (≤16 chars recommended).
    pub name: &'static str,
    /// Scoreboard criterion, e.g. `"dummy"`, `"playerKillCount"`.
    pub criteria: &'static str,
    /// Optional display name shown in the sidebar/tab list.
    pub display_name: Option<&'static str>,
}
inventory::collect!(TempScoreboard);

// ── Dynamic anonymous function registry ───────────────────────────────────────
//
// This registry is **thread-local**, not process-global. Every export
// (`try_export_components_json` and friends) runs as one synchronous call
// tree on a single thread — `SandEvent::setup()`, condition lowering,
// relation-query helpers, item-snapshot capture, and participant
// observation all register into and drain from the *same thread's* view of
// this registry, with no cross-thread interference.
//
// This matters because Rust's default test harness runs many `#[test]`
// functions from one binary **concurrently on separate threads**. A
// process-global `Mutex<Vec<..>>` registry (the previous design) meant any
// two tests that happened to register/drain dynamic functions at
// overlapping moments could corrupt each other's view of the registry —
// this was the root cause of the nondeterminism `LIM-EXP-006` documented
// as a workaround rather than a fix: two `try_export_components_json`
// calls in the *same test file* could race against a concurrently-running
// `#[test]` fn in that file, not against each other. A thread-local
// registry removes the race entirely: each thread's export sees only its
// own registrations, so a call to `drain_dyn_fns()` on one thread can never
// observe (or discard) another thread's still-in-progress registrations.
//
// Production callers are unaffected either way — Sand's own export
// pipeline is not itself multi-threaded internally.

use std::cell::RefCell;

type DynFnEntry = (String, Vec<String>);

thread_local! {
    static REGISTRY: RefCell<Vec<DynFnEntry>> = const { RefCell::new(Vec::new()) };
}

/// Register an anonymous function body at runtime.
///
/// Called by anonymous `run_fn!` blocks that capture local variables.
/// The `commands` are the pre-computed mcfunction lines.
pub fn register_dyn_fn(path: String, commands: Vec<String>) {
    REGISTRY.with_borrow_mut(|registry| {
        if !registry.iter().any(|(existing_path, existing_commands)| {
            existing_path == &path && existing_commands == &commands
        }) {
            registry.push((path, commands));
        }
    });
}

/// Register a generated helper function, reusing an existing helper with an
/// identical body when context semantics allow it.
pub fn register_dyn_fn_dedup(prefix: &str, commands: Vec<String>) -> String {
    REGISTRY.with_borrow_mut(|registry| {
        if let Some((path, _)) = registry.iter().find(|(path, existing_commands)| {
            path.starts_with(prefix) && existing_commands == &commands
        }) {
            return path.clone();
        }

        let path = format!("{prefix}/{}", stable_commands_key(&commands));
        if !registry.iter().any(|(existing_path, existing_commands)| {
            existing_path == &path && existing_commands == &commands
        }) {
            registry.push((path.clone(), commands));
        }
        path
    })
}

/// Drain all dynamically-registered anonymous functions registered on the
/// *current thread*.
///
/// Called once by the component builder after all user functions have run
/// for one export, so all `register_dyn_fn` calls made during that export
/// (which runs synchronously on this thread) are guaranteed to have
/// completed. Never observes or clears another thread's registrations.
pub fn drain_dyn_fns() -> Vec<(String, Vec<String>)> {
    REGISTRY.with_borrow_mut(std::mem::take)
}

fn stable_commands_key(commands: &[String]) -> String {
    let mut h: u32 = 2_166_136_261;
    for command in commands {
        for b in command.bytes().chain(std::iter::once(0)) {
            h ^= b as u32;
            h = h.wrapping_mul(16_777_619);
        }
    }
    format!("{h:08x}")
}

#[cfg(test)]
mod dyn_fn_registry_tests {
    use super::*;

    #[test]
    fn drain_returns_empty_when_nothing_registered() {
        let _ = drain_dyn_fns(); // clear any leftover state from this thread
        assert!(drain_dyn_fns().is_empty());
    }

    #[test]
    fn register_dyn_fn_dedup_reuses_path_for_identical_body() {
        let _ = drain_dyn_fns();
        let a = register_dyn_fn_dedup("sand/test_prefix", vec!["say a".to_string()]);
        let b = register_dyn_fn_dedup("sand/test_prefix", vec!["say a".to_string()]);
        assert_eq!(a, b);
        let drained = drain_dyn_fns();
        assert_eq!(drained.len(), 1);
    }

    #[test]
    fn register_dyn_fn_dedup_distinguishes_different_bodies() {
        let _ = drain_dyn_fns();
        let a = register_dyn_fn_dedup("sand/test_prefix", vec!["say a".to_string()]);
        let b = register_dyn_fn_dedup("sand/test_prefix", vec!["say b".to_string()]);
        assert_ne!(a, b);
        let drained = drain_dyn_fns();
        assert_eq!(drained.len(), 2);
    }

    #[test]
    fn register_dyn_fn_dedup_path_is_deterministic_across_calls() {
        let _ = drain_dyn_fns();
        let first = register_dyn_fn_dedup("sand/test_prefix", vec!["say hello".to_string()]);
        let _ = drain_dyn_fns();
        let second = register_dyn_fn_dedup("sand/test_prefix", vec!["say hello".to_string()]);
        assert_eq!(
            first, second,
            "identical body must yield identical content-addressed path across drains"
        );
    }

    #[test]
    fn drain_empties_the_registry_so_a_second_drain_is_empty() {
        let _ = drain_dyn_fns();
        register_dyn_fn_dedup("sand/test_prefix", vec!["say once".to_string()]);
        let first_drain = drain_dyn_fns();
        assert_eq!(first_drain.len(), 1);
        let second_drain = drain_dyn_fns();
        assert!(
            second_drain.is_empty(),
            "a second drain must not re-observe already-drained entries"
        );
    }

    #[test]
    fn registry_is_isolated_per_thread() {
        // The core guarantee behind the LIM-EXP-006 fix: two threads
        // registering/draining concurrently never observe or clear each
        // other's entries, because the registry is thread-local rather
        // than a single process-global Mutex.
        let _ = drain_dyn_fns();
        register_dyn_fn_dedup("sand/main_thread", vec!["say main".to_string()]);

        let handle = std::thread::spawn(|| {
            // A fresh thread starts with an empty thread-local registry —
            // it must not see the main thread's "say main" entry.
            let initial = drain_dyn_fns();
            assert!(
                initial.is_empty(),
                "a new thread must not observe another thread's registrations"
            );
            register_dyn_fn_dedup("sand/other_thread", vec!["say other".to_string()]);
            drain_dyn_fns()
        });
        let other_thread_drain = handle.join().unwrap();
        assert_eq!(other_thread_drain.len(), 1);
        assert_eq!(other_thread_drain[0].1, vec!["say other".to_string()]);

        // The main thread's own registration must still be there, untouched
        // by the other thread's register/drain calls.
        let main_thread_drain = drain_dyn_fns();
        assert_eq!(main_thread_drain.len(), 1);
        assert_eq!(main_thread_drain[0].1, vec!["say main".to_string()]);
    }

    #[test]
    fn register_dyn_fn_deduplicates_identical_path_and_body() {
        let _ = drain_dyn_fns();
        register_dyn_fn("sand/exact_path".to_string(), vec!["say x".to_string()]);
        register_dyn_fn("sand/exact_path".to_string(), vec!["say x".to_string()]);
        let drained = drain_dyn_fns();
        assert_eq!(drained.len(), 1);
    }
}
