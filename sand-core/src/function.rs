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
        revoke: bool,
    },

    /// All-deaths detection via the `deathCount` scoreboard criterion.
    ///
    /// Fires for every player death (mob, fall, fire, void, `/kill`, …).
    /// Sand generates a `__sand_death_check` tick function and a
    /// `__sand_death_init` load function (adds the `deathCount` objective).
    DeathTick,

    /// Fires on the first tick after a player (re)joins the server.
    ///
    /// Sand generates a `__sand_join_check` tick function that detects players
    /// who lack the `__sand_online` entity tag. Because entity tags are
    /// removed when a player disconnects, the event re-fires on every login.
    /// Handlers run before the `__sand_online` tag is applied.
    JoinTick,

    /// Fires on the tick after a player respawns from death.
    ///
    /// Piggybacks on the death check: dying players receive a
    /// `__sand_was_dead` tag, which is cleared once they are no longer in
    /// spectator mode. Sand generates a `__sand_respawn_check` tick function.
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
    /// At build time, Sand calls `make_trigger()` and `make_condition()` to
    /// determine which dispatch path to use. Exactly one must return `Some`.
    ///
    /// - `Some` from `make_trigger` → advancement-backed dispatch
    /// - `Some` from `make_condition` → tick-poll dispatch
    Custom {
        /// Returns `Some(AdvancementTrigger)` when using advancement dispatch.
        make_trigger: fn() -> Option<crate::AdvancementTrigger>,
        /// Returns `Some(condition_string)` when using tick-poll dispatch.
        make_condition: fn() -> Option<String>,
        /// Whether to revoke the advancement after firing (advancement dispatch only).
        revoke: fn() -> bool,
    },
}

/// Descriptor for a function registered via `#[sand_macros::event]`.
///
/// Collected via [`inventory::iter::<EventDescriptor>`] at export time.
///
/// # Fields
/// - `path` — function resource location path (no namespace), e.g. `"on_join"`
/// - `id_override` — optional full advancement ID override (advancement dispatch only)
/// - `make` — factory that returns the Vec<String> of mcfunction commands
/// - `dispatch` — whether to use an advancement trigger or the DeathTick tick loop
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
/// Register with [`temp_score!`] and Sand will emit
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

use std::sync::Mutex;
use std::sync::OnceLock;

fn dyn_fn_registry() -> &'static Mutex<Vec<(String, Vec<String>)>> {
    static REGISTRY: OnceLock<Mutex<Vec<(String, Vec<String>)>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

/// Register an anonymous function body at runtime.
///
/// Called by anonymous `run_fn!` blocks that capture local variables.
/// The `commands` are the pre-computed mcfunction lines.
pub fn register_dyn_fn(path: String, commands: Vec<String>) {
    dyn_fn_registry().lock().unwrap().push((path, commands));
}

/// Drain all dynamically-registered anonymous functions.
///
/// Called once by the component builder after all user functions have run,
/// so all `register_dyn_fn` calls are guaranteed to have completed.
pub fn drain_dyn_fns() -> Vec<(String, Vec<String>)> {
    std::mem::take(&mut *dyn_fn_registry().lock().unwrap())
}
