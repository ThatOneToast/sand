//! Typed scoreboard objective — a named integer counter in Minecraft.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use sand_commands::scoreboard::{Objective, ScoreHolder};
//!
//! static INFERNO_DMG: Objective = Objective::new("inferno_dmg");
//!
//! INFERNO_DMG.set(ScoreHolder::self_(), 0);
//! INFERNO_DMG.add(ScoreHolder::self_(), 1);
//! INFERNO_DMG.get(ScoreHolder::self_());
//! ```

use std::borrow::Cow;
use std::fmt;

use crate::Build;
use crate::selector::Selector;
use crate::text::TextComponent;

// ── ScoreHolder ───────────────────────────────────────────────────────────────

/// A scoreboard score holder — an entity selector or a named fake player.
///
/// # Examples
/// ```
/// use sand_commands::scoreboard::ScoreHolder;
/// use sand_commands::selector::Selector;
///
/// let self_holder = ScoreHolder::entity(Selector::self_());
/// assert_eq!(self_holder.to_string(), "@s");
///
/// let global = ScoreHolder::fake("#total_kills");
/// assert_eq!(global.to_string(), "#total_kills");
///
/// let everyone = ScoreHolder::all();
/// assert_eq!(everyone.to_string(), "*");
/// ```
#[derive(Debug, Clone)]
pub struct ScoreHolder(String);

impl ScoreHolder {
    /// Create a score holder from an entity selector.
    pub fn entity(selector: Selector) -> Self {
        ScoreHolder(selector.to_string())
    }

    /// Create a score holder from a named fake player.
    ///
    /// Convention: prefix with `#` (e.g. `"#const"`, `"#zero"`) to distinguish
    /// from real player names.
    pub fn fake(name: impl Into<String>) -> Self {
        ScoreHolder(name.into())
    }

    /// `*` — all score holders with any score in this objective.
    pub fn all() -> Self {
        ScoreHolder("*".into())
    }

    /// `@s` — score holder for the entity executing the command.
    pub fn self_() -> Self {
        ScoreHolder::entity(Selector::self_())
    }
}

impl fmt::Display for ScoreHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── ScoreOp ───────────────────────────────────────────────────────────────────

/// Arithmetic operation for `scoreboard players operation`.
///
/// # Examples
/// ```
/// use sand_commands::scoreboard::ScoreOp;
///
/// assert_eq!(ScoreOp::Add.to_string(), "+=");
/// assert_eq!(ScoreOp::Swap.to_string(), "><");
/// assert_eq!(ScoreOp::Min.to_string(), "<");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreOp {
    /// `+=` — add source to target.
    Add,
    /// `-=` — subtract source from target.
    Sub,
    /// `*=` — multiply target by source. Truncates toward zero.
    Mul,
    /// `/=` — divide target by source. Truncates toward zero.
    Div,
    /// `%=` — target becomes `target mod source`.
    Mod,
    /// `=` — assign source's value to target.
    Set,
    /// `<` — target becomes `min(target, source)`.
    Min,
    /// `>` — target becomes `max(target, source)`.
    Max,
    /// `><` — swap: exchange the values of target and source.
    Swap,
}

impl fmt::Display for ScoreOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ScoreOp::Add => "+=",
            ScoreOp::Sub => "-=",
            ScoreOp::Mul => "*=",
            ScoreOp::Div => "/=",
            ScoreOp::Mod => "%=",
            ScoreOp::Set => "=",
            ScoreOp::Min => "<",
            ScoreOp::Max => ">",
            ScoreOp::Swap => "><",
        };
        write!(f, "{s}")
    }
}

// ── ScoreCmp ──────────────────────────────────────────────────────────────────

/// Comparison operator for `execute if score <a> <obj> <cmp> <b> <obj>`.
///
/// # Examples
/// ```
/// use sand_commands::scoreboard::ScoreCmp;
///
/// assert_eq!(ScoreCmp::Eq.to_string(), "=");
/// assert_eq!(ScoreCmp::Le.to_string(), "<=");
/// assert_eq!(ScoreCmp::Ge.to_string(), ">=");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreCmp {
    /// `=` — left equals right.
    Eq,
    /// `<` — left is strictly less than right.
    Lt,
    /// `<=` — left is less than or equal to right.
    Le,
    /// `>` — left is strictly greater than right.
    Gt,
    /// `>=` — left is greater than or equal to right.
    Ge,
}

impl fmt::Display for ScoreCmp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ScoreCmp::Eq => "=",
            ScoreCmp::Lt => "<",
            ScoreCmp::Le => "<=",
            ScoreCmp::Gt => ">",
            ScoreCmp::Ge => ">=",
        };
        write!(f, "{s}")
    }
}

// ── ScoreboardPlayersOperation ────────────────────────────────────────────────

/// Result of [`scoreboard_players_operation`]. Implements [`Build`].
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

impl Build for ScoreboardPlayersOperation {
    fn build(&self) -> String {
        self.to_string()
    }
}

impl From<ScoreboardPlayersOperation> for String {
    fn from(v: ScoreboardPlayersOperation) -> Self {
        v.build()
    }
}

/// `scoreboard players operation <targets> <targetObjective> <op> <source> <sourceObjective>`
///
/// Performs integer arithmetic between two scores in-place.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplaySlot {
    /// `list` — player tab-list.
    List,
    /// `sidebar` — right-hand scoreboard sidebar.
    Sidebar,
    /// `belowname` — shown below the player name tag.
    BelowName,
    /// `sidebar.team.<color>` — team-colored sidebar.
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
/// # Declaration
///
/// ```rust,ignore
/// use sand_commands::scoreboard::Objective;
///
/// static INFERNO_DMG: Objective = Objective::new("inferno_dmg");
/// static COOLDOWN:    Objective = Objective::new("inferno_cd");
/// ```
pub struct Objective {
    name: Cow<'static, str>,
}

impl Objective {
    /// Const-compatible constructor for `static`/`const` declarations.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
        }
    }

    /// Create an objective with a runtime-determined name.
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

    /// `execute store result score <holder> <obj> run data get storage <storage_id> <key>`
    ///
    /// Load an integer value from a storage namespace into this objective.
    pub fn load_from(
        &self,
        holder: ScoreHolder,
        storage_id: impl Into<String>,
        key: impl Into<String>,
    ) -> String {
        format!(
            "execute store result score {} {} run data get storage {} {}",
            holder,
            self.name,
            storage_id.into(),
            key.into()
        )
    }

    /// `execute store result score <holder> <obj> run data get storage <storage_id> <key> <scale>`
    ///
    /// Load a float NBT value, multiplied by `scale`, into this objective.
    pub fn load_from_scaled(
        &self,
        holder: ScoreHolder,
        storage_id: impl Into<String>,
        key: impl Into<String>,
        scale: f64,
    ) -> String {
        format!(
            "execute store result score {} {} run data get storage {} {} {scale}",
            holder,
            self.name,
            storage_id.into(),
            key.into()
        )
    }

    // ── Objective lifecycle ────────────────────────────────────────────────

    /// `scoreboard objectives add <name> <criterion>` — create this objective.
    pub fn create(&self, criterion: impl Into<String>) -> String {
        format!(
            "scoreboard objectives add {} {}",
            self.name,
            criterion.into()
        )
    }

    /// `scoreboard objectives add <name> <criterion> <displayName>` — create with a display name.
    pub fn create_with_display(
        &self,
        criterion: impl Into<String>,
        display: impl Into<String>,
    ) -> String {
        format!(
            "scoreboard objectives add {} {} {}",
            self.name,
            criterion.into(),
            display.into()
        )
    }

    /// `scoreboard objectives remove <name>` — delete this objective.
    pub fn remove(&self) -> String {
        format!("scoreboard objectives remove {}", self.name)
    }

    /// `scoreboard objectives setdisplay <slot> <name>` — show in a display slot.
    pub fn set_display(&self, slot: DisplaySlot) -> String {
        format!("scoreboard objectives setdisplay {slot} {}", self.name)
    }

    /// `scoreboard objectives setdisplay <slot>` — clear the given display slot.
    pub fn clear_display(slot: DisplaySlot) -> String {
        format!("scoreboard objectives setdisplay {slot}")
    }

    /// `scoreboard objectives modify <name> displayname <text>` — change the display name.
    pub fn modify_display_name(&self, display: impl Into<String>) -> String {
        format!(
            "scoreboard objectives modify {} displayname {}",
            self.name,
            display.into()
        )
    }

    /// `scoreboard objectives modify <name> rendertype <type>` — change render type.
    pub fn modify_render_type(&self, render_type: impl Into<String>) -> String {
        format!(
            "scoreboard objectives modify {} rendertype {}",
            self.name,
            render_type.into()
        )
    }

    // ── Direct manipulation ────────────────────────────────────────────────

    /// `scoreboard players set <holder> <obj> <value>`
    pub fn set(&self, holder: ScoreHolder, value: i32) -> String {
        format!("scoreboard players set {} {} {}", holder, self.name, value)
    }

    /// `scoreboard players get <holder> <obj>`
    pub fn get(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players get {} {}", holder, self.name)
    }

    /// `scoreboard players add <holder> <obj> <amount>`
    pub fn add(&self, holder: ScoreHolder, amount: i32) -> String {
        format!("scoreboard players add {} {} {}", holder, self.name, amount)
    }

    /// `scoreboard players remove <holder> <obj> <amount>`
    pub fn subtract(&self, holder: ScoreHolder, amount: i32) -> String {
        format!(
            "scoreboard players remove {} {} {}",
            holder, self.name, amount
        )
    }

    /// `scoreboard players reset <holder> <obj>`
    pub fn reset(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players reset {} {}", holder, self.name)
    }

    // ── Arithmetic ────────────────────────────────────────────────────────

    /// `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`
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

    /// `scoreboard players enable <holder> <obj>` — enable a trigger objective.
    pub fn enable(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players enable {} {}", holder, self.name)
    }

    /// `scoreboard players set * <obj> <value>` — set score for ALL tracked players.
    pub fn set_all(&self, value: i32) -> String {
        format!("scoreboard players set * {} {}", self.name, value)
    }

    /// `scoreboard players reset * <obj>` — reset scores for ALL tracked players.
    pub fn reset_all(&self) -> String {
        format!("scoreboard players reset * {}", self.name)
    }

    // ── Named operation shortcuts ──────────────────────────────────────────

    /// `scoreboard players operation <lhs> <obj> += <rhs> <rhs_obj>`
    pub fn add_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Add, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> -= <rhs> <rhs_obj>`
    pub fn sub_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Sub, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> *= <rhs> <rhs_obj>`
    pub fn mul_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Mul, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> /= <rhs> <rhs_obj>`
    pub fn div_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Div, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> %= <rhs> <rhs_obj>`
    pub fn mod_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Mod, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> = <rhs> <rhs_obj>`
    pub fn copy_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Set, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> < <rhs> <rhs_obj>`
    pub fn min_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Min, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> > <rhs> <rhs_obj>`
    pub fn max_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Max, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> >< <rhs> <rhs_obj>`
    pub fn swap_with(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Swap, rhs, rhs_obj)
    }

    // ── Execute conditions ─────────────────────────────────────────────────

    /// Return a condition fragment `if score <holder> <obj> matches <range>`.
    pub fn if_matches(&self, holder: ScoreHolder, range: impl Into<String>) -> String {
        format!("if score {} {} matches {}", holder, self.name, range.into())
    }

    /// Return a condition fragment `unless score <holder> <obj> matches <range>`.
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
    pub fn as_text(&self, selector: Selector) -> TextComponent {
        TextComponent::score(selector.to_string(), self.name())
    }

    /// Create a `TextComponent` displaying a fake player's score in this objective.
    pub fn as_text_fake(&self, fake_player: impl Into<String>) -> TextComponent {
        TextComponent::score(fake_player, self.name())
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    static DMG: Objective = Objective::new("inferno_dmg");

    #[test]
    fn objective_const() {
        assert_eq!(DMG.name(), "inferno_dmg");
    }

    #[test]
    fn load_from() {
        assert_eq!(
            DMG.load_from(ScoreHolder::self_(), "my_pack:players", "uuid.damage"),
            "execute store result score @s inferno_dmg run data get storage my_pack:players uuid.damage"
        );
    }

    #[test]
    fn load_from_scaled() {
        assert_eq!(
            DMG.load_from_scaled(ScoreHolder::self_(), "my_pack:players", "uuid.damage", 10.0),
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
        assert_eq!(
            DMG.create("dummy"),
            "scoreboard objectives add inferno_dmg dummy"
        );
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

    #[test]
    fn scoreboard_players_operation_build() {
        use crate::Build;
        let op = scoreboard_players_operation("@s", "mana", ScoreOp::Add, "@s", "regen");
        assert_eq!(
            op.build(),
            "scoreboard players operation @s mana += @s regen"
        );
        let s: String = op.into();
        assert_eq!(s, "scoreboard players operation @s mana += @s regen");
    }
}
