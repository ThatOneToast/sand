//! Typed scoreboard objective — a named integer counter in Minecraft.
//!
//! An [`Objective`] represents a scoreboard objective and acts as the bridge
//! between [`Storage`] (which holds arbitrary NBT at runtime) and the commands
//! that need a concrete numeric value (ability cooldowns, damage amounts, etc.).
//!
//! # Quick start
//!
//! ```rust,ignore
//! use sand_core::cmd::{Objective, Storage, ScoreHolder, Selector};
//!
//! static INFERNO_DMG:  Objective = Objective::new("inferno_dmg");
//! static PLAYERS:      Storage   = Storage::per_player("my_pack:players");
//!
//! // Load a stored ability damage value into the objective for @s
//! INFERNO_DMG.load_from(ScoreHolder::self_(), &PLAYERS, "uuid.ability_damage")
//! // → execute store result score @s inferno_dmg
//! //       run data get storage my_pack:players uuid.ability_damage
//!
//! // For float values (e.g. 3.5 stored → 35 in the score)
//! INFERNO_DMG.load_from_scaled(ScoreHolder::self_(), &PLAYERS, "uuid.ability_damage", 10.0)
//!
//! // Direct manipulation
//! INFERNO_DMG.set(ScoreHolder::self_(), 0)          // scoreboard players set @s inferno_dmg 0
//! INFERNO_DMG.add(ScoreHolder::self_(), 1)           // scoreboard players add …
//! INFERNO_DMG.subtract(ScoreHolder::self_(), 1)      // scoreboard players remove …
//! INFERNO_DMG.get(ScoreHolder::self_())              // scoreboard players get …
//!
//! // Display in tellraw (no raw strings)
//! INFERNO_DMG.as_text(Selector::self_())
//! // → TextComponent::score("@s", "inferno_dmg")
//! ```

use std::borrow::Cow;
use std::fmt;

use super::{Command, ScoreHolder, ScoreOp, Selector, Storage, TextComponent};

// ── ScoreboardPlayersOperation ────────────────────────────────────────────────

/// Result of [`scoreboard_players_operation`]. Implements [`Command`] so it
/// can be used anywhere a command string is expected.
#[derive(Debug, Clone)]
pub struct ScoreboardPlayersOperation {
    targets: String,
    target_objective: String,
    op: ScoreOp,
    source: String,
    source_objective: String,
}

impl fmt::Display for ScoreboardPlayersOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "scoreboard players operation {} {} {} {} {}",
            self.targets, self.target_objective, self.op, self.source, self.source_objective
        )
    }
}

impl Command for ScoreboardPlayersOperation {}

/// `scoreboard players operation <targets> <targetObjective> <op> <source> <sourceObjective>`
///
/// Performs integer arithmetic between two scores in-place. The target score
/// is modified; the source score is read but not changed (except for `><` swap).
///
/// ```rust,ignore
/// cmd::scoreboard_players_operation("@s", "player_mana", ScoreOp::Add, "@s", "player_mana_regen")
/// // → scoreboard players operation @s player_mana += @s player_mana_regen
///
/// cmd::scoreboard_players_operation("@s", "mana", ScoreOp::Min, "#max_mana", "const")
/// // → scoreboard players operation @s mana < #max_mana const  (cap at max)
/// ```
pub fn scoreboard_players_operation(
    targets: impl Into<String>,
    target_objective: impl Into<String>,
    op: ScoreOp,
    source: impl Into<String>,
    source_objective: impl Into<String>,
) -> ScoreboardPlayersOperation {
    ScoreboardPlayersOperation {
        targets: targets.into(),
        target_objective: target_objective.into(),
        op,
        source: source.into(),
        source_objective: source_objective.into(),
    }
}

// ── DisplaySlot ───────────────────────────────────────────────────────────────

/// The display slot for `scoreboard objectives setdisplay`.
///
/// # Example
/// ```rust,ignore
/// KILL_COUNT.set_display(DisplaySlot::Sidebar)
/// // → scoreboard objectives setdisplay sidebar kill_count
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplaySlot {
    /// `list` — player tab-list.
    List,
    /// `sidebar` — right-hand scoreboard sidebar.
    Sidebar,
    /// `belowname` — shown below the player name tag in world.
    BelowName,
    /// `sidebar.team.<color>` — team-colored sidebar (e.g. `DisplaySlot::TeamSidebar("red".into())`).
    TeamSidebar(String),
}

impl fmt::Display for DisplaySlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplaySlot::List => write!(f, "list"),
            DisplaySlot::Sidebar => write!(f, "sidebar"),
            DisplaySlot::BelowName => write!(f, "belowname"),
            DisplaySlot::TeamSidebar(color) => write!(f, "sidebar.team.{color}"),
        }
    }
}

// ── Objective ─────────────────────────────────────────────────────────────────

/// A named Minecraft scoreboard objective.
///
/// Objectives hold one integer per score-holder (player or fake player).
/// They are the only way to feed runtime-computed numeric values into most
/// Minecraft commands.
///
/// # Declaration
///
/// ```rust,ignore
/// use sand_core::cmd::Objective;
///
/// static INFERNO_DMG: Objective = Objective::new("inferno_dmg");
/// static COOLDOWN:    Objective = Objective::new("inferno_cd");
/// ```
///
/// # Bridging storage → objective → display
///
/// ```rust,ignore
/// // 1. Load a per-player float value from NBT storage into the objective
/// INFERNO_DMG.load_from_scaled(ScoreHolder::self_(), &PLAYERS, "uuid.damage", 10.0)
/// //    → execute store result score @s inferno_dmg
/// //          run data get storage my_pack:players uuid.damage 10
///
/// // 2. Use the objective value in tellraw
/// INFERNO_DMG.as_text(Selector::self_()).color(ChatColor::Yellow)
/// ```
pub struct Objective {
    name: Cow<'static, str>,
}

impl Objective {
    /// Const-compatible constructor for `static`/`const` declarations.
    ///
    /// Use this for objectives known at compile time (the common case).
    /// Produces no heap allocation — the name is borrowed from the `'static` string.
    ///
    /// # Example
    /// ```rust,ignore
    /// static INFERNO_DMG: Objective = Objective::new("inferno_dmg");
    /// ```
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
        }
    }

    /// Create an objective with a runtime-determined name.
    ///
    /// Use this when the objective name is computed or loaded at runtime.
    /// The name is heap-allocated; prefer `new()` for static objectives.
    pub fn dynamic(name: impl Into<String>) -> Self {
        Self {
            name: Cow::Owned(name.into()),
        }
    }

    /// Return the objective name as a string.
    pub fn name(&self) -> &str {
        &self.name
    }

    // ── Load from storage ──────────────────────────────────────────────────

    /// Load an integer NBT value from storage into this objective for `holder`.
    ///
    /// Generates:
    /// ```text
    /// execute store result score <holder> <obj>
    ///     run data get storage <id> <key>
    /// ```
    ///
    /// Use this when the stored value is already an integer type (`Int`, `Long`).
    /// For float values use [`load_from_scaled`](Self::load_from_scaled).
    pub fn load_from(
        &self,
        holder: ScoreHolder,
        storage: &Storage,
        key: impl Into<String>,
    ) -> String {
        format!(
            "execute store result score {} {} run {}",
            holder,
            self.name,
            storage.get(key)
        )
    }

    /// Load a float NBT value from storage, multiplied by `scale`, into this
    /// objective for `holder`.
    ///
    /// Minecraft truncates the result to an integer after scaling. Use this
    /// when the stored value is a float (e.g. `3.5`) and you want to preserve
    /// precision by scaling it up (e.g. scale `10.0` → stored value `35`).
    ///
    /// Generates:
    /// ```text
    /// execute store result score <holder> <obj>
    ///     run data get storage <id> <key> <scale>
    /// ```
    ///
    /// # Example
    /// ```rust,ignore
    /// // PLAYERS stores 3.5 under "uuid.damage"
    /// // scale=10.0 → score = 35
    /// INFERNO_DMG.load_from_scaled(ScoreHolder::self_(), &PLAYERS, "uuid.damage", 10.0)
    /// ```
    pub fn load_from_scaled(
        &self,
        holder: ScoreHolder,
        storage: &Storage,
        key: impl Into<String>,
        scale: f64,
    ) -> String {
        format!(
            "execute store result score {} {} run {}",
            holder,
            self.name,
            storage.get_scaled(key, scale)
        )
    }

    // ── Objective lifecycle ────────────────────────────────────────────────

    /// `scoreboard objectives add <name> <criterion>` — create this objective.
    ///
    /// Call from your `#[component(Load)]` function so the objective exists on pack load.
    ///
    /// Common criteria: `"dummy"`, `"deathCount"`, `"playerKillCount"`, `"totalKillCount"`,
    /// `"health"`, `"food"`, `"xp"`, `"level"`, `"trigger"`.
    ///
    /// ```rust,ignore
    /// static MANA: Objective = Objective::new("mana");
    ///
    /// #[component(Load)]
    /// pub fn on_load() {
    ///     mcfunction! { MANA.create("dummy"); }
    /// }
    /// ```
    pub fn create(&self, criterion: impl Into<String>) -> String {
        format!("scoreboard objectives add {} {}", self.name, criterion.into())
    }

    /// `scoreboard objectives add <name> <criterion> <displayName>` — create with a display name.
    ///
    /// The display name supports JSON text (e.g. `r#"{"text":"Mana","color":"blue"}"#`).
    pub fn create_with_display(&self, criterion: impl Into<String>, display: impl Into<String>) -> String {
        format!(
            "scoreboard objectives add {} {} {}",
            self.name,
            criterion.into(),
            display.into()
        )
    }

    /// `scoreboard objectives remove <name>` — delete this objective.
    ///
    /// Also removes it from any display slots it occupies.
    pub fn remove(&self) -> String {
        format!("scoreboard objectives remove {}", self.name)
    }

    /// `scoreboard objectives setdisplay <slot> <name>` — show this objective in a display slot.
    ///
    /// ```rust,ignore
    /// MANA.set_display(DisplaySlot::Sidebar)
    /// // → scoreboard objectives setdisplay sidebar mana
    /// ```
    pub fn set_display(&self, slot: DisplaySlot) -> String {
        format!("scoreboard objectives setdisplay {slot} {}", self.name)
    }

    /// `scoreboard objectives setdisplay <slot>` — clear the given display slot (no objective shown).
    pub fn clear_display(slot: DisplaySlot) -> String {
        format!("scoreboard objectives setdisplay {slot}")
    }

    /// `scoreboard objectives modify <name> displayname <text>` — change the display name.
    ///
    /// Accepts a JSON text component string for formatted names.
    pub fn modify_display_name(&self, display: impl Into<String>) -> String {
        format!("scoreboard objectives modify {} displayname {}", self.name, display.into())
    }

    /// `scoreboard objectives modify <name> rendertype <type>` — change render type.
    ///
    /// Valid types: `"integer"`, `"hearts"`.
    pub fn modify_render_type(&self, render_type: impl Into<String>) -> String {
        format!("scoreboard objectives modify {} rendertype {}", self.name, render_type.into())
    }

    // ── Direct manipulation ────────────────────────────────────────────────

    /// `scoreboard players set <holder> <obj> <value>` — set the score to an exact value.
    ///
    /// Produces: `scoreboard players set <holder> <objective> <value>`
    pub fn set(&self, holder: ScoreHolder, value: i32) -> String {
        format!("scoreboard players set {} {} {}", holder, self.name, value)
    }

    /// `scoreboard players get <holder> <obj>` — fetch the score value.
    ///
    /// Returns the command string for use in `execute store result` chains.
    /// The command's return value is the number of bytes read (for use in score calculations).
    /// Produces: `scoreboard players get <holder> <objective>`
    pub fn get(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players get {} {}", holder, self.name)
    }

    /// `scoreboard players add <holder> <obj> <amount>` — increment the score.
    ///
    /// Produces: `scoreboard players add <holder> <objective> <amount>`
    pub fn add(&self, holder: ScoreHolder, amount: i32) -> String {
        format!("scoreboard players add {} {} {}", holder, self.name, amount)
    }

    /// `scoreboard players remove <holder> <obj> <amount>` — decrement the score.
    ///
    /// Produces: `scoreboard players remove <holder> <objective> <amount>`
    pub fn subtract(&self, holder: ScoreHolder, amount: i32) -> String {
        format!(
            "scoreboard players remove {} {} {}",
            holder, self.name, amount
        )
    }

    /// `scoreboard players reset <holder> <obj>` — clear the score (remove this holder from the objective).
    ///
    /// Produces: `scoreboard players reset <holder> <objective>`
    pub fn reset(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players reset {} {}", holder, self.name)
    }

    // ── Arithmetic between objectives ──────────────────────────────────────

    /// `scoreboard players operation <lhs_holder> <obj> <op> <rhs_holder> <rhs_obj>`
    ///
    /// Performs integer arithmetic between two objective scores in-place.
    ///
    /// ```rust,ignore
    /// // TOTAL_DMG @s += INFERNO_DMG @s
    /// TOTAL_DMG.operation(ScoreHolder::self_(), ScoreOp::Add, ScoreHolder::self_(), &INFERNO_DMG)
    /// ```
    pub fn operation(
        &self,
        lhs: ScoreHolder,
        op: ScoreOp,
        rhs: ScoreHolder,
        rhs_obj: &Objective,
    ) -> String {
        format!(
            "scoreboard players operation {} {} {} {} {}",
            lhs, self.name, op, rhs, rhs_obj.name
        )
    }

    /// `scoreboard players enable <holder> <obj>` — enable a trigger objective for a player.
    ///
    /// Only applies to objectives with criterion `"trigger"`. Must be re-enabled
    /// each time (Minecraft disables it after it is triggered).
    ///
    /// ```rust,ignore
    /// static TRIGGER: Objective = Objective::new("my_trigger");
    ///
    /// // In your tick function — re-enable every tick so players can trigger it:
    /// TRIGGER.enable(ScoreHolder::entity(Selector::all_players()))
    /// // → scoreboard players enable @a my_trigger
    /// ```
    pub fn enable(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players enable {} {}", holder, self.name)
    }

    /// `scoreboard players set * <obj> <value>` — set the score for ALL tracked players.
    pub fn set_all(&self, value: i32) -> String {
        format!("scoreboard players set * {} {}", self.name, value)
    }

    /// `scoreboard players reset * <obj>` — reset scores for ALL tracked players.
    pub fn reset_all(&self) -> String {
        format!("scoreboard players reset * {}", self.name)
    }

    // ── Named operation shortcuts ──────────────────────────────────────────

    /// `scoreboard players operation <lhs> <obj> += <rhs> <rhs_obj>` — add rhs score into lhs.
    pub fn add_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Add, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> -= <rhs> <rhs_obj>` — subtract rhs from lhs.
    pub fn sub_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Sub, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> *= <rhs> <rhs_obj>` — multiply lhs by rhs.
    pub fn mul_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Mul, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> /= <rhs> <rhs_obj>` — divide lhs by rhs (truncated).
    pub fn div_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Div, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> %= <rhs> <rhs_obj>` — set lhs to lhs mod rhs.
    pub fn mod_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Mod, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> = <rhs> <rhs_obj>` — copy rhs into lhs.
    pub fn copy_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Set, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> < <rhs> <rhs_obj>` — set lhs to min(lhs, rhs).
    pub fn min_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Min, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> > <rhs> <rhs_obj>` — set lhs to max(lhs, rhs).
    pub fn max_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Max, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> >< <rhs> <rhs_obj>` — swap lhs and rhs values.
    pub fn swap_with(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Swap, rhs, rhs_obj)
    }

    // ── Execute conditions ─────────────────────────────────────────────────

    /// Return a condition fragment `if score <holder> <obj> matches <range>`.
    ///
    /// Use this with `Execute::if_()` to add a score condition to an execute chain.
    /// Range format: `"5"` (exact), `"5.."` (5 or more), `"..5"` (5 or less), `"1..10"` (between).
    ///
    /// # Example
    /// ```rust,ignore
    /// Execute::new()
    ///     .if_(COOLDOWN.if_matches(ScoreHolder::self_(), "0"))
    ///     .run(cmd::say("ready!"))
    /// ```
    pub fn if_matches(&self, holder: ScoreHolder, range: impl Into<String>) -> String {
        format!("if score {} {} matches {}", holder, self.name, range.into())
    }

    /// Return a condition fragment `unless score <holder> <obj> matches <range>`.
    ///
    /// Use this with `Execute::if_()` to skip execution if score is in range.
    pub fn unless_matches(&self, holder: ScoreHolder, range: impl Into<String>) -> String {
        format!(
            "unless score {} {} matches {}",
            holder,
            self.name,
            range.into()
        )
    }

    // ── Display ───────────────────────────────────────────────────────────

    /// Create a `TextComponent` displaying this objective's value for an entity selector.
    ///
    /// Use in `title`, `actionbar`, `tellraw`, or `bossbar` commands to show live scoreboard values.
    ///
    /// # Example
    /// ```rust,ignore
    /// INFERNO_DMG.as_text(Selector::self_()).color(ChatColor::Yellow)
    /// // → {"score":{"name":"@s","objective":"inferno_dmg"},"color":"yellow"}
    /// ```
    pub fn as_text(&self, selector: Selector) -> TextComponent {
        TextComponent::score(selector.to_string(), self.name())
    }

    /// Create a `TextComponent` displaying a fake player's score in this objective.
    ///
    /// Use this for global counters or persistent values tied to a fake player name.
    ///
    /// # Example
    /// ```rust,ignore
    /// KILL_COUNT.as_text_fake("__global")
    /// // → {"score":{"name":"__global","objective":"kill_count"}}
    /// ```
    pub fn as_text_fake(&self, fake_player: impl Into<String>) -> TextComponent {
        TextComponent::score(fake_player, self.name())
    }
}

// ── ScoreHolder convenience ────────────────────────────────────────────────────

/// Convenience constructors added to [`ScoreHolder`] for common patterns.
impl ScoreHolder {
    /// `@s` — score holder for the entity executing the command.
    ///
    /// Shorthand for `ScoreHolder::entity(Selector::self_())`.
    pub fn self_() -> Self {
        ScoreHolder::entity(Selector::self_())
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::Storage;

    static DMG: Objective = Objective::new("inferno_dmg");
    static PLAYERS: Storage = Storage::per_player("my_pack:players");

    #[test]
    fn objective_const() {
        assert_eq!(DMG.name(), "inferno_dmg");
    }

    #[test]
    fn load_from() {
        assert_eq!(
            DMG.load_from(ScoreHolder::self_(), &PLAYERS, "uuid.damage"),
            "execute store result score @s inferno_dmg run data get storage my_pack:players uuid.damage"
        );
    }

    #[test]
    fn load_from_scaled() {
        assert_eq!(
            DMG.load_from_scaled(ScoreHolder::self_(), &PLAYERS, "uuid.damage", 10.0),
            "execute store result score @s inferno_dmg run data get storage my_pack:players uuid.damage 10"
        );
    }

    #[test]
    fn set_get_add_subtract() {
        assert_eq!(
            DMG.set(ScoreHolder::self_(), 0),
            "scoreboard players set @s inferno_dmg 0"
        );
        assert_eq!(
            DMG.get(ScoreHolder::self_()),
            "scoreboard players get @s inferno_dmg"
        );
        assert_eq!(
            DMG.add(ScoreHolder::self_(), 5),
            "scoreboard players add @s inferno_dmg 5"
        );
        assert_eq!(
            DMG.subtract(ScoreHolder::self_(), 2),
            "scoreboard players remove @s inferno_dmg 2"
        );
        assert_eq!(
            DMG.reset(ScoreHolder::self_()),
            "scoreboard players reset @s inferno_dmg"
        );
    }

    #[test]
    fn operation() {
        static OTHER: Objective = Objective::new("other_dmg");
        let cmd = DMG.operation(
            ScoreHolder::self_(),
            ScoreOp::Add,
            ScoreHolder::self_(),
            &OTHER,
        );
        assert_eq!(
            cmd,
            "scoreboard players operation @s inferno_dmg += @s other_dmg"
        );
    }

    #[test]
    fn create_and_lifecycle() {
        assert_eq!(DMG.create("dummy"), "scoreboard objectives add inferno_dmg dummy");
        assert_eq!(
            DMG.create_with_display("dummy", r#"{"text":"Damage"}"#),
            r#"scoreboard objectives add inferno_dmg dummy {"text":"Damage"}"#
        );
        assert_eq!(DMG.remove(), "scoreboard objectives remove inferno_dmg");
        assert_eq!(
            DMG.set_display(DisplaySlot::Sidebar),
            "scoreboard objectives setdisplay sidebar inferno_dmg"
        );
        assert_eq!(
            DMG.set_display(DisplaySlot::TeamSidebar("red".into())),
            "scoreboard objectives setdisplay sidebar.team.red inferno_dmg"
        );
        assert_eq!(
            Objective::clear_display(DisplaySlot::Sidebar),
            "scoreboard objectives setdisplay sidebar"
        );
    }

    #[test]
    fn enable_and_wildcards() {
        static TRIGGER: Objective = Objective::new("my_trigger");
        assert_eq!(
            TRIGGER.enable(ScoreHolder::entity(Selector::all_players())),
            "scoreboard players enable @a my_trigger"
        );
        assert_eq!(DMG.set_all(0), "scoreboard players set * inferno_dmg 0");
        assert_eq!(DMG.reset_all(), "scoreboard players reset * inferno_dmg");
    }

    #[test]
    fn named_operations() {
        static OTHER: Objective = Objective::new("other");
        assert_eq!(
            DMG.add_from(ScoreHolder::self_(), ScoreHolder::self_(), &OTHER),
            "scoreboard players operation @s inferno_dmg += @s other"
        );
        assert_eq!(
            DMG.copy_from(ScoreHolder::self_(), ScoreHolder::self_(), &OTHER),
            "scoreboard players operation @s inferno_dmg = @s other"
        );
        assert_eq!(
            DMG.swap_with(ScoreHolder::self_(), ScoreHolder::self_(), &OTHER),
            "scoreboard players operation @s inferno_dmg >< @s other"
        );
        assert_eq!(
            DMG.min_from(ScoreHolder::self_(), ScoreHolder::self_(), &OTHER),
            "scoreboard players operation @s inferno_dmg < @s other"
        );
    }

    #[test]
    fn if_matches() {
        assert_eq!(
            DMG.if_matches(ScoreHolder::self_(), "1.."),
            "if score @s inferno_dmg matches 1.."
        );
    }

    #[test]
    fn as_text() {
        let t = DMG.as_text(Selector::self_()).to_string();
        assert!(t.contains("\"objective\":\"inferno_dmg\""));
        assert!(t.contains("\"name\":\"@s\""));
    }
}
