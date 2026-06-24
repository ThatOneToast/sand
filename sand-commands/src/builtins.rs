//! Free-function builders for common Minecraft commands.
//!
//! All functions return a `String` containing the completed command.
//!
//! # Example
//! ```rust,ignore
//! use sand_commands::*;
//!
//! let cmds = vec![
//!     say("Hello, world!"),
//!     tag_add(Selector::self_(), "has_sword"),
//!     kill(Selector::all_entities().not_tag("immortal")),
//! ];
//! ```

use crate::coord::{Rotation, Vec3};
use crate::selector::{EntityTargets, Selector, SingleEntity};
use crate::text::TextComponent;

// ── Chat / messaging ──────────────────────────────────────────────────────────

/// `say <message>` — broadcast a message to all players.
pub fn say(message: impl Into<String>) -> String {
    format!("say {}", message.into())
}

/// `tell <target> <message>` / `msg` — send a private message to the target.
pub fn tell(target: Selector, message: impl Into<String>) -> String {
    format!("tell {} {}", target, message.into())
}

/// `me <action>` — send an emote message (prefixed with the executor's name).
pub fn me(action: impl Into<String>) -> String {
    format!("me {}", action.into())
}

/// `tellraw <target> <json>` — send a rich JSON text component to the target.
pub fn tellraw(target: impl std::fmt::Display, text: TextComponent) -> String {
    format!("tellraw {} {}", target, text)
}

/// `tellraw <target> <raw_json>` — send a raw JSON string to the target.
pub fn tellraw_raw(target: impl std::fmt::Display, json: impl Into<String>) -> String {
    format!("tellraw {} {}", target, json.into())
}

// ── Entity management ─────────────────────────────────────────────────────────

/// `kill <selector>` — kill all entities matching the selector.
pub fn kill(selector: Selector) -> String {
    format!("kill {}", selector)
}

/// `summon <entity_type> <x> <y> <z>` — summon an entity at an absolute position.
pub fn summon(entity_type: impl Into<String>, x: f64, y: f64, z: f64) -> String {
    format!("summon {} {} {} {}", entity_type.into(), x, y, z)
}

/// `summon <entity_type>` — summon an entity at the current position (`~ ~ ~`).
pub fn summon_here(entity_type: impl Into<String>) -> String {
    format!("summon {} ~ ~ ~", entity_type.into())
}

/// `tp <target> <destination>` — teleport the target to another entity's position.
pub fn tp_to_entity(target: Selector, destination: Selector) -> String {
    format!("tp {} {}", target, destination)
}

/// `tp <target> <x> <y> <z>` — teleport the target to absolute coordinates.
pub fn tp(target: Selector, x: f64, y: f64, z: f64) -> String {
    format!("tp {} {} {} {}", target, x, y, z)
}

/// `tp <target> ~<dx> ~<dy> ~<dz>` — teleport the target by a relative offset.
pub fn tp_relative(target: Selector, dx: f64, dy: f64, dz: f64) -> String {
    let fmt_r = |v: f64| -> String {
        if v == 0.0 {
            "~".to_string()
        } else if v == v.trunc() {
            format!("~{}", v as i64)
        } else {
            format!("~{v}")
        }
    };
    format!("tp {} {} {} {}", target, fmt_r(dx), fmt_r(dy), fmt_r(dz))
}

/// `tp <target> <pos>` — teleport using a typed [`Vec3`] position.
///
/// Supports absolute, relative (`~`), and local (`^`) coordinates.
///
/// # Examples
/// ```
/// use sand_commands::{Selector, coord::{Vec3, Coord}};
/// use sand_commands::builtins::tp_vec3;
///
/// assert_eq!(tp_vec3(Selector::self_(), Vec3::here()), "tp @s ~ ~ ~");
/// assert_eq!(tp_vec3(Selector::self_(), Vec3::absolute(10.0, 64.0, -5.0)), "tp @s 10 64 -5");
/// assert_eq!(
///     tp_vec3(Selector::self_(), Vec3::new(Coord::local_n(0.0), Coord::local_n(0.5), Coord::local_n(2.0))),
///     "tp @s ^ ^0.5 ^2",
/// );
/// ```
pub fn tp_vec3(target: Selector, pos: Vec3) -> String {
    format!("tp {} {}", target, pos)
}

/// `tp <target> <pos> <rotation>` — teleport with an explicit facing direction.
///
/// # Example
/// ```
/// use sand_commands::{Selector, coord::{Vec3, Rotation}};
/// use sand_commands::builtins::tp_with_rotation;
///
/// let cmd = tp_with_rotation(Selector::self_(), Vec3::here(), Rotation::absolute(90.0, 0.0));
/// assert_eq!(cmd, "tp @s ~ ~ ~ 90 0");
/// ```
pub fn tp_with_rotation(target: Selector, pos: Vec3, rotation: Rotation) -> String {
    format!("tp {} {} {}", target, pos, rotation)
}

/// `summon <entity_type> <pos>` — summon an entity at a typed [`Vec3`] position.
///
/// # Example
/// ```
/// use sand_commands::coord::Vec3;
/// use sand_commands::builtins::summon_at;
///
/// assert_eq!(summon_at("minecraft:zombie", Vec3::here()), "summon minecraft:zombie ~ ~ ~");
/// assert_eq!(summon_at("minecraft:armor_stand", Vec3::absolute(0.0, 64.0, 0.0)), "summon minecraft:armor_stand 0 64 0");
/// ```
pub fn summon_at(entity_type: impl Into<String>, pos: Vec3) -> String {
    format!("summon {} {}", entity_type.into(), pos)
}

/// `summon <entity_type> <pos> <nbt>` — summon an entity at a position with NBT data.
///
/// # Example
/// ```
/// use sand_commands::coord::Vec3;
/// use sand_commands::builtins::summon_at_with_nbt;
///
/// let cmd = summon_at_with_nbt("minecraft:armor_stand", Vec3::here(), "{Invisible:1b}");
/// assert_eq!(cmd, "summon minecraft:armor_stand ~ ~ ~ {Invisible:1b}");
/// ```
pub fn summon_at_with_nbt(
    entity_type: impl Into<String>,
    pos: Vec3,
    nbt: impl std::fmt::Display,
) -> String {
    format!("summon {} {} {}", entity_type.into(), pos, nbt)
}

// ── Tags ──────────────────────────────────────────────────────────────────────

/// `tag <selector> add <tag>` — add a tag to matching entities.
pub fn tag_add(selector: Selector, tag: impl Into<String>) -> String {
    format!("tag {} add {}", selector, tag.into())
}

/// `tag <selector> remove <tag>` — remove a tag from matching entities.
pub fn tag_remove(selector: Selector, tag: impl Into<String>) -> String {
    format!("tag {} remove {}", selector, tag.into())
}

// ── Gamemode ──────────────────────────────────────────────────────────────────

/// `gamemode survival <selector>`.
pub fn gamemode_survival(selector: Selector) -> String {
    format!("gamemode survival {}", selector)
}

/// `gamemode creative <selector>`.
pub fn gamemode_creative(selector: Selector) -> String {
    format!("gamemode creative {}", selector)
}

/// `gamemode adventure <selector>`.
pub fn gamemode_adventure(selector: Selector) -> String {
    format!("gamemode adventure {}", selector)
}

/// `gamemode spectator <selector>`.
pub fn gamemode_spectator(selector: Selector) -> String {
    format!("gamemode spectator {}", selector)
}

/// `gamemode <mode> <selector>` — set gamemode using a raw mode string.
pub fn gamemode(mode: impl Into<String>, selector: Selector) -> String {
    format!("gamemode {} {}", mode.into(), selector)
}

// ── Effects ───────────────────────────────────────────────────────────────────

/// `effect give <selector> <effect> <duration> <amplifier>` — apply a status effect.
pub fn effect_give(
    selector: Selector,
    effect: impl Into<String>,
    duration: u32,
    amplifier: u8,
) -> String {
    format!(
        "effect give {} {} {} {}",
        selector,
        effect.into(),
        duration,
        amplifier
    )
}

/// `effect give <selector> <effect> <duration> <amplifier> true` — hide particles.
pub fn effect_give_hidden(
    selector: Selector,
    effect: impl Into<String>,
    duration: u32,
    amplifier: u8,
) -> String {
    format!(
        "effect give {} {} {} {} true",
        selector,
        effect.into(),
        duration,
        amplifier
    )
}

/// `effect clear <selector>` — clear all status effects.
pub fn effect_clear(selector: Selector) -> String {
    format!("effect clear {}", selector)
}

/// `effect clear <selector> <effect>` — clear a specific status effect.
pub fn effect_clear_effect(selector: Selector, effect: impl Into<String>) -> String {
    format!("effect clear {} {}", selector, effect.into())
}

// ── Experience ────────────────────────────────────────────────────────────────

/// `experience add <selector> <amount> points` — add experience points.
pub fn xp_add_points(selector: Selector, amount: i32) -> String {
    format!("experience add {} {} points", selector, amount)
}

/// `experience add <selector> <amount> levels` — add experience levels.
pub fn xp_add_levels(selector: Selector, amount: i32) -> String {
    format!("experience add {} {} levels", selector, amount)
}

/// `experience set <selector> <amount> points` — set experience points.
pub fn xp_set_points(selector: Selector, amount: u32) -> String {
    format!("experience set {} {} points", selector, amount)
}

/// `experience set <selector> <amount> levels` — set experience levels.
pub fn xp_set_levels(selector: Selector, amount: i32) -> String {
    format!("experience set {} {} levels", selector, amount)
}

// ── Teams ─────────────────────────────────────────────────────────────────────

/// `team add <team> [<display_name>]` — create a new team.
pub fn team_add(team: impl Into<String>) -> String {
    format!("team add {}", team.into())
}

/// `team remove <team>` — delete a team.
pub fn team_remove(team: impl Into<String>) -> String {
    format!("team remove {}", team.into())
}

/// `team join <team> <selector>` — add entities to a team.
pub fn team_join(team: impl Into<String>, selector: Selector) -> String {
    format!("team join {} {}", team.into(), selector)
}

/// `team leave <selector>` — remove entities from their current team.
pub fn team_leave(selector: Selector) -> String {
    format!("team leave {}", selector)
}

// ── World / time / weather ────────────────────────────────────────────────────

/// `time set <value>` — set the world time (e.g. `"day"`, `"noon"`, `"night"`, or a tick count).
pub fn time_set(value: impl Into<String>) -> String {
    format!("time set {}", value.into())
}

/// `time add <value>` — add ticks to the current world time.
pub fn time_add(ticks: i32) -> String {
    format!("time add {}", ticks)
}

/// `weather clear [<duration>]` — set clear weather.
pub fn weather_clear() -> String {
    "weather clear".to_string()
}

/// `weather rain [<duration>]` — set rainy weather.
pub fn weather_rain() -> String {
    "weather rain".to_string()
}

/// `weather thunder [<duration>]` — set thunder weather.
pub fn weather_thunder() -> String {
    "weather thunder".to_string()
}

/// `difficulty <level>` — set world difficulty (peaceful, easy, normal, hard).
pub fn difficulty(level: impl Into<String>) -> String {
    format!("difficulty {}", level.into())
}

// ── Functions / scheduling ────────────────────────────────────────────────────

/// `function <namespace:path>` — run a datapack function.
pub fn function(id: impl Into<String>) -> String {
    format!("function {}", id.into())
}

/// `schedule function <id> <time> [append|replace]` — schedule a function.
pub fn schedule(id: impl Into<String>, time: impl Into<String>, mode: impl Into<String>) -> String {
    format!(
        "schedule function {} {} {}",
        id.into(),
        time.into(),
        mode.into()
    )
}

/// `schedule function <id> <time> replace` — schedule (replace any existing).
pub fn schedule_replace(id: impl Into<String>, time: impl Into<String>) -> String {
    schedule(id, time, "replace")
}

/// `schedule clear <id>` — cancel a scheduled function.
pub fn schedule_clear(id: impl Into<String>) -> String {
    format!("schedule clear {}", id.into())
}

/// `reload` — reload all datapacks.
pub fn reload() -> String {
    "reload".to_string()
}

// ── Gamerules ─────────────────────────────────────────────────────────────────

/// `gamerule <rule> <value>` — set a gamerule.
pub fn gamerule(rule: impl Into<String>, value: impl Into<String>) -> String {
    format!("gamerule {} {}", rule.into(), value.into())
}

/// `gamerule keepInventory true`.
pub fn gamerule_keep_inventory(enabled: bool) -> String {
    gamerule("keepInventory", enabled.to_string())
}

/// `gamerule doMobSpawning true/false`.
pub fn gamerule_mob_spawning(enabled: bool) -> String {
    gamerule("doMobSpawning", enabled.to_string())
}

/// `gamerule doFireTick true/false`.
pub fn gamerule_fire_tick(enabled: bool) -> String {
    gamerule("doFireTick", enabled.to_string())
}

// ── Damage ────────────────────────────────────────────────────────────────────

/// `damage <target> <amount> <damage_type>` — direct vanilla damage command.
///
/// Vanilla accepts exactly one entity target. Use [`Damage`] for high-level
/// damage that can safely target many entities.
pub fn damage(
    target: impl Into<SingleEntity>,
    amount: f64,
    damage_type: impl Into<String>,
) -> String {
    let target = target.into();
    format!("damage {} {} {}", target, amount, damage_type.into())
}

/// A damage amount that Sand can lower to Minecraft commands.
///
/// All safe constructors produce [`Fixed`](DamageAmount::Fixed) — a concrete
/// hit-point value that maps directly to the vanilla `damage` command. Score-backed
/// or per-event amounts are not yet supported; if you need them, track damage via
/// [`sand_core::systems::damage::DamageTracker`] and emit the value from a scoreboard
/// operation instead.
///
/// # Migration note
///
/// The `Score` and `SameAsEvent` variants were removed in this release because they
/// panicked at command-generation time and could never produce valid output. Use
/// `DamageAmount::fixed`, `DamageAmount::hearts`, or `DamageAmount::points` instead.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum DamageAmount {
    /// A fixed number of hit points.
    Fixed(f64),
}

impl DamageAmount {
    /// Fixed hit-point damage.
    pub fn fixed(amount: f64) -> Self {
        Self::Fixed(amount)
    }

    /// Damage in hearts (1.0 = one heart = 2 HP).
    ///
    /// This is the preferred user-facing constructor. The `damage` command takes
    /// HP (hit points), so `hearts(1.0)` emits `2.0` to the command.
    pub fn hearts(h: f32) -> Self {
        Self::Fixed(h as f64 * 2.0)
    }

    /// Damage in vanilla hit points (1.0 HP = 0.5 hearts).
    ///
    /// Equivalent to [`fixed`](DamageAmount::fixed). Use when you are already
    /// thinking in HP rather than hearts.
    pub fn points(p: f32) -> Self {
        Self::Fixed(p as f64)
    }

    fn as_fixed(&self) -> f64 {
        match self {
            Self::Fixed(v) => *v,
        }
    }
}

impl From<f64> for DamageAmount {
    fn from(value: f64) -> Self {
        Self::Fixed(value)
    }
}

impl From<f32> for DamageAmount {
    fn from(value: f32) -> Self {
        Self::Fixed(value as f64)
    }
}

/// Built-in vanilla damage type IDs for command damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageKind {
    /// `minecraft:generic`
    Generic,
    /// `minecraft:magic`
    Magic,
    /// `minecraft:thorns`
    Thorns,
    /// `minecraft:freeze`
    Freeze,
    /// `minecraft:lightning_bolt`
    LightningBolt,
}

impl DamageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Generic => "minecraft:generic",
            Self::Magic => "minecraft:magic",
            Self::Thorns => "minecraft:thorns",
            Self::Freeze => "minecraft:freeze",
            Self::LightningBolt => "minecraft:lightning_bolt",
        }
    }
}

impl std::fmt::Display for DamageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<DamageKind> for String {
    fn from(value: DamageKind) -> Self {
        value.as_str().to_string()
    }
}

/// High-level damage builder that knows how to lower multi-target damage safely.
#[derive(Debug, Clone)]
pub struct Damage {
    targets: DamageTargets,
    amount: DamageAmount,
    damage_type: String,
    source: Option<SingleEntity>,
    centered_at: Option<SingleEntity>,
}

#[derive(Debug, Clone)]
pub enum DamageTargets {
    One(SingleEntity),
    Many(EntityTargets),
}

impl Damage {
    /// Create a damage builder with default generic fixed 1.0 damage to `@s`.
    pub fn new() -> Self {
        Self {
            targets: DamageTargets::One(SingleEntity::self_()),
            amount: DamageAmount::Fixed(1.0),
            damage_type: DamageKind::Generic.as_str().to_string(),
            source: None,
            centered_at: None,
        }
    }

    /// Start a reflected-damage builder centered on `source`.
    pub fn reflect_from(source: impl Into<SingleEntity>) -> Self {
        let source = source.into();
        Self::new().centered_at(source)
    }

    /// Set target entity or entities.
    pub fn to(mut self, targets: impl IntoDamageTargets) -> Self {
        self.targets = targets.into_damage_targets();
        self
    }

    /// Set damage amount.
    pub fn amount(mut self, amount: impl Into<DamageAmount>) -> Self {
        self.amount = amount.into();
        self
    }

    /// Set damage type/resource ID.
    pub fn damage_type(mut self, damage_type: impl Into<String>) -> Self {
        self.damage_type = damage_type.into();
        self
    }

    /// Alias for [`damage_type`](Damage::damage_type).
    pub fn kind(self, damage_type: impl Into<String>) -> Self {
        self.damage_type(damage_type)
    }

    /// Attribute the damage source to a single entity.
    pub fn source(mut self, source: impl Into<SingleEntity>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Explicitly omit source attribution.
    pub fn without_source(mut self) -> Self {
        self.source = None;
        self
    }

    /// Run target selection at another single entity's position.
    pub fn centered_at(mut self, center: impl Into<SingleEntity>) -> Self {
        self.centered_at = Some(center.into());
        self
    }

    /// Build one or more valid Minecraft command lines.
    pub fn run(self) -> Vec<String> {
        let Self {
            targets,
            amount,
            damage_type,
            source,
            centered_at,
        } = self;
        let amount = amount.as_fixed();
        match targets {
            DamageTargets::One(target) => {
                vec![damage_command(
                    target,
                    amount,
                    &damage_type,
                    source.as_ref(),
                )]
            }
            DamageTargets::Many(targets) => {
                let inner =
                    damage_command(SingleEntity::self_(), amount, &damage_type, source.as_ref());
                let prefix = match centered_at {
                    Some(center) => format!("execute at {center} as {targets}"),
                    None => format!("execute as {targets}"),
                };
                vec![format!("{prefix} run {inner}")]
            }
        }
    }
}

fn damage_command(
    target: SingleEntity,
    amount: f64,
    damage_type: &str,
    source: Option<&SingleEntity>,
) -> String {
    let base = format!("damage {target} {amount} {damage_type}");
    match source {
        Some(source) => format!("{base} by {source}"),
        None => base,
    }
}

impl Default for Damage {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts typed target wrappers into the damage builder's target model.
pub trait IntoDamageTargets {
    fn into_damage_targets(self) -> DamageTargets;
}

impl IntoDamageTargets for SingleEntity {
    fn into_damage_targets(self) -> DamageTargets {
        DamageTargets::One(self)
    }
}

impl IntoDamageTargets for EntityTargets {
    fn into_damage_targets(self) -> DamageTargets {
        DamageTargets::Many(self)
    }
}

// ── Attributes ────────────────────────────────────────────────────────────────

/// `attribute <target> <attribute> get` — get an attribute value.
pub fn attribute_get(target: Selector, attribute: impl Into<String>) -> String {
    format!("attribute {} {} get", target, attribute.into())
}

/// `attribute <target> <attribute> base set <value>` — set an attribute's base value.
pub fn attribute_base_set(target: Selector, attribute: impl Into<String>, value: f64) -> String {
    format!(
        "attribute {} {} base set {}",
        target,
        attribute.into(),
        value
    )
}

// ── Misc ──────────────────────────────────────────────────────────────────────

/// `clear <selector>` — clear the entire inventory of matching entities.
pub fn clear(selector: Selector) -> String {
    format!("clear {}", selector)
}

/// `clear <selector> <item>` — remove all of a specific item from matching entities.
pub fn clear_item(selector: Selector, item: impl Into<String>) -> String {
    format!("clear {} {}", selector, item.into())
}

/// `give <selector> <item>` — give an item to matching entities.
pub fn give(selector: Selector, item: impl Into<String>) -> String {
    format!("give {} {}", selector, item.into())
}

/// `give <selector> <item> <count>` — give a stack of items.
pub fn give_count(selector: Selector, item: impl Into<String>, count: u32) -> String {
    format!("give {} {} {}", selector, item.into(), count)
}

/// `setblock <x> <y> <z> <block>` — place a block at absolute coordinates.
pub fn setblock_abs(x: i32, y: i32, z: i32, block: impl Into<String>) -> String {
    format!("setblock {} {} {} {}", x, y, z, block.into())
}

/// `seed` — output the world seed.
pub fn seed() -> String {
    "seed".to_string()
}

/// `list` — list all connected players.
pub fn list() -> String {
    "list".to_string()
}

/// `kick <player> [<reason>]` — kick a player.
pub fn kick(player: impl Into<String>, reason: Option<&str>) -> String {
    match reason {
        Some(r) => format!("kick {} {}", player.into(), r),
        None => format!("kick {}", player.into()),
    }
}

/// `return fail` — stop the current function and return a failure value.
///
/// In Minecraft 1.20.2+, `return fail` terminates the current `.mcfunction`
/// and signals to callers that the function failed (return value −1).
/// Equivalent to the Java API `return -1` for `execute … run function`.
///
/// Use inside branch functions to halt that branch without affecting the parent.
pub fn return_fail() -> String {
    "return fail".to_string()
}

/// `return <value>` — stop the current function and return an integer value.
///
/// `return_cmd(0)` → `return 0` (success)
/// `return_cmd(1)` → `return 1`
///
/// In Minecraft 1.20.2+, `return <n>` terminates the current `.mcfunction`
/// with the given integer result code, visible to `execute store result`.
pub fn return_cmd(value: i32) -> String {
    format!("return {value}")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn say_test() {
        assert_eq!(say("hello"), "say hello");
    }

    #[test]
    fn kill_test() {
        assert_eq!(kill(Selector::all_entities()), "kill @e");
    }

    #[test]
    fn tag_test() {
        assert_eq!(tag_add(Selector::self_(), "ready"), "tag @s add ready");
        assert_eq!(
            tag_remove(Selector::self_(), "ready"),
            "tag @s remove ready"
        );
    }

    #[test]
    fn effect_give_test() {
        assert_eq!(
            effect_give(Selector::self_(), "minecraft:speed", 100, 1),
            "effect give @s minecraft:speed 100 1"
        );
    }

    #[test]
    fn gamemode_test() {
        assert_eq!(gamemode_creative(Selector::self_()), "gamemode creative @s");
    }

    #[test]
    fn xp_test() {
        assert_eq!(
            xp_add_points(Selector::self_(), 50),
            "experience add @s 50 points"
        );
        assert_eq!(
            xp_set_levels(Selector::all_players(), 10),
            "experience set @a 10 levels"
        );
    }

    #[test]
    fn team_test() {
        assert_eq!(team_add("red"), "team add red");
        assert_eq!(team_join("red", Selector::self_()), "team join red @s");
        assert_eq!(team_leave(Selector::self_()), "team leave @s");
    }

    #[test]
    fn function_test() {
        assert_eq!(function("my_pack:tick"), "function my_pack:tick");
    }

    #[test]
    fn gamerule_test() {
        assert_eq!(gamerule_keep_inventory(true), "gamerule keepInventory true");
    }

    #[test]
    fn damage_test() {
        assert_eq!(
            damage(Selector::self_(), 5.0, "minecraft:generic"),
            "damage @s 5 minecraft:generic"
        );
    }

    #[test]
    fn damage_many_lowers_through_execute_as() {
        let cmds = Damage::new()
            .to(EntityTargets::nearby(5.0)
                .excluding_players()
                .excluding_self())
            .amount(DamageAmount::fixed(4.0))
            .damage_type(DamageKind::Generic)
            .run();

        assert_eq!(
            cmds,
            vec![
                "execute as @e[distance=0.1..5,type=!minecraft:player] run damage @s 4 minecraft:generic"
            ]
        );
        assert!(!cmds[0].contains("damage @e"));
    }

    #[test]
    fn reflected_many_damage_centers_on_source_without_unsafe_attribution() {
        let cmds = Damage::reflect_from(SingleEntity::self_())
            .to(EntityTargets::nearby(5.0)
                .excluding_players()
                .excluding_self())
            .amount(4.0)
            .damage_type(DamageKind::Generic)
            .run();

        assert_eq!(
            cmds,
            vec![
                "execute at @s as @e[distance=0.1..5,type=!minecraft:player] run damage @s 4 minecraft:generic"
            ]
        );
    }

    #[test]
    fn damage_amount_hearts_one_heart_is_two_hp() {
        let cmd = Damage::new()
            .amount(DamageAmount::hearts(1.0))
            .damage_type(DamageKind::Generic)
            .run();
        assert_eq!(cmd, vec!["damage @s 2 minecraft:generic"]);
    }

    #[test]
    fn damage_amount_hearts_half_heart_is_one_hp() {
        let cmd = Damage::new()
            .amount(DamageAmount::hearts(0.5))
            .damage_type(DamageKind::Generic)
            .run();
        assert_eq!(cmd, vec!["damage @s 1 minecraft:generic"]);
    }

    #[test]
    fn damage_amount_points_passthrough() {
        let cmd = Damage::new()
            .amount(DamageAmount::points(5.0))
            .damage_type(DamageKind::Generic)
            .run();
        assert_eq!(cmd, vec!["damage @s 5 minecraft:generic"]);
    }

    #[test]
    fn schedule_test() {
        assert_eq!(
            schedule_replace("my_pack:delayed", "20t"),
            "schedule function my_pack:delayed 20t replace"
        );
    }

    // ── Issue #1: DamageAmount never panics ───────────────────────────────────

    #[test]
    fn damage_amount_fixed_no_panic() {
        let a = DamageAmount::fixed(5.0);
        assert_eq!(a.as_fixed(), 5.0);
    }

    #[test]
    fn damage_amount_hearts_no_panic() {
        let a = DamageAmount::hearts(2.0);
        assert_eq!(a.as_fixed(), 4.0);
    }

    #[test]
    fn damage_amount_points_no_panic() {
        let a = DamageAmount::points(3.0);
        assert_eq!(a.as_fixed(), 3.0);
    }

    #[test]
    fn damage_amount_from_f64_no_panic() {
        let a: DamageAmount = 6.0_f64.into();
        assert_eq!(a.as_fixed(), 6.0);
    }

    #[test]
    fn damage_amount_from_f32_no_panic() {
        let a: DamageAmount = 2.5_f32.into();
        assert_eq!(a.as_fixed(), 2.5);
    }

    // ── Issue #2: typed teleport / summon ─────────────────────────────────────

    #[test]
    fn tp_vec3_here() {
        use crate::coord::Vec3;
        assert_eq!(tp_vec3(Selector::self_(), Vec3::here()), "tp @s ~ ~ ~");
    }

    #[test]
    fn tp_vec3_absolute() {
        use crate::coord::Vec3;
        assert_eq!(
            tp_vec3(Selector::self_(), Vec3::absolute(10.0, 64.0, -5.0)),
            "tp @s 10 64 -5"
        );
    }

    #[test]
    fn tp_vec3_local_coords() {
        use crate::coord::{Coord, Vec3};
        let pos = Vec3::new(
            Coord::local(),
            Coord::local_n(0.5_f64),
            Coord::local_n(2.0_f64),
        );
        assert_eq!(tp_vec3(Selector::self_(), pos), "tp @s ^ ^0.5 ^2");
    }

    #[test]
    fn tp_with_rotation_absolute() {
        use crate::coord::{Rotation, Vec3};
        let cmd = tp_with_rotation(
            Selector::self_(),
            Vec3::here(),
            Rotation::absolute(90.0, 0.0),
        );
        assert_eq!(cmd, "tp @s ~ ~ ~ 90 0");
    }

    #[test]
    fn tp_with_rotation_relative() {
        use crate::coord::{Rotation, Vec3};
        let cmd = tp_with_rotation(Selector::self_(), Vec3::here(), Rotation::here());
        assert_eq!(cmd, "tp @s ~ ~ ~ ~ ~");
    }

    #[test]
    fn summon_at_here() {
        use crate::coord::Vec3;
        assert_eq!(
            summon_at("minecraft:zombie", Vec3::here()),
            "summon minecraft:zombie ~ ~ ~"
        );
    }

    #[test]
    fn summon_at_absolute() {
        use crate::coord::Vec3;
        assert_eq!(
            summon_at("minecraft:armor_stand", Vec3::absolute(0.0, 64.0, 0.0)),
            "summon minecraft:armor_stand 0 64 0"
        );
    }

    #[test]
    fn summon_at_with_nbt_test() {
        use crate::coord::Vec3;
        let cmd = summon_at_with_nbt("minecraft:armor_stand", Vec3::here(), "{Invisible:1b}");
        assert_eq!(cmd, "summon minecraft:armor_stand ~ ~ ~ {Invisible:1b}");
    }

    // ── Additional command golden tests ───────────────────────────────────────

    #[test]
    fn tell_whisper() {
        assert_eq!(
            tell(Selector::nearest_player(), "secret message"),
            "tell @p secret message"
        );
    }

    #[test]
    fn me_emote() {
        assert_eq!(me("waves"), "me waves");
    }

    #[test]
    fn attribute_get_test() {
        assert_eq!(
            attribute_get(Selector::self_(), "minecraft:generic.max_health"),
            "attribute @s minecraft:generic.max_health get"
        );
    }

    #[test]
    fn attribute_base_set_test() {
        assert_eq!(
            attribute_base_set(Selector::self_(), "minecraft:generic.max_health", 40.0),
            "attribute @s minecraft:generic.max_health base set 40"
        );
    }

    #[test]
    fn clear_inventory() {
        assert_eq!(clear(Selector::self_()), "clear @s");
        assert_eq!(
            clear_item(Selector::self_(), "minecraft:dirt"),
            "clear @s minecraft:dirt"
        );
    }

    #[test]
    fn give_item() {
        assert_eq!(
            give(Selector::self_(), "minecraft:diamond_sword"),
            "give @s minecraft:diamond_sword"
        );
        assert_eq!(
            give_count(Selector::all_players(), "minecraft:apple", 64),
            "give @a minecraft:apple 64"
        );
    }

    #[test]
    fn time_commands() {
        assert_eq!(time_set("day"), "time set day");
        assert_eq!(time_set("6000"), "time set 6000");
        assert_eq!(time_add(20), "time add 20");
    }

    #[test]
    fn weather_commands() {
        assert_eq!(weather_clear(), "weather clear");
        assert_eq!(weather_rain(), "weather rain");
        assert_eq!(weather_thunder(), "weather thunder");
    }

    #[test]
    fn difficulty_command() {
        assert_eq!(difficulty("hard"), "difficulty hard");
        assert_eq!(difficulty("peaceful"), "difficulty peaceful");
    }

    #[test]
    fn schedule_append_mode() {
        assert_eq!(
            schedule("my_pack:delayed", "40t", "append"),
            "schedule function my_pack:delayed 40t append"
        );
        assert_eq!(
            schedule_clear("my_pack:delayed"),
            "schedule clear my_pack:delayed"
        );
    }

    #[test]
    fn return_variants() {
        assert_eq!(return_fail(), "return fail");
        assert_eq!(return_cmd(0), "return 0");
        assert_eq!(return_cmd(1), "return 1");
        assert_eq!(return_cmd(-1), "return -1");
    }

    #[test]
    fn xp_add_levels_test() {
        assert_eq!(
            xp_add_levels(Selector::self_(), 5),
            "experience add @s 5 levels"
        );
        assert_eq!(
            xp_set_points(Selector::self_(), 0),
            "experience set @s 0 points"
        );
    }

    #[test]
    fn setblock_abs_test() {
        assert_eq!(
            setblock_abs(10, 64, -5, "minecraft:stone"),
            "setblock 10 64 -5 minecraft:stone"
        );
    }

    #[test]
    fn gamerule_variants() {
        assert_eq!(gamerule_mob_spawning(false), "gamerule doMobSpawning false");
        assert_eq!(gamerule_fire_tick(true), "gamerule doFireTick true");
        assert_eq!(
            gamerule("sendCommandFeedback", "false"),
            "gamerule sendCommandFeedback false"
        );
    }

    #[test]
    fn effect_give_hidden_test() {
        assert_eq!(
            effect_give_hidden(Selector::self_(), "minecraft:speed", 30, 1),
            "effect give @s minecraft:speed 30 1 true"
        );
    }

    #[test]
    fn reload_and_seed_and_list() {
        assert_eq!(reload(), "reload");
        assert_eq!(seed(), "seed");
        assert_eq!(list(), "list");
    }
}
