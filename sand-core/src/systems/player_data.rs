//! Typed player-data schema helpers (`systems-player-data` feature).
//!
//! Provides typed field handles plus a builder for defining per-player data
//! schemas backed by existing scoreboard and state primitives.
//! [`PlayerDataSchema`] is the user-facing definition; [`PlayerSchema`] remains
//! a compatible alias. Storage schemas from `#[derive(SandStorage)]` can also be
//! attached for unified introspection and documentation.
//!
//! # Lifecycle scope
//!
//! Field handles provide typed access, while schema registration generates
//! setup and non-clobbering initialization commands. It does **not**
//! automatically choose application lifecycle events.
//!
//! You must:
//! - Call [`PlayerDataSchema::define_all`] from your load function to define scoreboard objectives.
//! - Call [`PlayerDataSchema::init_player`] from a join or first-join handler to set player defaults.
//! - Wire timer/cooldown ticks and lifecycle manually using the underlying [`Timer`] and
//!   [`Cooldown`] APIs (see their docs for tick management).
//!
//! The handles reuse the same objectives as [`ScoreVar`], [`Flag`], [`Timer`],
//! [`Cooldown`], and [`GameState`]; no duplicate scoreboard representation is
//! created.
//!
//! # Naming and namespacing
//!
//! `PlayerDataSchema::new("magic")` accepts a **human label** for the schema.  It
//! does **not** prefix scoreboard objective names: the objective name is
//! determined entirely by the [`ScoreVar`], [`Flag`], or [`Cooldown`] you pass
//! in.  This means two schemas *can* share an objective if they register the
//! same static variable — which is valid (they share that score).  If you need
//! separate objectives that both map to a logical field named `"mana"`, create
//! two distinct static variables.
//!
//! ```rust,ignore
//! // These two schemas share the "mana" objective because they share the static:
//! static MANA: ScoreVar<i32> = ScoreVar::new("mana");
//!
//! let magic   = PlayerDataSchema::new("magic").score(&MANA, 100);
//! let stamina = PlayerDataSchema::new("stamina").score(&MANA, 50); // same objective
//!
//! // To keep them separate, use distinct statics:
//! static MAGIC_MANA:   ScoreVar<i32> = ScoreVar::new("magic_mana");
//! static STAMINA_MANA: ScoreVar<i32> = ScoreVar::new("stamina_mana");
//! ```
//!
//! # Scoreboard fields vs storage fields
//!
//! Use **scoreboard fields** (`score`, `flag`, `cooldown`) for:
//! - Per-player numeric state (mana, health stages, timers)
//! - Boolean flags (has_ability_unlocked, is_in_combat)
//! - Cooldown timers
//!
//! Use **storage fields** (`storage`) for:
//! - Rich per-pack state that doesn't fit in integers (item data, config)
//! - Global pack state shared across players
//! - Complex compound structures
//!
//! ⚠️ **Important**: Minecraft `data storage` is **global**, not per-player.
//! Attaching a storage schema to a `PlayerDataSchema` does **not** create a
//! per-player storage slot.  Sand does not automatically key storage paths by
//! player UUID.  See [`PlayerDataSchema::storage`] for details and workarounds.
//!
//! # `define_all()` behavior
//!
//! `define_all()` emits `scoreboard objectives add` commands for every
//! registered scoreboard field.  Storage schemas do **not** generate commands —
//! Minecraft NBT storage paths require no explicit definition; they spring into
//! existence on first write.
//!
//! Calling `define_all()` multiple times is safe: Minecraft prints a message
//! if the objective already exists but does not abort.
//!
//! # Example
//!
//! ```rust,ignore
//! use sand_core::state::{Cooldown, Flag, ScoreVar, Ticks, Timer};
//! use sand_core::systems::player_data::PlayerDataSchema;
//! use sand_macros::SandStorage;
//!
//! // Scoreboard statics (one per objective name):
//! static MANA:      ScoreVar<i32> = ScoreVar::new("mana");
//! static HAS_CELLS: Flag          = Flag::new("has_cells");
//! static DASH_CD:   Cooldown      = Cooldown::new("dash", Ticks::seconds(3));
//! static REGEN:     Timer         = Timer::new("regen", Ticks::seconds(2));
//!
//! // Storage schema for rich compound state:
//! #[derive(SandStorage)]
//! #[sand(storage = "powers:players", root = "players")]
//! pub struct PlayerMagic {
//!     pub max_mana: i32,
//!     pub tier: i32,
//! }
//!
//! fn player_magic_schema() -> PlayerDataSchema {
//!     PlayerDataSchema::new("magic")
//!         .score(&MANA, 100)           // default mana = 100
//!         .flag(&HAS_CELLS, false)     // has_cells starts false
//!         .cooldown(&DASH_CD)          // cooldown objective — no default value
//!         .timer(&REGEN)               // timer objective — no default value
//!         .storage(PlayerMagic::SCHEMA) // attached for introspection
//! }
//!
//! // In your load function:
//! // schema.define_all()      → define every scoreboard objective
//!
//! // In your join handler:
//! // schema.init_player("@s") → set defaults for new player
//! ```

use std::marker::PhantomData;

use crate::condition::Condition;
use crate::state::{
    Cooldown, Flag, FlagRef, GameState, GameStateRef, NbtRef, ScoreRef, ScoreVar, StorageField,
    StorageSchema, Ticks, Timer, TypedGameState,
};

// ── Typed field handles ──────────────────────────────────────────────────────

/// Schema field handle backed by the existing [`ScoreVar`].
pub struct ScoreField<T = i32> {
    value: ScoreVar<T>,
    default: i32,
}

impl<T> ScoreField<T> {
    /// Defines an integer-backed player field stored in the named scoreboard objective.
    pub const fn new(objective: &'static str) -> Self {
        Self {
            value: ScoreVar::new(objective),
            default: 0,
        }
    }

    /// Sets the score assigned when this player field is initialized.
    pub const fn default(mut self, value: i32) -> Self {
        self.default = value;
        self
    }

    /// Binds this field to one player's selector.
    pub fn of<'a>(&'a self, selector: &str) -> ScoreRef<'a, T> {
        self.value.of(selector)
    }

    /// Returns the typed scoreboard variable underlying this field.
    pub fn value(&self) -> &ScoreVar<T> {
        &self.value
    }

    /// Returns the score assigned during player initialization.
    pub fn default_value(&self) -> i32 {
        self.default
    }
}

/// Schema field handle backed by the existing [`Flag`].
pub struct FlagField {
    value: Flag,
    default: bool,
}

impl FlagField {
    /// Defines a boolean player field stored in the named scoreboard objective.
    pub const fn new(objective: &'static str) -> Self {
        Self {
            value: Flag::new(objective),
            default: false,
        }
    }

    /// Sets the boolean assigned when this player field is initialized.
    pub const fn default(mut self, value: bool) -> Self {
        self.default = value;
        self
    }

    /// Binds this flag field to one player's selector.
    pub fn of<'a>(&'a self, selector: &str) -> FlagRef<'a> {
        self.value.of(selector)
    }

    /// Returns the typed scoreboard flag underlying this field.
    pub fn value(&self) -> &Flag {
        &self.value
    }

    /// Returns the flag assigned during player initialization.
    pub fn default_value(&self) -> bool {
        self.default
    }
}

/// Schema field handle backed by the existing [`Timer`].
pub struct TimerField {
    value: Timer,
}

impl TimerField {
    /// Defines a player timer with its scoreboard objective and duration.
    pub const fn new(objective: &'static str, duration: Ticks) -> Self {
        Self {
            value: Timer::new(objective, duration),
        }
    }

    /// Binds this timer field to one player's selector.
    pub fn of<'a>(&'a self, selector: impl Into<String>) -> TimerFieldRef<'a> {
        TimerFieldRef {
            field: self,
            selector: selector.into(),
        }
    }

    /// Returns the typed timer underlying this field.
    pub fn value(&self) -> &Timer {
        &self.value
    }
}

pub struct TimerFieldRef<'a> {
    field: &'a TimerField,
    selector: String,
}

impl TimerFieldRef<'_> {
    /// Renders the command that starts this player's timer.
    pub fn start(&self) -> String {
        self.field.value.start(&self.selector)
    }

    /// Renders the command that resets this player's timer.
    pub fn reset(&self) -> String {
        self.field.value.reset(&self.selector)
    }

    /// Builds the typed condition for active.
    pub fn active(&self) -> Condition {
        self.field.value.active(&self.selector)
    }

    /// Builds the typed condition for expired.
    pub fn expired(&self) -> Condition {
        self.field.value.expired(&self.selector)
    }
}

/// Schema field handle backed by the existing [`Cooldown`].
pub struct CooldownField {
    value: Cooldown,
}

impl CooldownField {
    /// Defines a player cooldown with its scoreboard objective and duration.
    pub const fn new(objective: &'static str, duration: Ticks) -> Self {
        Self {
            value: Cooldown::new(objective, duration),
        }
    }

    /// Binds this cooldown field to one player's selector.
    pub fn of<'a>(&'a self, selector: impl Into<String>) -> CooldownFieldRef<'a> {
        CooldownFieldRef {
            field: self,
            selector: selector.into(),
        }
    }

    /// Returns the typed cooldown underlying this field.
    pub fn value(&self) -> &Cooldown {
        &self.value
    }
}

pub struct CooldownFieldRef<'a> {
    field: &'a CooldownField,
    selector: String,
}

impl CooldownFieldRef<'_> {
    /// Renders the command that starts this player's cooldown.
    pub fn start(&self) -> String {
        self.field.value.start(&self.selector)
    }

    /// Renders the command that stops this player's cooldown.
    pub fn stop(&self) -> String {
        self.field.value.stop(&self.selector)
    }

    /// Builds the typed condition for ready.
    pub fn ready(&self) -> Condition {
        self.field.value.ready(&self.selector)
    }

    /// Builds the typed condition for active.
    pub fn active(&self) -> Condition {
        self.field.value.active(&self.selector)
    }
}

/// Schema field handle backed by the existing enum-backed [`GameState`].
pub struct GameStateField<S: TypedGameState> {
    value: GameState<S>,
}

impl<S: TypedGameState> GameStateField<S> {
    /// Defines an enum-like player state stored in the named scoreboard objective.
    pub const fn new(objective: &'static str) -> Self {
        Self {
            value: GameState::new(objective),
        }
    }

    /// Defines a player state with an explicit initial encoded score.
    pub const fn with_default_score(objective: &'static str, default: i32) -> Self {
        Self {
            value: GameState::with_default_score(objective, default),
        }
    }

    /// Binds this game-state field to one player's selector.
    pub fn of<'a>(&'a self, selector: &str) -> GameStateRef<'a, S> {
        self.value.of(selector)
    }

    /// Returns the typed game-state variable underlying this field.
    pub fn value(&self) -> &GameState<S> {
        &self.value
    }
}

/// Explicit global-storage field handle. It never claims player scoping.
pub struct GlobalStorageField<Schema, T> {
    value: StorageField<Schema, T>,
    marker: PhantomData<fn() -> (Schema, T)>,
}

impl<Schema, T> GlobalStorageField<Schema, T> {
    /// Defines a global typed field within the supplied command-storage schema.
    pub const fn new(schema: &StorageSchema<Schema>, field: &'static str) -> Self {
        Self {
            value: schema.field(field),
            marker: PhantomData,
        }
    }

    /// Returns the typed NBT reference for this global storage field.
    pub fn nbt(&self) -> NbtRef<T> {
        self.value.path()
    }

    /// Returns the schema field underlying this global storage declaration.
    pub fn value(&self) -> &StorageField<Schema, T> {
        &self.value
    }
}

// ── FieldInit ─────────────────────────────────────────────────────────────────

enum FieldInit {
    Score { obj: String, default: i32 },
    Flag { obj: String, default: bool },
    TimerObj { obj: String },
    CooldownObj { obj: String },
    StateObj { obj: String, default: Option<i32> },
}

impl FieldInit {
    fn define_cmd(&self) -> String {
        match self {
            FieldInit::Score { obj, .. }
            | FieldInit::Flag { obj, .. }
            | FieldInit::TimerObj { obj }
            | FieldInit::CooldownObj { obj }
            | FieldInit::StateObj { obj, .. } => {
                format!("scoreboard objectives add {obj} dummy")
            }
        }
    }

    fn init_cmd(&self, selector: &str) -> Option<String> {
        match self {
            FieldInit::Score { obj, default } => Some(format!(
                "execute unless score {selector} {obj} matches -2147483648.. run scoreboard players set {selector} {obj} {default}"
            )),
            FieldInit::Flag { obj, default } => {
                let val = if *default { 1 } else { 0 };
                Some(format!(
                    "execute unless score {selector} {obj} matches -2147483648.. run scoreboard players set {selector} {obj} {val}"
                ))
            }
            FieldInit::TimerObj { .. } => None,
            FieldInit::CooldownObj { .. } => None,
            FieldInit::StateObj {
                obj,
                default: Some(default),
            } => Some(format!(
                "execute unless score {selector} {obj} matches -2147483648.. run scoreboard players set {selector} {obj} {default}"
            )),
            FieldInit::StateObj { default: None, .. } => None,
        }
    }
}

// ── StorageDescriptor ─────────────────────────────────────────────────────────

/// A lightweight descriptor for a storage schema attached to a [`PlayerDataSchema`].
///
/// Holds the raw storage ID and root path strings.  No commands are emitted
/// for storage schemas — Minecraft storage paths need no explicit definition.
#[derive(Debug, Clone)]
pub struct StorageDescriptor {
    /// Minecraft storage resource location (e.g. `"powers:players"`).
    pub storage: &'static str,
    /// NBT root path inside the storage (e.g. `"players"`).
    pub root: &'static str,
}

// ── PlayerDataSchema ──────────────────────────────────────────────────────────────

/// A mixed per-player data bundle: scoreboard fields, flags, cooldowns,
/// and attached storage schema references.
///
/// Build with the chained builder methods.  Call [`define_all`](PlayerDataSchema::define_all)
/// in your load function and [`init_player`](PlayerDataSchema::init_player) in join
/// handlers.  See the [module docs](self) for naming rules and per-player
/// storage limitations.
pub struct PlayerDataSchema {
    /// Human label for this schema.  Not used to prefix objective names.
    namespace: &'static str,
    fields: Vec<FieldInit>,
    storage_schemas: Vec<StorageDescriptor>,
}

impl PlayerDataSchema {
    /// Create an empty schema with the given human label.
    ///
    /// The label is for documentation/introspection only — it does **not**
    /// prefix scoreboard objectives.  Two schemas can share a label without
    /// conflict, and two schemas with the same-named `ScoreVar` will share
    /// an objective (which is often intentional).
    pub const fn new(namespace: &'static str) -> Self {
        Self {
            namespace,
            fields: Vec::new(),
            storage_schemas: Vec::new(),
        }
    }

    /// The human label passed to [`new`](Self::new).
    pub fn name(&self) -> &str {
        self.namespace
    }

    // ── Scoreboard fields ─────────────────────────────────────────────────────

    /// Register a `ScoreVar` with a default value for new players.
    ///
    /// The objective name comes from `var.objective_name()`, not from the
    /// schema namespace.
    pub fn score<T>(mut self, var: &ScoreVar<T>, default: i32) -> Self {
        self.fields.push(FieldInit::Score {
            obj: var.objective_name(),
            default,
        });
        self
    }

    /// Registers an integer score field for initialization in this player-data schema.
    pub fn score_field<T>(self, field: &ScoreField<T>) -> Self {
        self.score(field.value(), field.default_value())
    }

    /// Register a `Flag` with a default boolean value for new players.
    pub fn flag(mut self, flag: &Flag, default: bool) -> Self {
        self.fields.push(FieldInit::Flag {
            obj: flag.objective_name(),
            default,
        });
        self
    }

    /// Registers a boolean flag field for initialization in this player-data schema.
    pub fn flag_field(self, field: &FlagField) -> Self {
        self.flag(field.value(), field.default_value())
    }

    /// Register a [`Timer`] objective (define only; no per-player default).
    ///
    /// This method **only** defines/registers the timer's scoreboard objective.
    /// It does **not** automatically start ticks or wire lifecycle events.
    ///
    /// To actually use the timer in gameplay, you must separately:
    /// - Call the timer's tick methods during server ticks (e.g., in a `#[tick]` function).
    /// - Manage timer lifecycle wiring (e.g., starting timers in events).
    ///
    /// See [`Timer`] for tick/lifecycle APIs.
    pub fn timer(mut self, timer: &Timer) -> Self {
        self.fields.push(FieldInit::TimerObj {
            obj: timer.objective_name(),
        });
        self
    }

    /// Registers a timer field for lifecycle setup in this player-data schema.
    pub fn timer_field(self, field: &TimerField) -> Self {
        self.timer(field.value())
    }

    /// Register a `Cooldown` objective (define only; no per-player default).
    ///
    /// This method **only** defines/registers the cooldown's scoreboard objective.
    /// It does **not** automatically manage cooldown state or tick them down.
    ///
    /// To actually use the cooldown in gameplay, you must separately:
    /// - Call the cooldown's tick methods during server ticks (e.g., in a `#[tick]` function).
    /// - Manage cooldown lifecycle wiring (e.g., starting cooldowns on ability use).
    ///
    /// See [`Cooldown`] for tick/lifecycle APIs.
    pub fn cooldown(mut self, cd: &Cooldown) -> Self {
        self.fields.push(FieldInit::CooldownObj {
            obj: cd.objective_name(),
        });
        self
    }

    /// Registers a cooldown field for lifecycle setup in this player-data schema.
    pub fn cooldown_field(self, field: &CooldownField) -> Self {
        self.cooldown(field.value())
    }

    /// Registers a typed game-state field for initialization in this player-data schema.
    pub fn game_state<S: TypedGameState>(mut self, field: &GameStateField<S>) -> Self {
        self.fields.push(FieldInit::StateObj {
            obj: field.value().objective_name(),
            default: field.value().default_score(),
        });
        self
    }

    /// Attach one explicit global-storage handle for schema introspection.
    pub fn global_storage_field<Schema, T>(
        mut self,
        field: &GlobalStorageField<Schema, T>,
    ) -> Self {
        self.storage_schemas.push(StorageDescriptor {
            storage: field.value().storage(),
            root: field.value().root_path(),
        });
        self
    }

    // ── Storage schemas ───────────────────────────────────────────────────────

    /// Attach a [`StorageSchema`] to this player schema for tracking and
    /// documentation.
    ///
    /// # What this does
    ///
    /// Storage schemas attached here are tracked for introspection via
    /// [`storage_locations`](Self::storage_locations).  **No commands are
    /// emitted** for storage schemas by [`define_all`](Self::define_all) —
    /// Minecraft NBT storage paths require no explicit definition.
    ///
    /// # Per-player storage limitation
    ///
    /// Minecraft `data storage` is a **global** namespace, not per-player.
    /// Attaching a storage schema to `PlayerDataSchema` does not automatically
    /// key storage by player UUID or name.
    ///
    /// If you need per-player compound data, the common approaches are:
    ///
    /// 1. **Scoreboard for numeric fields** (recommended for most cases).
    /// 2. **UUID-keyed storage paths** — write a helper that computes
    ///    `data modify storage my_pack:players <uuid>.mana set value 100`.
    ///    Sand does not generate these automatically because the UUID is a
    ///    runtime value, not a compile-time constant.
    /// 3. **Explicit entity data where vanilla permits it** — use the unified
    ///    typed NBT API. Arbitrary player NBT and inventory writes are rejected;
    ///    use scoreboard or typed item-location operations for those values.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(SandStorage)]
    /// #[sand(storage = "powers:global", root = "config")]
    /// pub struct PackConfig { pub max_mana: i32 }
    ///
    /// let schema = PlayerDataSchema::new("magic")
    ///     .score(&MANA, 100)
    ///     .storage(PackConfig::SCHEMA); // global config, not per-player
    /// ```
    pub fn storage<T>(mut self, schema: StorageSchema<T>) -> Self {
        self.storage_schemas.push(StorageDescriptor {
            storage: schema.storage(),
            root: schema.root_path(),
        });
        self
    }

    // ── Command generation ────────────────────────────────────────────────────

    /// Commands to define all scoreboard objectives (for your load function).
    ///
    /// Storage schemas do **not** generate commands — Minecraft NBT storage
    /// needs no explicit definition.
    ///
    /// The emitted `scoreboard objectives add` commands are idempotent: if an
    /// objective already exists, Minecraft prints a warning but does not abort.
    /// It is safe to call `define_all()` more than once or to run its output
    /// in every reload.
    pub fn define_all(&self) -> Vec<String> {
        let mut seen = std::collections::BTreeSet::new();
        let mut commands = Vec::new();
        for field in &self.fields {
            let command = field.define_cmd();
            let objective = command
                .split_whitespace()
                .nth(3)
                .unwrap_or_default()
                .to_string();
            if seen.insert(objective) {
                commands.push(command);
            }
        }
        commands
    }

    /// Commands to initialize a new player's scores to their defaults.
    ///
    /// Each command uses `unless score … matches -2147483648..` so it is a
    /// no-op for players who already have scores (reconnects, respawns, etc.).
    ///
    /// Cooldowns have no default value, so they are skipped here.  Storage
    /// schemas are not affected by this method.
    ///
    /// Compatibility/raw path: `selector` is an unvalidated string,
    /// interpolated directly into generated commands. Prefer
    /// [`PlayerDataSchema::try_init_player`] in normal code — see
    /// [#146](https://github.com/ThatOneToast/sand/issues/146).
    pub fn init_player(&self, selector: &str) -> Vec<String> {
        let mut seen = std::collections::BTreeSet::new();
        let mut commands = Vec::new();
        for command in self
            .fields
            .iter()
            .filter_map(|field| field.init_cmd(selector))
        {
            let objective = command
                .split_whitespace()
                .nth(4)
                .unwrap_or_default()
                .to_string();
            if seen.insert(objective) {
                commands.push(command);
            }
        }
        commands
    }

    /// Validated counterpart to [`PlayerDataSchema::init_player`] — takes a typed
    /// [`sand_commands::ScoreHolder`] and validates it before generating
    /// commands, instead of interpolating an unvalidated selector string.
    ///
    /// ```
    /// use sand_core::systems::player_data::PlayerDataSchema;
    /// use sand_core::state::ScoreVar;
    /// use sand_commands::ScoreHolder;
    ///
    /// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    /// let schema = PlayerDataSchema::new("player").score(&MANA, 100);
    ///
    /// assert!(schema.try_init_player(ScoreHolder::self_()).is_ok());
    /// assert!(schema.try_init_player(ScoreHolder::fake("bad holder")).is_err());
    /// ```
    pub fn try_init_player(
        &self,
        holder: impl Into<sand_commands::ScoreHolder>,
    ) -> sand_commands::CommandResult<Vec<String>> {
        let holder = holder.into();
        // Every emitted command is `execute unless score <holder> <obj>
        // matches ... run scoreboard players set <holder> <obj> ...`, which
        // (like `ScoreVar::try_of`) requires exactly one score holder —
        // reject wildcards and multi-entity selectors here rather than
        // emitting an `execute unless score *` / `execute unless score @a`
        // that Minecraft will refuse to parse.
        holder.validate_single(&sand_commands::CommandProfile::unprofiled())?;
        Ok(self.init_player(&holder.to_string()))
    }

    // ── Introspection ─────────────────────────────────────────────────────────

    /// Returns descriptors for all attached storage schemas.
    ///
    /// Each descriptor exposes the storage resource location string and the
    /// NBT root path.  Use this for debugging, code generation, or building
    /// documentation.
    pub fn storage_locations(&self) -> &[StorageDescriptor] {
        &self.storage_schemas
    }

    /// `true` if at least one storage schema has been attached.
    pub fn has_storage(&self) -> bool {
        !self.storage_schemas.is_empty()
    }

    /// The number of registered scoreboard-style fields
    /// (score + flag + timer + cooldown).
    pub fn scoreboard_field_count(&self) -> usize {
        self.fields.len()
    }
}

/// Compatibility alias for the earlier, shorter player-schema name.
pub type PlayerSchema = PlayerDataSchema;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Cooldown, Flag, ScoreVar, StorageSchema, Ticks, Timer, TypedGameState};

    static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    static HAS_CELLS: Flag = Flag::new("has_cells");
    static REGEN: Timer = Timer::new("regen", Ticks::new(40));
    static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
    static MANA_FIELD: ScoreField<i32> = ScoreField::new("arcane_mana").default(100);
    static WAND_FIELD: FlagField = FlagField::new("arcane_wand").default(false);
    static REGEN_FIELD: TimerField = TimerField::new("arcane_regen", Ticks::new(40));
    static CAST_FIELD: CooldownField = CooldownField::new("arcane_cast", Ticks::new(60));

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Class {
        Unbound = 0,
        Mage = 1,
    }

    impl TypedGameState for Class {
        fn to_score(self) -> i32 {
            self as i32
        }

        fn from_score(score: i32) -> Option<Self> {
            match score {
                0 => Some(Self::Unbound),
                1 => Some(Self::Mage),
                _ => None,
            }
        }
    }

    static CLASS_FIELD: GameStateField<Class> =
        GameStateField::with_default_score("arcane_class", 0);

    fn schema() -> PlayerDataSchema {
        PlayerDataSchema::new("test_pack")
            .score(&MANA, 100)
            .flag(&HAS_CELLS, false)
            .timer(&REGEN)
            .cooldown(&DASH)
    }

    #[test]
    fn typed_field_handles_reuse_existing_primitives() {
        let schema = PlayerDataSchema::new("arcane")
            .score_field(&MANA_FIELD)
            .flag_field(&WAND_FIELD)
            .timer_field(&REGEN_FIELD)
            .cooldown_field(&CAST_FIELD)
            .game_state(&CLASS_FIELD);
        assert_eq!(
            schema.define_all(),
            vec![
                "scoreboard objectives add arcane_mana dummy",
                "scoreboard objectives add arcane_wand dummy",
                "scoreboard objectives add arcane_regen dummy",
                "scoreboard objectives add arcane_cast dummy",
                "scoreboard objectives add arcane_class dummy",
            ]
        );
        assert_eq!(
            schema.init_player("@s"),
            vec![
                "execute unless score @s arcane_mana matches -2147483648.. run scoreboard players set @s arcane_mana 100",
                "execute unless score @s arcane_wand matches -2147483648.. run scoreboard players set @s arcane_wand 0",
                "execute unless score @s arcane_class matches -2147483648.. run scoreboard players set @s arcane_class 0",
            ]
        );
        assert_eq!(
            MANA_FIELD
                .of("@s")
                .gte(25)
                .execute_commands(false, "say mana"),
            ["execute if score @s arcane_mana matches 25.. run say mana"]
        );
        assert_eq!(
            WAND_FIELD
                .of("@s")
                .is_true()
                .execute_commands(false, "say wand"),
            ["execute if score @s arcane_wand matches 1 run say wand"]
        );
        assert_eq!(
            CAST_FIELD
                .of("@s")
                .ready()
                .execute_commands(false, "say ready"),
            ["execute if score @s arcane_cast matches 0 run say ready"]
        );
        assert_eq!(
            CLASS_FIELD
                .of("@s")
                .is(Class::Mage)
                .execute_commands(false, "say mage"),
            ["execute if score @s arcane_class matches 1 run say mage"]
        );
    }

    #[test]
    fn duplicate_typed_handles_do_not_duplicate_objectives() {
        let schema = PlayerDataSchema::new("arcane")
            .score_field(&MANA_FIELD)
            .score_field(&MANA_FIELD);
        assert_eq!(schema.define_all().len(), 1);
        assert_eq!(schema.init_player("@s").len(), 1);
    }

    // ── existing tests (unchanged behavior) ─────────────────────────────────

    #[test]
    fn define_all_generates_all_scoreboard_commands() {
        let cmds = schema().define_all();
        assert_eq!(cmds.len(), 4);
        for cmd in &cmds {
            assert!(cmd.starts_with("scoreboard objectives add "), "got: {cmd}");
        }
        assert!(cmds[0].contains("mana"), "score obj: {}", cmds[0]);
        assert!(cmds[1].contains("has_cells"), "flag obj: {}", cmds[1]);
        assert!(cmds[2].contains("regen"), "timer obj: {}", cmds[2]);
        assert!(cmds[3].contains("dash"), "cd obj: {}", cmds[3]);
    }

    #[test]
    fn init_player_skips_timer_and_cooldown() {
        let cmds = schema().init_player("@s");
        assert_eq!(cmds.len(), 2, "only score and flag have defaults");
    }

    #[test]
    fn init_player_score_default() {
        let cmds = schema().init_player("@s");
        assert!(
            cmds[0].contains("unless score @s mana matches -2147483648.."),
            "got: {}",
            cmds[0]
        );
        assert!(cmds[0].contains("set @s mana 100"), "got: {}", cmds[0]);
    }

    #[test]
    fn init_player_flag_default_false() {
        let cmds = schema().init_player("@s");
        assert!(
            cmds[1].contains("unless score @s has_cells matches -2147483648.."),
            "got: {}",
            cmds[1]
        );
        assert!(cmds[1].contains("set @s has_cells 0"), "got: {}", cmds[1]);
    }

    // ── #146: try_init_player ───────────────────────────────────────────────

    #[test]
    fn try_init_player_matches_infallible_init_player_for_valid_holder() {
        use sand_commands::ScoreHolder;
        assert_eq!(
            schema().try_init_player(ScoreHolder::self_()).unwrap(),
            schema().init_player("@s")
        );
    }

    #[test]
    fn try_init_player_rejects_invalid_fake_player_holder() {
        use sand_commands::ScoreHolder;
        assert!(
            schema()
                .try_init_player(ScoreHolder::fake("bad holder"))
                .is_err()
        );
    }

    #[test]
    fn try_init_player_rejects_wildcard_and_multi_entity_holders() {
        // Every emitted command is `execute unless score <holder> ...`,
        // which requires exactly one score holder — a wildcard or a
        // multi-entity selector must be rejected rather than emitting
        // `execute unless score * mana matches ...` /
        // `execute unless score @a mana matches ...`.
        use sand_commands::ScoreHolder;
        use sand_commands::selector::Selector;
        assert!(schema().try_init_player(ScoreHolder::wildcard()).is_err());
        assert!(
            schema()
                .try_init_player(ScoreHolder::entity(Selector::all_players()))
                .is_err()
        );
    }

    // ── name accessor ────────────────────────────────────────────────────────

    #[test]
    fn name_accessor_returns_label() {
        let s = PlayerDataSchema::new("my_pack");
        assert_eq!(s.name(), "my_pack");
    }

    // ── namespace does not prefix objectives ─────────────────────────────────

    #[test]
    fn namespace_does_not_prefix_objectives() {
        // Two schemas registering the same ScoreVar share the same objective.
        // The namespace is a label only.
        static MANA2: ScoreVar<i32> = ScoreVar::new("mana");
        let schema_a = PlayerDataSchema::new("magic").score(&MANA2, 100);
        let schema_b = PlayerDataSchema::new("stamina").score(&MANA2, 50);
        let cmds_a = schema_a.define_all();
        let cmds_b = schema_b.define_all();
        assert_eq!(cmds_a[0], cmds_b[0], "same ScoreVar → same objective");
    }

    #[test]
    fn distinct_statics_produce_distinct_objectives() {
        static MAGIC_MANA: ScoreVar<i32> = ScoreVar::new("magic_mana");
        static STAMINA_MANA: ScoreVar<i32> = ScoreVar::new("stamina_mana");
        let schema_a = PlayerDataSchema::new("magic").score(&MAGIC_MANA, 100);
        let schema_b = PlayerDataSchema::new("stamina").score(&STAMINA_MANA, 50);
        let obj_a = &schema_a.define_all()[0];
        let obj_b = &schema_b.define_all()[0];
        assert_ne!(obj_a, obj_b);
        assert!(obj_a.contains("magic_mana"), "got: {obj_a}");
        assert!(obj_b.contains("stamina_mana"), "got: {obj_b}");
    }

    // ── define_all idempotency ────────────────────────────────────────────────

    #[test]
    fn define_all_is_idempotent_same_output() {
        let s = schema();
        let first = s.define_all();
        let second = s.define_all();
        assert_eq!(
            first, second,
            "define_all should produce identical output each call"
        );
    }

    // ── storage schema registration ──────────────────────────────────────────

    const TEST_SCHEMA: StorageSchema<()> = StorageSchema::new("powers:players", "players");

    #[test]
    fn storage_attaches_descriptor() {
        let s = schema().storage(TEST_SCHEMA);
        assert!(s.has_storage());
        assert_eq!(s.storage_locations().len(), 1);
        let desc = &s.storage_locations()[0];
        assert_eq!(desc.storage, "powers:players");
        assert_eq!(desc.root, "players");
    }

    #[test]
    fn storage_multiple_schemas() {
        const SCHEMA_B: StorageSchema<u8> = StorageSchema::new("powers:config", "config");
        let s = schema().storage(TEST_SCHEMA).storage(SCHEMA_B);
        assert_eq!(s.storage_locations().len(), 2);
    }

    #[test]
    fn define_all_excludes_storage_schemas() {
        // Storage schemas do not emit commands — nothing to define in Minecraft.
        let cmds_without = schema().define_all();
        let cmds_with = schema().storage(TEST_SCHEMA).define_all();
        assert_eq!(
            cmds_without, cmds_with,
            "attaching a storage schema must not add extra define_all commands"
        );
    }

    #[test]
    fn scoreboard_field_count_excludes_storage() {
        let s = schema().storage(TEST_SCHEMA);
        assert_eq!(s.scoreboard_field_count(), 4); // score + flag + timer + cooldown only
    }

    #[test]
    fn no_storage_by_default() {
        let s = schema();
        assert!(!s.has_storage());
        assert_eq!(s.storage_locations().len(), 0);
    }

    #[test]
    fn schema_with_only_storage() {
        let s = PlayerDataSchema::new("global").storage(TEST_SCHEMA);
        assert!(s.has_storage());
        assert_eq!(s.scoreboard_field_count(), 0);
        assert!(
            s.define_all().is_empty(),
            "no scoreboard fields → no define cmds"
        );
        assert!(
            s.init_player("@s").is_empty(),
            "no scoreboard fields → no init cmds"
        );
    }

    #[test]
    fn overlapping_field_names_across_schemas_separate_statics() {
        // If you want two schemas to have logically independent "mana" fields,
        // the statics must be different.
        static SCHEMA_A_MANA: ScoreVar<i32> = ScoreVar::new("magic_mana");
        static SCHEMA_B_MANA: ScoreVar<i32> = ScoreVar::new("stamina_mana");
        let sa = PlayerDataSchema::new("magic").score(&SCHEMA_A_MANA, 0);
        let sb = PlayerDataSchema::new("stamina").score(&SCHEMA_B_MANA, 0);
        let da = sa.define_all()[0].clone();
        let db = sb.define_all()[0].clone();
        assert_ne!(da, db, "distinct statics → distinct objectives");
    }

    #[test]
    fn player_data_schema_alias_matches_player_schema() {
        let schema: PlayerDataSchema = PlayerDataSchema::new("alias").timer(&REGEN);
        assert_eq!(schema.name(), "alias");
        assert_eq!(
            schema.define_all(),
            vec!["scoreboard objectives add regen dummy"]
        );
    }
}
