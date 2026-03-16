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

use super::{ScoreHolder, ScoreOp, Selector, Storage, TextComponent};

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
