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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::player_data::ScoreField",
    aliases = ["sand::prelude::ScoreField"],
    module = "sand::systems",
    summary = "Schema field handle backed by the existing [`ScoreVar`].",
    context = "Schema field handle backed by the existing [`ScoreVar`]. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::player_data::ScoreField;",
    availability = ["Cargo feature: systems-player-data"],
)]
/// Schema field handle backed by the existing [`ScoreVar`].
pub struct ScoreField<T = i32> {
    value: ScoreVar<T>,
    default: i32,
}

impl<T> ScoreField<T> {
    /// Defines an integer-backed player field stored in the named scoreboard objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::ScoreField::new",
        aliases = ["sand::prelude::ScoreField::new"],
        module = "sand::systems",
        kind = "method",
        summary = "Defines an integer-backed player field stored in the named scoreboard objective.",
        context = "Defines an integer-backed player field stored in the named scoreboard objective. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(objective = "`objective` provides the objective used when defining an integer-backed player field stored in the named scoreboard objective."),
        returns = "A `ScoreField` defining an integer-backed player field stored in the named scoreboard objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(objective: & 'static str)  {\n    let score_field = sand::systems::player_data::ScoreField ::< T >::new(objective);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub const fn new(objective: &'static str) -> Self {
        Self {
            value: ScoreVar::new(objective),
            default: 0,
        }
    }

    /// Sets the score assigned when this player field is initialized.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::ScoreField::default",
        aliases = ["sand::prelude::ScoreField::default"],
        module = "sand::systems",
        kind = "method",
        summary = "Sets the score assigned when this player field is initialized.",
        context = "Sets the score assigned when this player field is initialized. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(value = "`value` provides the value being applied or compared used to set the score assigned when this player field is initialized."),
        returns = "The `ScoreField` value with the documented change applied to set the score assigned when this player field is initialized.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(score_field_value: sand::systems::player_data::ScoreField < T >, value: i32)  {\n    let updated_score_field = score_field_value.default(value);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub const fn default(mut self, value: i32) -> Self {
        self.default = value;
        self
    }

    /// Binds this field to one player's selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::ScoreField::of",
        aliases = ["sand::prelude::ScoreField::of"],
        module = "sand::systems",
        kind = "method",
        summary = "Binds this field to one player's selector.",
        context = "Binds this field to one player's selector. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(selector = "`selector` provides the Minecraft target selection used to bind this field to one player's selector."),
        returns = "The `ScoreRef < 'a , T >` value produced to bind this field to one player's selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<'a, T: 'static>(score_field_value: sand::systems::player_data::ScoreField < T >, selector: & str)  {\n    let of = score_field_value.of(selector);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn of<'a>(&'a self, selector: &str) -> ScoreRef<'a, T> {
        self.value.of(selector)
    }

    /// Returns the typed scoreboard variable underlying this field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::ScoreField::value",
        aliases = ["sand::prelude::ScoreField::value"],
        module = "sand::systems",
        kind = "method",
        summary = "Returns the typed scoreboard variable underlying this field.",
        context = "Returns the typed scoreboard variable underlying this field. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns the typed scoreboard variable underlying this field.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(score_field_value: &sand::systems::player_data::ScoreField < T >)  {\n    let value = score_field_value.value();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn value(&self) -> &ScoreVar<T> {
        &self.value
    }

    /// Returns the score assigned during player initialization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::ScoreField::default_value",
        aliases = ["sand::prelude::ScoreField::default_value"],
        module = "sand::systems",
        kind = "method",
        summary = "Returns the score assigned during player initialization.",
        context = "Returns the score assigned during player initialization. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns the score assigned during player initialization.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(score_field_value: &sand::systems::player_data::ScoreField < T >)  {\n    let default_value = score_field_value.default_value();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn default_value(&self) -> i32 {
        self.default
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::player_data::FlagField",
    aliases = ["sand::prelude::FlagField"],
    module = "sand::systems",
    summary = "Schema field handle backed by the existing [`Flag`].",
    context = "Schema field handle backed by the existing [`Flag`]. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::player_data::FlagField;",
    availability = ["Cargo feature: systems-player-data"],
)]
/// Schema field handle backed by the existing [`Flag`].
pub struct FlagField {
    value: Flag,
    default: bool,
}

impl FlagField {
    /// Defines a boolean player field stored in the named scoreboard objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::FlagField::new",
        aliases = ["sand::prelude::FlagField::new"],
        module = "sand::systems",
        kind = "method",
        summary = "Defines a boolean player field stored in the named scoreboard objective.",
        context = "Defines a boolean player field stored in the named scoreboard objective. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(objective = "`objective` provides the objective used when defining a boolean player field stored in the named scoreboard objective."),
        returns = "A `FlagField` defining a boolean player field stored in the named scoreboard objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective: & 'static str)  {\n    let flag_field = sand::systems::player_data::FlagField::new(objective);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub const fn new(objective: &'static str) -> Self {
        Self {
            value: Flag::new(objective),
            default: false,
        }
    }

    /// Sets the boolean assigned when this player field is initialized.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::FlagField::default",
        aliases = ["sand::prelude::FlagField::default"],
        module = "sand::systems",
        kind = "method",
        summary = "Sets the boolean assigned when this player field is initialized.",
        context = "Sets the boolean assigned when this player field is initialized. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(value = "`value` provides the value being applied or compared used to set the boolean assigned when this player field is initialized."),
        returns = "The `FlagField` value with the documented change applied to set the boolean assigned when this player field is initialized.",
        example = "use sand::prelude::*;\n\nfn demonstrate(flag_field_value: sand::systems::player_data::FlagField, value: bool)  {\n    let updated_flag_field = flag_field_value.default(value);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub const fn default(mut self, value: bool) -> Self {
        self.default = value;
        self
    }

    /// Binds this flag field to one player's selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::FlagField::of",
        aliases = ["sand::prelude::FlagField::of"],
        module = "sand::systems",
        kind = "method",
        summary = "Binds this flag field to one player's selector.",
        context = "Binds this flag field to one player's selector. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(selector = "`selector` provides the Minecraft target selection used to bind this flag field to one player's selector."),
        returns = "The `FlagRef < 'a >` value produced to bind this flag field to one player's selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<'a>(flag_field_value: sand::systems::player_data::FlagField, selector: & str)  {\n    let of = flag_field_value.of(selector);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn of<'a>(&'a self, selector: &str) -> FlagRef<'a> {
        self.value.of(selector)
    }

    /// Returns the typed scoreboard flag underlying this field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::FlagField::value",
        aliases = ["sand::prelude::FlagField::value"],
        module = "sand::systems",
        kind = "method",
        summary = "Returns the typed scoreboard flag underlying this field.",
        context = "Returns the typed scoreboard flag underlying this field. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns the typed scoreboard flag underlying this field.",
        example = "use sand::prelude::*;\n\nfn demonstrate(flag_field_value: &sand::systems::player_data::FlagField)  {\n    let value = flag_field_value.value();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn value(&self) -> &Flag {
        &self.value
    }

    /// Returns the flag assigned during player initialization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::FlagField::default_value",
        aliases = ["sand::prelude::FlagField::default_value"],
        module = "sand::systems",
        kind = "method",
        summary = "Returns the flag assigned during player initialization.",
        context = "Returns the flag assigned during player initialization. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns the flag assigned during player initialization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(flag_field_value: &sand::systems::player_data::FlagField)  {\n    let is_default_value = flag_field_value.default_value();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn default_value(&self) -> bool {
        self.default
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::player_data::TimerField",
    aliases = ["sand::prelude::TimerField"],
    module = "sand::systems",
    summary = "Schema field handle backed by the existing [`Timer`].",
    context = "Schema field handle backed by the existing [`Timer`]. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::player_data::TimerField;",
    availability = ["Cargo feature: systems-player-data"],
)]
/// Schema field handle backed by the existing [`Timer`].
pub struct TimerField {
    value: Timer,
}

impl TimerField {
    /// Defines a player timer with its scoreboard objective and duration.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::TimerField::new",
        aliases = ["sand::prelude::TimerField::new"],
        module = "sand::systems",
        kind = "method",
        summary = "Defines a player timer with its scoreboard objective and duration.",
        context = "Defines a player timer with its scoreboard objective and duration. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(objective = "`objective` provides the objective used when defining a player timer with its scoreboard objective and duration.", duration = "`duration` provides the Minecraft tick duration used to define a player timer with its scoreboard objective and duration."),
        returns = "A `TimerField` defining a player timer with its scoreboard objective and duration.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective: & 'static str, duration: sand::state::Ticks)  {\n    let timer_field = sand::systems::player_data::TimerField::new(objective, duration);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub const fn new(objective: &'static str, duration: Ticks) -> Self {
        Self {
            value: Timer::new(objective, duration),
        }
    }

    /// Binds this timer field to one player's selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::TimerField::of",
        aliases = ["sand::prelude::TimerField::of"],
        module = "sand::systems",
        kind = "method",
        summary = "Binds this timer field to one player's selector.",
        context = "Binds this timer field to one player's selector. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(selector = "`selector` provides the Minecraft target selection used to bind this timer field to one player's selector."),
        returns = "The `TimerFieldRef < 'a >` value produced to bind this timer field to one player's selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<'a>(timer_field_value: sand::systems::player_data::TimerField, selector: impl Into < String >)  {\n    let of = timer_field_value.of(selector);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn of<'a>(&'a self, selector: impl Into<String>) -> TimerFieldRef<'a> {
        TimerFieldRef {
            field: self,
            selector: selector.into(),
        }
    }

    /// Returns the typed timer underlying this field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::TimerField::value",
        aliases = ["sand::prelude::TimerField::value"],
        module = "sand::systems",
        kind = "method",
        summary = "Returns the typed timer underlying this field.",
        context = "Returns the typed timer underlying this field. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns the typed timer underlying this field.",
        example = "use sand::prelude::*;\n\nfn demonstrate(timer_field_value: &sand::systems::player_data::TimerField)  {\n    let value = timer_field_value.value();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn value(&self) -> &Timer {
        &self.value
    }
}

#[doc = "Configures timer field ref in the player data gameplay system."]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::player_data::TimerFieldRef",
    aliases = ["sand::prelude::TimerFieldRef"],
    module = "sand::systems",
    summary = "Configures timer field ref in the player data gameplay system.",
    context = "Configures timer field ref in the player data gameplay system. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::player_data::TimerFieldRef;",
    availability = ["Cargo feature: systems-player-data"],
)]
pub struct TimerFieldRef<'a> {
    field: &'a TimerField,
    selector: String,
}

impl TimerFieldRef<'_> {
    /// Renders the command that starts this player's timer.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::TimerFieldRef::start",
        aliases = ["sand::prelude::TimerFieldRef::start"],
        module = "sand::systems",
        kind = "method",
        summary = "Renders the command that starts this player's timer.",
        context = "Renders the command that starts this player's timer. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The rendered Minecraft command text produced to render the command that starts this player's timer.",
        example = "use sand::prelude::*;\n\nfn demonstrate(timer_field_ref_value: &sand::systems::player_data::TimerFieldRef < '_ >)  {\n    let command = timer_field_ref_value.start();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn start(&self) -> String {
        self.field.value.start(&self.selector)
    }

    /// Renders the command that resets this player's timer.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::TimerFieldRef::reset",
        aliases = ["sand::prelude::TimerFieldRef::reset"],
        module = "sand::systems",
        kind = "method",
        summary = "Renders the command that resets this player's timer.",
        context = "Renders the command that resets this player's timer. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The rendered Minecraft command text produced to render the command that resets this player's timer.",
        example = "use sand::prelude::*;\n\nfn demonstrate(timer_field_ref_value: &sand::systems::player_data::TimerFieldRef < '_ >)  {\n    let command = timer_field_ref_value.reset();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn reset(&self) -> String {
        self.field.value.reset(&self.selector)
    }

    /// Builds the typed condition for active.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::TimerFieldRef::active",
        aliases = ["sand::prelude::TimerFieldRef::active"],
        module = "sand::systems",
        kind = "method",
        summary = "Builds the typed condition for active.",
        context = "Builds the typed condition for active. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Condition` value produced to build the typed condition for active.",
        example = "use sand::prelude::*;\n\nfn demonstrate(timer_field_ref_value: &sand::systems::player_data::TimerFieldRef < '_ >)  {\n    let active = timer_field_ref_value.active();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn active(&self) -> Condition {
        self.field.value.active(&self.selector)
    }

    /// Builds the typed condition for expired.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::TimerFieldRef::expired",
        aliases = ["sand::prelude::TimerFieldRef::expired"],
        module = "sand::systems",
        kind = "method",
        summary = "Builds the typed condition for expired.",
        context = "Builds the typed condition for expired. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Condition` value produced to build the typed condition for expired.",
        example = "use sand::prelude::*;\n\nfn demonstrate(timer_field_ref_value: &sand::systems::player_data::TimerFieldRef < '_ >)  {\n    let expired = timer_field_ref_value.expired();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn expired(&self) -> Condition {
        self.field.value.expired(&self.selector)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::player_data::CooldownField",
    aliases = ["sand::prelude::CooldownField"],
    module = "sand::systems",
    summary = "Schema field handle backed by the existing [`Cooldown`].",
    context = "Schema field handle backed by the existing [`Cooldown`]. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::player_data::CooldownField;",
    availability = ["Cargo feature: systems-player-data"],
)]
/// Schema field handle backed by the existing [`Cooldown`].
pub struct CooldownField {
    value: Cooldown,
}

impl CooldownField {
    /// Defines a player cooldown with its scoreboard objective and duration.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::CooldownField::new",
        aliases = ["sand::prelude::CooldownField::new"],
        module = "sand::systems",
        kind = "method",
        summary = "Defines a player cooldown with its scoreboard objective and duration.",
        context = "Defines a player cooldown with its scoreboard objective and duration. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(objective = "`objective` provides the objective used when defining a player cooldown with its scoreboard objective and duration.", duration = "`duration` provides the Minecraft tick duration used to define a player cooldown with its scoreboard objective and duration."),
        returns = "A `CooldownField` defining a player cooldown with its scoreboard objective and duration.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective: & 'static str, duration: sand::state::Ticks)  {\n    let cooldown_field = sand::systems::player_data::CooldownField::new(objective, duration);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub const fn new(objective: &'static str, duration: Ticks) -> Self {
        Self {
            value: Cooldown::new(objective, duration),
        }
    }

    /// Binds this cooldown field to one player's selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::CooldownField::of",
        aliases = ["sand::prelude::CooldownField::of"],
        module = "sand::systems",
        kind = "method",
        summary = "Binds this cooldown field to one player's selector.",
        context = "Binds this cooldown field to one player's selector. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(selector = "`selector` provides the Minecraft target selection used to bind this cooldown field to one player's selector."),
        returns = "The `CooldownFieldRef < 'a >` value produced to bind this cooldown field to one player's selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<'a>(cooldown_field_value: sand::systems::player_data::CooldownField, selector: impl Into < String >)  {\n    let of = cooldown_field_value.of(selector);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn of<'a>(&'a self, selector: impl Into<String>) -> CooldownFieldRef<'a> {
        CooldownFieldRef {
            field: self,
            selector: selector.into(),
        }
    }

    /// Returns the typed cooldown underlying this field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::CooldownField::value",
        aliases = ["sand::prelude::CooldownField::value"],
        module = "sand::systems",
        kind = "method",
        summary = "Returns the typed cooldown underlying this field.",
        context = "Returns the typed cooldown underlying this field. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns the typed cooldown underlying this field.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_field_value: &sand::systems::player_data::CooldownField)  {\n    let value = cooldown_field_value.value();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn value(&self) -> &Cooldown {
        &self.value
    }
}

#[doc = "Configures cooldown field ref in the player data gameplay system."]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::player_data::CooldownFieldRef",
    aliases = ["sand::prelude::CooldownFieldRef"],
    module = "sand::systems",
    summary = "Configures cooldown field ref in the player data gameplay system.",
    context = "Configures cooldown field ref in the player data gameplay system. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::player_data::CooldownFieldRef;",
    availability = ["Cargo feature: systems-player-data"],
)]
pub struct CooldownFieldRef<'a> {
    field: &'a CooldownField,
    selector: String,
}

impl CooldownFieldRef<'_> {
    /// Renders the command that starts this player's cooldown.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::CooldownFieldRef::start",
        aliases = ["sand::prelude::CooldownFieldRef::start"],
        module = "sand::systems",
        kind = "method",
        summary = "Renders the command that starts this player's cooldown.",
        context = "Renders the command that starts this player's cooldown. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The rendered Minecraft command text produced to render the command that starts this player's cooldown.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_field_ref_value: &sand::systems::player_data::CooldownFieldRef < '_ >)  {\n    let command = cooldown_field_ref_value.start();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn start(&self) -> String {
        self.field.value.start(&self.selector)
    }

    /// Renders the command that stops this player's cooldown.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::CooldownFieldRef::stop",
        aliases = ["sand::prelude::CooldownFieldRef::stop"],
        module = "sand::systems",
        kind = "method",
        summary = "Renders the command that stops this player's cooldown.",
        context = "Renders the command that stops this player's cooldown. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The rendered Minecraft command text produced to render the command that stops this player's cooldown.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_field_ref_value: &sand::systems::player_data::CooldownFieldRef < '_ >)  {\n    let command = cooldown_field_ref_value.stop();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn stop(&self) -> String {
        self.field.value.stop(&self.selector)
    }

    /// Builds the typed condition for ready.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::CooldownFieldRef::ready",
        aliases = ["sand::prelude::CooldownFieldRef::ready"],
        module = "sand::systems",
        kind = "method",
        summary = "Builds the typed condition for ready.",
        context = "Builds the typed condition for ready. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Condition` value produced to build the typed condition for ready.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_field_ref_value: &sand::systems::player_data::CooldownFieldRef < '_ >)  {\n    let ready = cooldown_field_ref_value.ready();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn ready(&self) -> Condition {
        self.field.value.ready(&self.selector)
    }

    /// Builds the typed condition for active.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::CooldownFieldRef::active",
        aliases = ["sand::prelude::CooldownFieldRef::active"],
        module = "sand::systems",
        kind = "method",
        summary = "Builds the typed condition for active.",
        context = "Builds the typed condition for active. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Condition` value produced to build the typed condition for active.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_field_ref_value: &sand::systems::player_data::CooldownFieldRef < '_ >)  {\n    let active = cooldown_field_ref_value.active();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn active(&self) -> Condition {
        self.field.value.active(&self.selector)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::player_data::GameStateField",
    aliases = ["sand::prelude::GameStateField"],
    module = "sand::systems",
    summary = "Schema field handle backed by the existing enum-backed [`GameState`].",
    context = "Schema field handle backed by the existing enum-backed [`GameState`]. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::player_data::GameStateField;",
    availability = ["Cargo feature: systems-player-data"],
)]
/// Schema field handle backed by the existing enum-backed [`GameState`].
pub struct GameStateField<S: TypedGameState> {
    value: GameState<S>,
}

impl<S: TypedGameState> GameStateField<S> {
    /// Defines an enum-like player state stored in the named scoreboard objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::GameStateField::new",
        aliases = ["sand::prelude::GameStateField::new"],
        module = "sand::systems",
        kind = "method",
        summary = "Defines an enum-like player state stored in the named scoreboard objective.",
        context = "Defines an enum-like player state stored in the named scoreboard objective. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(objective = "`objective` provides the objective used when defining an enum-like player state stored in the named scoreboard objective."),
        returns = "A `GameStateField` defining an enum-like player state stored in the named scoreboard objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate<S : sand::state::TypedGameState + 'static>(objective: & 'static str)  {\n    let game_state_field = sand::systems::player_data::GameStateField ::< S >::new(objective);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub const fn new(objective: &'static str) -> Self {
        Self {
            value: GameState::new(objective),
        }
    }

    /// Defines a player state with an explicit initial encoded score.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::GameStateField::with_default_score",
        aliases = ["sand::prelude::GameStateField::with_default_score"],
        module = "sand::systems",
        kind = "method",
        summary = "Defines a player state with an explicit initial encoded score.",
        context = "Defines a player state with an explicit initial encoded score. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(objective = "`objective` provides the objective used when defining a player state with an explicit initial encoded score.", default = "`default` provides the default used when defining a player state with an explicit initial encoded score."),
        returns = "A `GameStateField` defining a player state with an explicit initial encoded score.",
        example = "use sand::prelude::*;\n\nfn demonstrate<S : sand::state::TypedGameState + 'static>(objective: & 'static str, default: i32)  {\n    let game_state_field = sand::systems::player_data::GameStateField ::< S >::with_default_score(objective, default);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub const fn with_default_score(objective: &'static str, default: i32) -> Self {
        Self {
            value: GameState::with_default_score(objective, default),
        }
    }

    /// Binds this game-state field to one player's selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::GameStateField::of",
        aliases = ["sand::prelude::GameStateField::of"],
        module = "sand::systems",
        kind = "method",
        summary = "Binds this game-state field to one player's selector.",
        context = "Binds this game-state field to one player's selector. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(selector = "`selector` provides the Minecraft target selection used to bind this game-state field to one player's selector."),
        returns = "The `GameStateRef < 'a , S >` value produced to bind this game-state field to one player's selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<'a, S : sand::state::TypedGameState + 'static>(game_state_field_value: sand::systems::player_data::GameStateField < S >, selector: & str)  {\n    let of = game_state_field_value.of(selector);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn of<'a>(&'a self, selector: &str) -> GameStateRef<'a, S> {
        self.value.of(selector)
    }

    /// Returns the typed game-state variable underlying this field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::GameStateField::value",
        aliases = ["sand::prelude::GameStateField::value"],
        module = "sand::systems",
        kind = "method",
        summary = "Returns the typed game-state variable underlying this field.",
        context = "Returns the typed game-state variable underlying this field. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns the typed game-state variable underlying this field.",
        example = "use sand::prelude::*;\n\nfn demonstrate<S : sand::state::TypedGameState + 'static>(game_state_field_value: &sand::systems::player_data::GameStateField < S >)  {\n    let value = game_state_field_value.value();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn value(&self) -> &GameState<S> {
        &self.value
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::player_data::GlobalStorageField",
    aliases = ["sand::prelude::GlobalStorageField"],
    module = "sand::systems",
    summary = "Explicit global-storage field handle. It never claims player scoping.",
    context = "Explicit global-storage field handle. It never claims player scoping. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::player_data::GlobalStorageField;",
    availability = ["Cargo feature: systems-player-data"],
)]
/// Explicit global-storage field handle. It never claims player scoping.
pub struct GlobalStorageField<Schema, T> {
    value: StorageField<Schema, T>,
    marker: PhantomData<fn() -> (Schema, T)>,
}

impl<Schema, T> GlobalStorageField<Schema, T> {
    /// Defines a global typed field within the supplied command-storage schema.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::GlobalStorageField::new",
        aliases = ["sand::prelude::GlobalStorageField::new"],
        module = "sand::systems",
        kind = "method",
        summary = "Defines a global typed field within the supplied command-storage schema.",
        context = "Defines a global typed field within the supplied command-storage schema. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(schema = "`schema` provides the schema used when defining a global typed field within the supplied command-storage schema.", field = "`field` provides the field used when defining a global typed field within the supplied command-storage schema."),
        returns = "A `GlobalStorageField` defining a global typed field within the supplied command-storage schema.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(schema: & sand::data::StorageSchema < Schema >, field: & 'static str)  {\n    let global_storage_field = sand::systems::player_data::GlobalStorageField ::< Schema , T >::new(schema, field);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub const fn new(schema: &StorageSchema<Schema>, field: &'static str) -> Self {
        Self {
            value: schema.field(field),
            marker: PhantomData,
        }
    }

    /// Returns the typed NBT reference for this global storage field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::GlobalStorageField::nbt",
        aliases = ["sand::prelude::GlobalStorageField::nbt"],
        module = "sand::systems",
        kind = "method",
        summary = "Returns the typed NBT reference for this global storage field.",
        context = "Returns the typed NBT reference for this global storage field. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns the typed NBT reference for this global storage field.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(global_storage_field_value: &sand::systems::player_data::GlobalStorageField < Schema , T >)  {\n    let nbt = global_storage_field_value.nbt();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn nbt(&self) -> NbtRef<T> {
        self.value.path()
    }

    /// Returns the schema field underlying this global storage declaration.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::GlobalStorageField::value",
        aliases = ["sand::prelude::GlobalStorageField::value"],
        module = "sand::systems",
        kind = "method",
        summary = "Returns the schema field underlying this global storage declaration.",
        context = "Returns the schema field underlying this global storage declaration. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns the schema field underlying this global storage declaration.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(global_storage_field_value: &sand::systems::player_data::GlobalStorageField < Schema , T >)  {\n    let value = global_storage_field_value.value();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::player_data::PlayerDataSchema",
    aliases = ["sand::prelude::PlayerDataSchema", "sand::prelude::PlayerSchema", "sand::systems::player_data::PlayerSchema"],
    module = "sand::systems",
    summary = "A mixed per-player data bundle: scoreboard fields, flags, cooldowns, and attached storage schema references.",
    context = "A mixed per-player data bundle: scoreboard fields, flags, cooldowns, and attached storage schema references. Build with the chained builder methods.  Call [`define_all`](PlayerDataSchema::define_all) in your load function and [`init_player`](PlayerDataSchema::init_player) in join handlers.  See the [module docs](self) for naming rules and per-player storage limitations.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::player_data::PlayerDataSchema;",
    availability = ["Cargo feature: systems-player-data"],
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::new",
        aliases = ["sand::prelude::PlayerDataSchema::new", "sand::prelude::PlayerSchema::new", "sand::systems::player_data::PlayerSchema::new"],
        module = "sand::systems",
        kind = "method",
        summary = "Create an empty schema with the given human label.",
        context = "Create an empty schema with the given human label. The label is for documentation/introspection only — it does not prefix scoreboard objectives.  Two schemas can share a label without conflict, and two schemas with the same-named `ScoreVar` will share an objective (which is often intentional).",
        minecraft = "The label is for documentation/introspection only — it does not prefix scoreboard objectives.  Two schemas can share a label without conflict, and two schemas with the same-named `ScoreVar` will share an objective (which is often intentional).",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(namespace = "`namespace` is used when creating an empty schema with the given human label."),
        returns = "A `PlayerDataSchema` representing an empty schema with the given human label.",
        example = "use sand::prelude::*;\n\nfn demonstrate(namespace: & 'static str)  {\n    let player_data_schema = sand::systems::player_data::PlayerDataSchema::new(namespace);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub const fn new(namespace: &'static str) -> Self {
        Self {
            namespace,
            fields: Vec::new(),
            storage_schemas: Vec::new(),
        }
    }

    /// The human label passed to [`new`](Self::new).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::name",
        aliases = ["sand::prelude::PlayerDataSchema::name", "sand::prelude::PlayerSchema::name", "sand::systems::player_data::PlayerSchema::name"],
        module = "sand::systems",
        kind = "method",
        summary = "The human label passed to [`new`](Self::new).",
        context = "The human label passed to [`new`](Self::new). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The string value produced to use the human label passed to [`new`](Self::new).",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: &sand::systems::player_data::PlayerDataSchema)  {\n    let name = player_data_schema_value.name();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn name(&self) -> &str {
        self.namespace
    }

    // ── Scoreboard fields ─────────────────────────────────────────────────────

    /// Register a `ScoreVar` with a default value for new players.
    ///
    /// The objective name comes from `var.objective_name()`, not from the
    /// schema namespace.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::score",
        aliases = ["sand::prelude::PlayerDataSchema::score", "sand::prelude::PlayerSchema::score", "sand::systems::player_data::PlayerSchema::score"],
        module = "sand::systems",
        kind = "method",
        summary = "Register a `ScoreVar` with a default value for new players.",
        context = "Register a `ScoreVar` with a default value for new players. The objective name comes from `var.objective_name()`, not from the schema namespace.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(var = "`var` is used to register a `ScoreVar` with a default value for new players.", default = "`default` is used to register a `ScoreVar` with a default value for new players."),
        returns = "The `PlayerDataSchema` value with the documented change applied to register a `ScoreVar` with a default value for new players.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(player_data_schema_value: sand::systems::player_data::PlayerDataSchema, var: & sand::state::ScoreVar < T >, default: i32)  {\n    let updated_player_data_schema = player_data_schema_value.score::<T>(var, default);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn score<T>(mut self, var: &ScoreVar<T>, default: i32) -> Self {
        self.fields.push(FieldInit::Score {
            obj: var.objective_name(),
            default,
        });
        self
    }

    /// Registers an integer score field for initialization in this player-data schema.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::score_field",
        aliases = ["sand::prelude::PlayerDataSchema::score_field", "sand::prelude::PlayerSchema::score_field", "sand::systems::player_data::PlayerSchema::score_field"],
        module = "sand::systems",
        kind = "method",
        summary = "Registers an integer score field for initialization in this player-data schema.",
        context = "Registers an integer score field for initialization in this player-data schema. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(field = "`field` is used to register an integer score field for initialization in this player-data schema."),
        returns = "The `PlayerDataSchema` value with the documented change applied to register an integer score field for initialization in this player-data schema.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(player_data_schema_value: sand::systems::player_data::PlayerDataSchema, field: & sand::systems::player_data::ScoreField < T >)  {\n    let updated_player_data_schema = player_data_schema_value.score_field::<T>(field);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn score_field<T>(self, field: &ScoreField<T>) -> Self {
        self.score(field.value(), field.default_value())
    }

    /// Register a `Flag` with a default boolean value for new players.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::flag",
        aliases = ["sand::prelude::PlayerDataSchema::flag", "sand::prelude::PlayerSchema::flag", "sand::systems::player_data::PlayerSchema::flag"],
        module = "sand::systems",
        kind = "method",
        summary = "Register a `Flag` with a default boolean value for new players.",
        context = "Register a `Flag` with a default boolean value for new players. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(flag = "`flag` is used to register a `Flag` with a default boolean value for new players.", default = "`default` provides the switch that enables or disables the behavior used to register a `Flag` with a default boolean value for new players."),
        returns = "The `PlayerDataSchema` value with the documented change applied to register a `Flag` with a default boolean value for new players.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: sand::systems::player_data::PlayerDataSchema, flag: & sand::state::Flag, default: bool)  {\n    let updated_player_data_schema = player_data_schema_value.flag(flag, default);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn flag(mut self, flag: &Flag, default: bool) -> Self {
        self.fields.push(FieldInit::Flag {
            obj: flag.objective_name(),
            default,
        });
        self
    }

    /// Registers a boolean flag field for initialization in this player-data schema.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::flag_field",
        aliases = ["sand::prelude::PlayerDataSchema::flag_field", "sand::prelude::PlayerSchema::flag_field", "sand::systems::player_data::PlayerSchema::flag_field"],
        module = "sand::systems",
        kind = "method",
        summary = "Registers a boolean flag field for initialization in this player-data schema.",
        context = "Registers a boolean flag field for initialization in this player-data schema. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(field = "`field` is used to register a boolean flag field for initialization in this player-data schema."),
        returns = "The `PlayerDataSchema` value with the documented change applied to register a boolean flag field for initialization in this player-data schema.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: sand::systems::player_data::PlayerDataSchema, field: & sand::systems::player_data::FlagField)  {\n    let updated_player_data_schema = player_data_schema_value.flag_field(field);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::timer",
        aliases = ["sand::prelude::PlayerDataSchema::timer", "sand::prelude::PlayerSchema::timer", "sand::systems::player_data::PlayerSchema::timer"],
        module = "sand::systems",
        kind = "method",
        summary = "Register a [`Timer`] objective (define only; no per-player default).",
        context = "Register a [`Timer`] objective (define only; no per-player default). This method only defines/registers the timer's scoreboard objective. It does not automatically start ticks or wire lifecycle events. To actually use the timer in gameplay, you must separately: - Call the timer's tick methods during server ticks (e.g., in a `#[tick]` function). - Manage timer lifecycle wiring (e.g., starting timers in events). See [`Timer`] for tick/lifecycle APIs.",
        minecraft = "This method only defines/registers the timer's scoreboard objective. It does not automatically start ticks or wire lifecycle events.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(timer = "`timer` is used to register a [`Timer`] objective (define only; no per-player default)."),
        returns = "The `PlayerDataSchema` value with the documented change applied to register a [`Timer`] objective (define only; no per-player default).",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: sand::systems::player_data::PlayerDataSchema, timer: & sand::state::Timer)  {\n    let updated_player_data_schema = player_data_schema_value.timer(timer);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn timer(mut self, timer: &Timer) -> Self {
        self.fields.push(FieldInit::TimerObj {
            obj: timer.objective_name(),
        });
        self
    }

    /// Registers a timer field for lifecycle setup in this player-data schema.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::timer_field",
        aliases = ["sand::prelude::PlayerDataSchema::timer_field", "sand::prelude::PlayerSchema::timer_field", "sand::systems::player_data::PlayerSchema::timer_field"],
        module = "sand::systems",
        kind = "method",
        summary = "Registers a timer field for lifecycle setup in this player-data schema.",
        context = "Registers a timer field for lifecycle setup in this player-data schema. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(field = "`field` is used to register a timer field for lifecycle setup in this player-data schema."),
        returns = "The `PlayerDataSchema` value with the documented change applied to register a timer field for lifecycle setup in this player-data schema.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: sand::systems::player_data::PlayerDataSchema, field: & sand::systems::player_data::TimerField)  {\n    let updated_player_data_schema = player_data_schema_value.timer_field(field);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::cooldown",
        aliases = ["sand::prelude::PlayerDataSchema::cooldown", "sand::prelude::PlayerSchema::cooldown", "sand::systems::player_data::PlayerSchema::cooldown"],
        module = "sand::systems",
        kind = "method",
        summary = "Register a `Cooldown` objective (define only; no per-player default).",
        context = "Register a `Cooldown` objective (define only; no per-player default). This method only defines/registers the cooldown's scoreboard objective. It does not automatically manage cooldown state or tick them down. To actually use the cooldown in gameplay, you must separately: - Call the cooldown's tick methods during server ticks (e.g., in a `#[tick]` function). - Manage cooldown lifecycle wiring (e.g., starting cooldowns on ability use). See [`Cooldown`] for tick/lifecycle APIs.",
        minecraft = "This method only defines/registers the cooldown's scoreboard objective. It does not automatically manage cooldown state or tick them down.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(cd = "`cd` is used to register a `Cooldown` objective (define only; no per-player default)."),
        returns = "The `PlayerDataSchema` value with the documented change applied to register a `Cooldown` objective (define only; no per-player default).",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: sand::systems::player_data::PlayerDataSchema, cd: & sand::state::Cooldown)  {\n    let updated_player_data_schema = player_data_schema_value.cooldown(cd);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn cooldown(mut self, cd: &Cooldown) -> Self {
        self.fields.push(FieldInit::CooldownObj {
            obj: cd.objective_name(),
        });
        self
    }

    /// Registers a cooldown field for lifecycle setup in this player-data schema.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::cooldown_field",
        aliases = ["sand::prelude::PlayerDataSchema::cooldown_field", "sand::prelude::PlayerSchema::cooldown_field", "sand::systems::player_data::PlayerSchema::cooldown_field"],
        module = "sand::systems",
        kind = "method",
        summary = "Registers a cooldown field for lifecycle setup in this player-data schema.",
        context = "Registers a cooldown field for lifecycle setup in this player-data schema. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(field = "`field` is used to register a cooldown field for lifecycle setup in this player-data schema."),
        returns = "The `PlayerDataSchema` value with the documented change applied to register a cooldown field for lifecycle setup in this player-data schema.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: sand::systems::player_data::PlayerDataSchema, field: & sand::systems::player_data::CooldownField)  {\n    let updated_player_data_schema = player_data_schema_value.cooldown_field(field);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn cooldown_field(self, field: &CooldownField) -> Self {
        self.cooldown(field.value())
    }

    /// Registers a typed game-state field for initialization in this player-data schema.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::game_state",
        aliases = ["sand::prelude::PlayerDataSchema::game_state", "sand::prelude::PlayerSchema::game_state", "sand::systems::player_data::PlayerSchema::game_state"],
        module = "sand::systems",
        kind = "method",
        summary = "Registers a typed game-state field for initialization in this player-data schema.",
        context = "Registers a typed game-state field for initialization in this player-data schema. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(field = "`field` is used to register a typed game-state field for initialization in this player-data schema."),
        returns = "The `PlayerDataSchema` value with the documented change applied to register a typed game-state field for initialization in this player-data schema.",
        example = "use sand::prelude::*;\n\nfn demonstrate<S : sand::state::TypedGameState + 'static>(player_data_schema_value: sand::systems::player_data::PlayerDataSchema, field: & sand::systems::player_data::GameStateField < S >)  {\n    let updated_player_data_schema = player_data_schema_value.game_state::<S>(field);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn game_state<S: TypedGameState>(mut self, field: &GameStateField<S>) -> Self {
        self.fields.push(FieldInit::StateObj {
            obj: field.value().objective_name(),
            default: field.value().default_score(),
        });
        self
    }

    /// Attach one explicit global-storage handle for schema introspection.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::global_storage_field",
        aliases = ["sand::prelude::PlayerDataSchema::global_storage_field", "sand::prelude::PlayerSchema::global_storage_field", "sand::systems::player_data::PlayerSchema::global_storage_field"],
        module = "sand::systems",
        kind = "method",
        summary = "Attach one explicit global-storage handle for schema introspection.",
        context = "Attach one explicit global-storage handle for schema introspection. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(field = "`field` is used to attach one explicit global-storage handle for schema introspection."),
        returns = "The `PlayerDataSchema` value with the documented change applied to attach one explicit global-storage handle for schema introspection.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(player_data_schema_value: sand::systems::player_data::PlayerDataSchema, field: & sand::systems::player_data::GlobalStorageField < Schema , T >)  {\n    let updated_player_data_schema = player_data_schema_value.global_storage_field::<Schema, T>(field);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::storage",
        aliases = ["sand::prelude::PlayerDataSchema::storage", "sand::prelude::PlayerSchema::storage", "sand::systems::player_data::PlayerSchema::storage"],
        module = "sand::systems",
        kind = "method",
        summary = "Attach a [`StorageSchema`] to this player schema for tracking and documentation.",
        context = "Attach a [`StorageSchema`] to this player schema for tracking and documentation. Storage schemas attached here are tracked for introspection via [`storage_locations`](Self::storage_locations).  No commands are emitted for storage schemas by [`define_all`](Self::define_all) — Minecraft NBT storage paths require no explicit definition. Minecraft `data storage` is a global namespace, not per-player. Attaching a storage schema to `PlayerDataSchema` does not automatically key storage by player UUID or name. If you need per-player compound data, the common approaches are:",
        minecraft = "Minecraft `data storage` is a global namespace, not per-player. Attaching a storage schema to `PlayerDataSchema` does not automatically key storage by player UUID or name.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(schema = "`schema` is used to attach a [`StorageSchema`] to this player schema for tracking and documentation."),
        returns = "The `PlayerDataSchema` value with the documented change applied to attach a [`StorageSchema`] to this player schema for tracking and documentation.",
        example = "pub struct PackConfig { pub max_mana: i32 }\nlet schema = PlayerDataSchema::new(\"magic\")\n.score(&MANA, 100)\n.storage(PackConfig::SCHEMA); // global config, not per-player",
        availability = ["Cargo feature: systems-player-data"],
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::define_all",
        aliases = ["sand::prelude::PlayerDataSchema::define_all", "sand::prelude::PlayerSchema::define_all", "sand::systems::player_data::PlayerSchema::define_all"],
        module = "sand::systems",
        kind = "method",
        summary = "Commands to define all scoreboard objectives (for your load function).",
        context = "Commands to define all scoreboard objectives (for your load function). Storage schemas do not generate commands — Minecraft NBT storage needs no explicit definition. The emitted `scoreboard objectives add` commands are idempotent: if an objective already exists, Minecraft prints a warning but does not abort. It is safe to call `define_all()` more than once or to run its output in every reload.",
        minecraft = "Storage schemas do not generate commands — Minecraft NBT storage needs no explicit definition.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The ordered values produced to command to define all scoreboard objectives (for your load function).",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: &sand::systems::player_data::PlayerDataSchema)  {\n    let values = player_data_schema_value.define_all();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::init_player",
        aliases = ["sand::prelude::PlayerDataSchema::init_player", "sand::prelude::PlayerSchema::init_player", "sand::systems::player_data::PlayerSchema::init_player"],
        module = "sand::systems",
        kind = "method",
        summary = "Commands to initialize a new player's scores to their defaults.",
        context = "Commands to initialize a new player's scores to their defaults. Each command uses `unless score … matches -2147483648..` so it is a no-op for players who already have scores (reconnects, respawns, etc.). Cooldowns have no default value, so they are skipped here.  Storage schemas are not affected by this method. Compatibility/raw path: `selector` is an unvalidated string, interpolated directly into generated commands. Prefer [`PlayerDataSchema::try_init_player`] in normal code — see [#146](https://github.com/ThatOneToast/sand/issues/146).",
        minecraft = "Each command uses `unless score … matches -2147483648..` so it is a no-op for players who already have scores (reconnects, respawns, etc.).",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(selector = "Compatibility/raw path: `selector` is an unvalidated string, interpolated directly into generated commands. Prefer [`PlayerDataSchema::try_init_player`] in normal code — see [#146](https://github.com/ThatOneToast/sand/issues/146)."),
        returns = "The ordered values produced to command to initialize a new player's scores to their defaults.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: &sand::systems::player_data::PlayerDataSchema, selector: & str)  {\n    let values = player_data_schema_value.init_player(selector);\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::try_init_player",
        aliases = ["sand::prelude::PlayerDataSchema::try_init_player", "sand::prelude::PlayerSchema::try_init_player", "sand::systems::player_data::PlayerSchema::try_init_player"],
        module = "sand::systems",
        kind = "method",
        summary = "Validated counterpart to [`PlayerDataSchema::init_player`] — takes a typed [`sand::command::ScoreHolder`] and validates it before generating commands, instead of interpolating an unvalidated selector string.",
        context = "Validated counterpart to [`PlayerDataSchema::init_player`] — takes a typed [`sand::command::ScoreHolder`] and validates it before generating commands, instead of interpolating an unvalidated selector string. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(holder = "`holder` sets the holder for validated counterpart to [`PlayerDataSchema::init_player`] — takes a typed [`sand::command::ScoreHolder`] and validates it before generating commands, instead of interpolating an unvalidated selector string."),
        returns = "The `sand :: command :: CommandResult < Vec < String > >` value produced to use validated counterpart to [`PlayerDataSchema::init_player`] — takes a typed [`sand::command::ScoreHolder`] and validates it before generating commands, instead of interpolating an unvalidated selector string.",
        example = "use sand::systems::player_data::PlayerDataSchema;\nuse sand::state::ScoreVar;\nuse sand::command::ScoreHolder;\nstatic MANA: ScoreVar<i32> = ScoreVar::new(\"mana\");\nlet schema = PlayerDataSchema::new(\"player\").score(&MANA, 100);\nassert!(schema.try_init_player(ScoreHolder::self_()).is_ok());\nassert!(schema.try_init_player(ScoreHolder::fake(\"bad holder\")).is_err());",
        availability = ["Cargo feature: systems-player-data"],
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::storage_locations",
        aliases = ["sand::prelude::PlayerDataSchema::storage_locations", "sand::prelude::PlayerSchema::storage_locations", "sand::systems::player_data::PlayerSchema::storage_locations"],
        module = "sand::systems",
        kind = "method",
        summary = "Returns descriptors for all attached storage schemas.",
        context = "Returns descriptors for all attached storage schemas. Each descriptor exposes the storage resource location string and the NBT root path.  Use this for debugging, code generation, or building documentation.",
        minecraft = "Each descriptor exposes the storage resource location string and the NBT root path.  Use this for debugging, code generation, or building documentation.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns descriptors for all attached storage schemas.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: &sand::systems::player_data::PlayerDataSchema)  {\n    let storage_locations = player_data_schema_value.storage_locations();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn storage_locations(&self) -> &[StorageDescriptor] {
        &self.storage_schemas
    }

    /// `true` if at least one storage schema has been attached.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::has_storage",
        aliases = ["sand::prelude::PlayerDataSchema::has_storage", "sand::prelude::PlayerSchema::has_storage", "sand::systems::player_data::PlayerSchema::has_storage"],
        module = "sand::systems",
        kind = "method",
        summary = "`true` if at least one storage schema has been attached.",
        context = "`true` if at least one storage schema has been attached. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "`true` when the documented condition holds to emit the documented `true` if at least one storage schema has been attached form; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: &sand::systems::player_data::PlayerDataSchema)  {\n    let is_has_storage = player_data_schema_value.has_storage();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn has_storage(&self) -> bool {
        !self.storage_schemas.is_empty()
    }

    /// The number of registered scoreboard-style fields
    /// (score + flag + timer + cooldown).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::player_data::PlayerDataSchema::scoreboard_field_count",
        aliases = ["sand::prelude::PlayerDataSchema::scoreboard_field_count", "sand::prelude::PlayerSchema::scoreboard_field_count", "sand::systems::player_data::PlayerSchema::scoreboard_field_count"],
        module = "sand::systems",
        kind = "method",
        summary = "The number of registered scoreboard-style fields (score + flag + timer + cooldown).",
        context = "The number of registered scoreboard-style fields (score + flag + timer + cooldown). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `usize` value produced to use the number of registered scoreboard-style fields (score + flag + timer + cooldown).",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_data_schema_value: &sand::systems::player_data::PlayerDataSchema)  {\n    let scoreboard_field_count = player_data_schema_value.scoreboard_field_count();\n}",
        availability = ["Cargo feature: systems-player-data"],
    )]
    pub fn scoreboard_field_count(&self) -> usize {
        self.fields.len()
    }
}

/// Compatibility alias for the earlier, shorter player-schema name.
pub use PlayerDataSchema as PlayerSchema;

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
