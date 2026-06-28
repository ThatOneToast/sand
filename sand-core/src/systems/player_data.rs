//! Manual player-data schema helpers (`systems-player-data` feature).
//!
//! Provides a builder API for defining typed per-player data schemas backed
//! by scoreboard objectives.  Storage schemas from [`SandStorage`] can also be
//! attached for unified introspection and documentation.
//!
//! This module does **not** auto-register player data schemas or generate
//! lifecycle wiring today.  Call [`PlayerSchema::define_all`] from your load
//! function and [`PlayerSchema::init_player`] from a join or first-join handler.
//! Automatic export/lifecycle wiring is future work tracked by #47 and #68.
//!
//! # Naming and namespacing
//!
//! `PlayerSchema::new("magic")` accepts a **human label** for the schema.  It
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
//! let magic   = PlayerSchema::new("magic").score(&MANA, 100);
//! let stamina = PlayerSchema::new("stamina").score(&MANA, 50); // same objective
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
//! Attaching a storage schema to a `PlayerSchema` does **not** create a
//! per-player storage slot.  Sand does not automatically key storage paths by
//! player UUID.  See [`PlayerSchema::storage`] for details and workarounds.
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
//! use sand_core::state::{ScoreVar, Flag, Cooldown, Ticks};
//! use sand_core::systems::player_data::PlayerSchema;
//! use sand_macros::SandStorage;
//!
//! // Scoreboard statics (one per objective name):
//! static MANA:      ScoreVar<i32> = ScoreVar::new("mana");
//! static HAS_CELLS: Flag          = Flag::new("has_cells");
//! static DASH_CD:   Cooldown      = Cooldown::new("dash", Ticks::seconds(3));
//!
//! // Storage schema for rich compound state:
//! #[derive(SandStorage)]
//! #[sand(storage = "powers:players", root = "players")]
//! pub struct PlayerMagic {
//!     pub max_mana: i32,
//!     pub tier: i32,
//! }
//!
//! fn player_magic_schema() -> PlayerSchema {
//!     PlayerSchema::new("magic")
//!         .score(&MANA, 100)           // default mana = 100
//!         .flag(&HAS_CELLS, false)     // has_cells starts false
//!         .cooldown(&DASH_CD)          // cooldown objective — no default value
//!         .storage(PlayerMagic::SCHEMA) // attached for introspection
//! }
//!
//! // In your load function:
//! // schema.define_all()      → define every scoreboard objective
//!
//! // In your join handler:
//! // schema.init_player("@s") → set defaults for new player
//! ```

use crate::state::{Cooldown, Flag, ScoreVar, StorageSchema};

// ── FieldInit ─────────────────────────────────────────────────────────────────

enum FieldInit {
    Score { obj: String, default: i32 },
    Flag { obj: String, default: bool },
    CooldownObj { obj: String },
}

impl FieldInit {
    fn define_cmd(&self) -> String {
        match self {
            FieldInit::Score { obj, .. }
            | FieldInit::Flag { obj, .. }
            | FieldInit::CooldownObj { obj } => {
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
            FieldInit::CooldownObj { .. } => None,
        }
    }
}

// ── StorageDescriptor ─────────────────────────────────────────────────────────

/// A lightweight descriptor for a storage schema attached to a [`PlayerSchema`].
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

// ── PlayerSchema ──────────────────────────────────────────────────────────────

/// A mixed per-player data bundle: scoreboard fields, flags, cooldowns,
/// and attached storage schema references.
///
/// Build with the chained builder methods.  Call [`define_all`](PlayerSchema::define_all)
/// in your load function and [`init_player`](PlayerSchema::init_player) in join
/// handlers.  See the [module docs](self) for naming rules and per-player
/// storage limitations.
pub struct PlayerSchema {
    /// Human label for this schema.  Not used to prefix objective names.
    namespace: &'static str,
    fields: Vec<FieldInit>,
    storage_schemas: Vec<StorageDescriptor>,
}

impl PlayerSchema {
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

    /// Register a `Flag` with a default boolean value for new players.
    pub fn flag(mut self, flag: &Flag, default: bool) -> Self {
        self.fields.push(FieldInit::Flag {
            obj: flag.objective_name(),
            default,
        });
        self
    }

    /// Register a `Cooldown` objective (define only; no per-player default).
    pub fn cooldown(mut self, cd: &Cooldown) -> Self {
        self.fields.push(FieldInit::CooldownObj {
            obj: cd.objective_name(),
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
    /// Attaching a storage schema to `PlayerSchema` does not automatically
    /// key storage by player UUID or name.
    ///
    /// If you need per-player compound data, the common approaches are:
    ///
    /// 1. **Scoreboard for numeric fields** (recommended for most cases).
    /// 2. **UUID-keyed storage paths** — write a helper that computes
    ///    `data modify storage my_pack:players <uuid>.mana set value 100`.
    ///    Sand does not generate these automatically because the UUID is a
    ///    runtime value, not a compile-time constant.
    /// 3. **Entity NBT** — `data modify entity @s … set value …` writes to
    ///    the player's own NBT, which is truly per-player but has a different
    ///    API (use [`StorageField`](crate::state::StorageField) paths with
    ///    entity selectors manually).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(SandStorage)]
    /// #[sand(storage = "powers:global", root = "config")]
    /// pub struct PackConfig { pub max_mana: i32 }
    ///
    /// let schema = PlayerSchema::new("magic")
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
        self.fields.iter().map(|f| f.define_cmd()).collect()
    }

    /// Commands to initialize a new player's scores to their defaults.
    ///
    /// Each command uses `unless score … matches -2147483648..` so it is a
    /// no-op for players who already have scores (reconnects, respawns, etc.).
    ///
    /// Cooldowns have no default value, so they are skipped here.  Storage
    /// schemas are not affected by this method.
    pub fn init_player(&self, selector: &str) -> Vec<String> {
        self.fields
            .iter()
            .filter_map(|f| f.init_cmd(selector))
            .collect()
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

    /// The number of registered scoreboard-style fields (score + flag + cooldown).
    pub fn scoreboard_field_count(&self) -> usize {
        self.fields.len()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Cooldown, Flag, ScoreVar, StorageSchema, Ticks};

    static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    static HAS_CELLS: Flag = Flag::new("has_cells");
    static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));

    fn schema() -> PlayerSchema {
        PlayerSchema::new("test_pack")
            .score(&MANA, 100)
            .flag(&HAS_CELLS, false)
            .cooldown(&DASH)
    }

    // ── existing tests (unchanged behavior) ─────────────────────────────────

    #[test]
    fn define_all_generates_three_commands() {
        let cmds = schema().define_all();
        assert_eq!(cmds.len(), 3);
        for cmd in &cmds {
            assert!(cmd.starts_with("scoreboard objectives add "), "got: {cmd}");
        }
        assert!(cmds[0].contains("mana"), "score obj: {}", cmds[0]);
        assert!(cmds[1].contains("has_cells"), "flag obj: {}", cmds[1]);
        assert!(cmds[2].contains("dash"), "cd obj: {}", cmds[2]);
    }

    #[test]
    fn init_player_skips_cooldown() {
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

    // ── name accessor ────────────────────────────────────────────────────────

    #[test]
    fn name_accessor_returns_label() {
        let s = PlayerSchema::new("my_pack");
        assert_eq!(s.name(), "my_pack");
    }

    // ── namespace does not prefix objectives ─────────────────────────────────

    #[test]
    fn namespace_does_not_prefix_objectives() {
        // Two schemas registering the same ScoreVar share the same objective.
        // The namespace is a label only.
        static MANA2: ScoreVar<i32> = ScoreVar::new("mana");
        let schema_a = PlayerSchema::new("magic").score(&MANA2, 100);
        let schema_b = PlayerSchema::new("stamina").score(&MANA2, 50);
        let cmds_a = schema_a.define_all();
        let cmds_b = schema_b.define_all();
        assert_eq!(cmds_a[0], cmds_b[0], "same ScoreVar → same objective");
    }

    #[test]
    fn distinct_statics_produce_distinct_objectives() {
        static MAGIC_MANA: ScoreVar<i32> = ScoreVar::new("magic_mana");
        static STAMINA_MANA: ScoreVar<i32> = ScoreVar::new("stamina_mana");
        let schema_a = PlayerSchema::new("magic").score(&MAGIC_MANA, 100);
        let schema_b = PlayerSchema::new("stamina").score(&STAMINA_MANA, 50);
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
        assert_eq!(s.scoreboard_field_count(), 3); // score + flag + cooldown only
    }

    #[test]
    fn no_storage_by_default() {
        let s = schema();
        assert!(!s.has_storage());
        assert_eq!(s.storage_locations().len(), 0);
    }

    #[test]
    fn schema_with_only_storage() {
        let s = PlayerSchema::new("global").storage(TEST_SCHEMA);
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
        let sa = PlayerSchema::new("magic").score(&SCHEMA_A_MANA, 0);
        let sb = PlayerSchema::new("stamina").score(&SCHEMA_B_MANA, 0);
        let da = sa.define_all()[0].clone();
        let db = sb.define_all()[0].clone();
        assert_ne!(da, db, "distinct statics → distinct objectives");
    }
}
