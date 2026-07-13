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
use crate::error::CommandResult;
use crate::selector::{EntityTargets, Selector, SingleEntity};
use crate::text::TextComponent;
use crate::validate;

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
///
/// Raw/unchecked: accepts non-finite coordinates and any string entity type,
/// which can produce command text Minecraft rejects (`NaN`/`inf` coordinates)
/// or silently fails to summon (an unknown entity id). Prefer
/// [`try_summon`] on the validated path.
pub fn summon(entity_type: impl Into<String>, x: f64, y: f64, z: f64) -> String {
    format!("summon {} {} {} {}", entity_type.into(), x, y, z)
}

/// Fallible [`summon`] — rejects non-finite coordinates and an empty/malformed
/// entity type before producing command text.
pub fn try_summon(entity_type: impl Into<String>, x: f64, y: f64, z: f64) -> CommandResult<String> {
    let entity_type = entity_type.into();
    validate::resource_location_shape(&entity_type, "summon", "entity_type")?;
    validate::finite(x, "summon", "x")?;
    validate::finite(y, "summon", "y")?;
    validate::finite(z, "summon", "z")?;
    Ok(format!("summon {entity_type} {x} {y} {z}"))
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
///
/// Raw/unchecked: accepts non-finite coordinates. Prefer [`try_tp`] on the
/// validated path.
pub fn tp(target: Selector, x: f64, y: f64, z: f64) -> String {
    format!("tp {} {} {} {}", target, x, y, z)
}

/// Fallible [`tp`] — rejects non-finite coordinates before producing command text.
pub fn try_tp(target: Selector, x: f64, y: f64, z: f64) -> CommandResult<String> {
    validate::finite(x, "tp", "x")?;
    validate::finite(y, "tp", "y")?;
    validate::finite(z, "tp", "z")?;
    Ok(format!("tp {target} {x} {y} {z}"))
}

/// `tp <target> ~<dx> ~<dy> ~<dz>` — teleport the target by a relative offset.
///
/// Raw/unchecked: accepts non-finite offsets. Prefer [`try_tp_relative`] on
/// the validated path.
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

/// Fallible [`tp_relative`] — rejects non-finite offsets before producing
/// command text.
pub fn try_tp_relative(target: Selector, dx: f64, dy: f64, dz: f64) -> CommandResult<String> {
    validate::finite(dx, "tp_relative", "dx")?;
    validate::finite(dy, "tp_relative", "dy")?;
    validate::finite(dz, "tp_relative", "dz")?;
    Ok(tp_relative(target, dx, dy, dz))
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
///
/// Raw/unchecked: accepts empty tags or tags containing whitespace/control
/// characters, which Minecraft's command grammar splits on. Prefer
/// [`try_tag_add`] on the validated path.
pub fn tag_add(selector: Selector, tag: impl Into<String>) -> String {
    format!("tag {} add {}", selector, tag.into())
}

/// Fallible [`tag_add`] — rejects empty tags and tags containing
/// whitespace/control characters.
pub fn try_tag_add(selector: Selector, tag: impl Into<String>) -> CommandResult<String> {
    let tag = tag.into();
    validate::no_whitespace_or_control(&tag, "tag_add", "tag")?;
    Ok(format!("tag {selector} add {tag}"))
}

/// `tag <selector> remove <tag>` — remove a tag from matching entities.
///
/// Raw/unchecked: see [`tag_add`]. Prefer [`try_tag_remove`] on the validated path.
pub fn tag_remove(selector: Selector, tag: impl Into<String>) -> String {
    format!("tag {} remove {}", selector, tag.into())
}

/// Fallible [`tag_remove`] — rejects empty tags and tags containing
/// whitespace/control characters.
pub fn try_tag_remove(selector: Selector, tag: impl Into<String>) -> CommandResult<String> {
    let tag = tag.into();
    validate::no_whitespace_or_control(&tag, "tag_remove", "tag")?;
    Ok(format!("tag {selector} remove {tag}"))
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
///
/// Raw/unchecked: accepts any string, including modes vanilla doesn't
/// recognize. Prefer the fixed-mode helpers above, or [`try_gamemode`] on the
/// validated path.
pub fn gamemode(mode: impl Into<String>, selector: Selector) -> String {
    format!("gamemode {} {}", mode.into(), selector)
}

/// The four vanilla gamemode names accepted by [`try_gamemode`].
pub const VALID_GAMEMODES: &[&str] = &["survival", "creative", "adventure", "spectator"];

/// Fallible [`gamemode`] — rejects any mode string outside
/// [`VALID_GAMEMODES`].
pub fn try_gamemode(mode: impl Into<String>, selector: Selector) -> CommandResult<String> {
    let mode = mode.into();
    if !VALID_GAMEMODES.contains(&mode.as_str()) {
        return Err(crate::error::CommandError::new(
            "gamemode",
            "mode",
            format!(
                "must be one of {VALID_GAMEMODES:?}, got `{mode}` — \
                 or use gamemode_survival/creative/adventure/spectator(...)"
            ),
        ));
    }
    Ok(format!("gamemode {mode} {selector}"))
}

// ── Effects ───────────────────────────────────────────────────────────────────

/// `effect give <selector> <effect> <duration> <amplifier>` — apply a status effect.
///
/// Raw/unchecked: accepts any string effect id. Prefer [`try_effect_give`] on
/// the validated path.
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

/// Fallible [`effect_give`] — rejects an effect id that isn't a valid
/// `namespace:path` resource location.
pub fn try_effect_give(
    selector: Selector,
    effect: impl Into<String>,
    duration: u32,
    amplifier: u8,
) -> CommandResult<String> {
    let effect = effect.into();
    validate::resource_location_shape(&effect, "effect_give", "effect")?;
    Ok(format!(
        "effect give {selector} {effect} {duration} {amplifier}"
    ))
}

/// `effect give <selector> <effect> <duration> <amplifier> true` — hide particles.
///
/// Raw/unchecked: accepts any string effect id. Prefer
/// [`try_effect_give_hidden`] on the validated path.
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

/// Fallible [`effect_give_hidden`] — rejects an effect id that isn't a valid
/// `namespace:path` resource location.
pub fn try_effect_give_hidden(
    selector: Selector,
    effect: impl Into<String>,
    duration: u32,
    amplifier: u8,
) -> CommandResult<String> {
    let effect = effect.into();
    validate::resource_location_shape(&effect, "effect_give_hidden", "effect")?;
    Ok(format!(
        "effect give {selector} {effect} {duration} {amplifier} true"
    ))
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
///
/// Raw/unchecked: accepts empty/whitespace team names. Prefer [`try_team_add`]
/// on the validated path.
pub fn team_add(team: impl Into<String>) -> String {
    format!("team add {}", team.into())
}

/// Fallible [`team_add`] — rejects empty team names or names containing
/// whitespace/control characters.
pub fn try_team_add(team: impl Into<String>) -> CommandResult<String> {
    let team = team.into();
    validate::no_whitespace_or_control(&team, "team_add", "team")?;
    Ok(format!("team add {team}"))
}

/// `team remove <team>` — delete a team.
///
/// Raw/unchecked: see [`team_add`]. Prefer [`try_team_remove`] on the validated path.
pub fn team_remove(team: impl Into<String>) -> String {
    format!("team remove {}", team.into())
}

/// Fallible [`team_remove`] — rejects empty team names or names containing
/// whitespace/control characters.
pub fn try_team_remove(team: impl Into<String>) -> CommandResult<String> {
    let team = team.into();
    validate::no_whitespace_or_control(&team, "team_remove", "team")?;
    Ok(format!("team remove {team}"))
}

/// `team join <team> <selector>` — add entities to a team.
///
/// Raw/unchecked: see [`team_add`]. Prefer [`try_team_join`] on the validated path.
pub fn team_join(team: impl Into<String>, selector: Selector) -> String {
    format!("team join {} {}", team.into(), selector)
}

/// Fallible [`team_join`] — rejects empty team names or names containing
/// whitespace/control characters.
pub fn try_team_join(team: impl Into<String>, selector: Selector) -> CommandResult<String> {
    let team = team.into();
    validate::no_whitespace_or_control(&team, "team_join", "team")?;
    Ok(format!("team join {team} {selector}"))
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
///
/// Raw/unchecked: accepts any string, including levels vanilla doesn't
/// recognize. Prefer [`try_difficulty`] on the validated path.
pub fn difficulty(level: impl Into<String>) -> String {
    format!("difficulty {}", level.into())
}

/// The four vanilla difficulty names accepted by [`try_difficulty`].
pub const VALID_DIFFICULTIES: &[&str] = &["peaceful", "easy", "normal", "hard"];

/// Fallible [`difficulty`] — rejects any level string outside
/// [`VALID_DIFFICULTIES`].
pub fn try_difficulty(level: impl Into<String>) -> CommandResult<String> {
    let level = level.into();
    if !VALID_DIFFICULTIES.contains(&level.as_str()) {
        return Err(crate::error::CommandError::new(
            "difficulty",
            "level",
            format!("must be one of {VALID_DIFFICULTIES:?}, got `{level}`"),
        ));
    }
    Ok(format!("difficulty {level}"))
}

// ── Functions / scheduling ────────────────────────────────────────────────────

/// `function <namespace:path>` — run a datapack function.
///
/// Raw/unchecked: accepts any string, including a malformed resource
/// location. Prefer [`try_function`] on the validated path.
pub fn function(id: impl Into<String>) -> String {
    format!("function {}", id.into())
}

/// Fallible [`function`] — rejects an id that isn't a valid `namespace:path`
/// resource location.
pub fn try_function(id: impl Into<String>) -> CommandResult<String> {
    let id = id.into();
    validate::resource_location_shape(&id, "function", "id")?;
    Ok(format!("function {id}"))
}

/// `schedule function <id> <time> [append|replace]` — schedule a function.
///
/// Raw/unchecked: accepts any string id. Prefer [`try_schedule`] on the
/// validated path.
pub fn schedule(id: impl Into<String>, time: impl Into<String>, mode: impl Into<String>) -> String {
    format!(
        "schedule function {} {} {}",
        id.into(),
        time.into(),
        mode.into()
    )
}

/// Fallible [`schedule`] — rejects an id that isn't a valid `namespace:path`
/// resource location.
pub fn try_schedule(
    id: impl Into<String>,
    time: impl Into<String>,
    mode: impl Into<String>,
) -> CommandResult<String> {
    let id = id.into();
    validate::resource_location_shape(&id, "schedule", "id")?;
    Ok(format!(
        "schedule function {id} {} {}",
        time.into(),
        mode.into()
    ))
}

/// `schedule function <id> <time> replace` — schedule (replace any existing).
pub fn schedule_replace(id: impl Into<String>, time: impl Into<String>) -> String {
    schedule(id, time, "replace")
}

/// Fallible [`schedule_replace`] — rejects an id that isn't a valid
/// `namespace:path` resource location.
pub fn try_schedule_replace(
    id: impl Into<String>,
    time: impl Into<String>,
) -> CommandResult<String> {
    try_schedule(id, time, "replace")
}

/// `schedule clear <id>` — cancel a scheduled function.
///
/// Raw/unchecked: accepts any string id. Prefer [`try_schedule_clear`] on the
/// validated path.
pub fn schedule_clear(id: impl Into<String>) -> String {
    format!("schedule clear {}", id.into())
}

/// Fallible [`schedule_clear`] — rejects an id that isn't a valid
/// `namespace:path` resource location.
pub fn try_schedule_clear(id: impl Into<String>) -> CommandResult<String> {
    let id = id.into();
    validate::resource_location_shape(&id, "schedule_clear", "id")?;
    Ok(format!("schedule clear {id}"))
}

/// `reload` — reload all datapacks.
pub fn reload() -> String {
    "reload".to_string()
}

// ── Gamerules ─────────────────────────────────────────────────────────────────

/// `gamerule <rule> <value>` — set a gamerule.
///
/// Raw/unchecked: accepts an empty rule name. Prefer [`try_gamerule`] on the
/// validated path.
pub fn gamerule(rule: impl Into<String>, value: impl Into<String>) -> String {
    format!("gamerule {} {}", rule.into(), value.into())
}

/// Fallible [`gamerule`] — rejects an empty rule name or a value containing
/// whitespace/control characters.
pub fn try_gamerule(rule: impl Into<String>, value: impl Into<String>) -> CommandResult<String> {
    let rule = rule.into();
    let value = value.into();
    validate::no_whitespace_or_control(&rule, "gamerule", "rule")?;
    validate::no_whitespace_or_control(&value, "gamerule", "value")?;
    Ok(format!("gamerule {rule} {value}"))
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

/// Fallible [`damage`] — rejects a non-finite amount or a damage type that
/// isn't a valid `namespace:path` resource location.
pub fn try_damage(
    target: impl Into<SingleEntity>,
    amount: f64,
    damage_type: impl Into<String>,
) -> CommandResult<String> {
    let target = target.into();
    let damage_type = damage_type.into();
    validate::finite(amount, "damage", "amount")?;
    validate::resource_location_shape(&damage_type, "damage", "damage_type")?;
    Ok(format!("damage {target} {amount} {damage_type}"))
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
    ///
    /// Raw/unchecked: accepts non-finite amounts (`NaN`/`±inf`), which
    /// [`Damage::run`] would format directly into command text. Prefer
    /// [`try_fixed`](Self::try_fixed) on the validated path, or
    /// [`Damage::try_run`] to validate at build time.
    pub fn fixed(amount: f64) -> Self {
        Self::Fixed(amount)
    }

    /// Fallible [`fixed`](Self::fixed) — rejects a non-finite amount.
    pub fn try_fixed(amount: f64) -> CommandResult<Self> {
        validate::finite(amount, "DamageAmount::try_fixed", "amount")?;
        Ok(Self::Fixed(amount))
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
    ///
    /// Raw/unchecked: accepts a non-finite [`DamageAmount`] and an empty/
    /// malformed `damage_type`. Prefer [`try_run`](Self::try_run) on the
    /// validated path.
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

    /// Fallible [`run`](Self::run) — rejects a non-finite damage amount or a
    /// `damage_type` that isn't a valid `namespace:path` resource location
    /// before producing command text.
    pub fn try_run(self) -> CommandResult<Vec<String>> {
        validate::finite(self.amount.as_fixed(), "Damage::try_run", "amount")?;
        validate::resource_location_shape(&self.damage_type, "Damage::try_run", "damage_type")?;
        Ok(self.run())
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
///
/// Raw/unchecked: accepts a non-finite value and any string attribute id.
/// Prefer [`try_attribute_base_set`] on the validated path.
pub fn attribute_base_set(target: Selector, attribute: impl Into<String>, value: f64) -> String {
    format!(
        "attribute {} {} base set {}",
        target,
        attribute.into(),
        value
    )
}

/// Fallible [`attribute_base_set`] — rejects a non-finite value or an
/// attribute id that isn't a valid `namespace:path` resource location.
pub fn try_attribute_base_set(
    target: Selector,
    attribute: impl Into<String>,
    value: f64,
) -> CommandResult<String> {
    let attribute = attribute.into();
    validate::resource_location_shape(&attribute, "attribute_base_set", "attribute")?;
    validate::finite(value, "attribute_base_set", "value")?;
    Ok(format!("attribute {target} {attribute} base set {value}"))
}

// ── Misc ──────────────────────────────────────────────────────────────────────

/// `clear <selector>` — clear the entire inventory of matching entities.
pub fn clear(selector: Selector) -> String {
    format!("clear {}", selector)
}

/// `clear <selector> <item>` — remove all of a specific item from matching entities.
///
/// Raw/unchecked: accepts an empty item id/component string. Prefer
/// [`try_clear_item`] on the validated path.
pub fn clear_item(selector: Selector, item: impl Into<String>) -> String {
    format!("clear {} {}", selector, item.into())
}

/// Fallible [`clear_item`] — rejects an empty item string.
///
/// This only checks non-emptiness, not full item-component syntax — an item
/// string produced by `CustomItem` should be validated with
/// `CustomItem::validate()`/`try_to_string()` (in `sand-components`) before
/// it ever reaches this helper.
pub fn try_clear_item(selector: Selector, item: impl Into<String>) -> CommandResult<String> {
    let item = item.into();
    validate::non_empty(&item, "clear_item", "item")?;
    Ok(format!("clear {selector} {item}"))
}

/// `give <selector> <item>` — give an item to matching entities.
///
/// Raw/unchecked: accepts an empty item id/component string. Prefer
/// [`try_give`] on the validated path.
pub fn give(selector: Selector, item: impl Into<String>) -> String {
    format!("give {} {}", selector, item.into())
}

/// Fallible [`give`] — rejects an empty item string. See [`try_clear_item`]
/// for the item-component validation boundary this helper does not duplicate.
pub fn try_give(selector: Selector, item: impl Into<String>) -> CommandResult<String> {
    let item = item.into();
    validate::non_empty(&item, "give", "item")?;
    Ok(format!("give {selector} {item}"))
}

/// `give <selector> <item> <count>` — give a stack of items.
///
/// Raw/unchecked: accepts an empty item id and a `count` of `0` (a no-op
/// vanilla accepts but that likely indicates an authoring mistake). Prefer
/// [`try_give_count`] on the validated path.
pub fn give_count(selector: Selector, item: impl Into<String>, count: u32) -> String {
    format!("give {} {} {}", selector, item.into(), count)
}

/// Fallible [`give_count`] — rejects an empty item string or a zero count.
pub fn try_give_count(
    selector: Selector,
    item: impl Into<String>,
    count: u32,
) -> CommandResult<String> {
    let item = item.into();
    validate::non_empty(&item, "give_count", "item")?;
    validate::positive_u32(count, "give_count", "count")?;
    Ok(format!("give {selector} {item} {count}"))
}

/// `setblock <x> <y> <z> <block>` — place a block at absolute coordinates.
///
/// Raw/unchecked: accepts an empty block id. Prefer [`try_setblock_abs`] on
/// the validated path.
pub fn setblock_abs(x: i32, y: i32, z: i32, block: impl Into<String>) -> String {
    format!("setblock {} {} {} {}", x, y, z, block.into())
}

/// Fallible [`setblock_abs`] — rejects an empty block id.
pub fn try_setblock_abs(x: i32, y: i32, z: i32, block: impl Into<String>) -> CommandResult<String> {
    let block = block.into();
    validate::non_empty(&block, "setblock_abs", "block")?;
    Ok(format!("setblock {x} {y} {z} {block}"))
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
///
/// Raw/unchecked: accepts an empty player name. Prefer [`try_kick`] on the
/// validated path.
pub fn kick(player: impl Into<String>, reason: Option<&str>) -> String {
    match reason {
        Some(r) => format!("kick {} {}", player.into(), r),
        None => format!("kick {}", player.into()),
    }
}

/// Fallible [`kick`] — rejects an empty player name.
pub fn try_kick(player: impl Into<String>, reason: Option<&str>) -> CommandResult<String> {
    let player = player.into();
    validate::non_empty(&player, "kick", "player")?;
    Ok(match reason {
        Some(r) => format!("kick {player} {r}"),
        None => format!("kick {player}"),
    })
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

    // ── #170: try_* validated-path positive cases (unchanged text vs raw) ─────

    #[test]
    fn try_summon_matches_raw_for_valid_input() {
        assert_eq!(
            try_summon("minecraft:zombie", 1.0, 2.0, 3.0).unwrap(),
            summon("minecraft:zombie", 1.0, 2.0, 3.0)
        );
    }

    #[test]
    fn try_tp_matches_raw_for_valid_input() {
        assert_eq!(
            try_tp(Selector::self_(), 1.0, 2.0, 3.0).unwrap(),
            tp(Selector::self_(), 1.0, 2.0, 3.0)
        );
    }

    #[test]
    fn try_tag_add_matches_raw_for_valid_input() {
        assert_eq!(
            try_tag_add(Selector::self_(), "ready").unwrap(),
            tag_add(Selector::self_(), "ready")
        );
    }

    #[test]
    fn try_gamemode_matches_raw_for_valid_mode() {
        assert_eq!(
            try_gamemode("creative", Selector::self_()).unwrap(),
            gamemode("creative", Selector::self_())
        );
    }

    #[test]
    fn try_give_count_matches_raw_for_valid_input() {
        assert_eq!(
            try_give_count(Selector::self_(), "minecraft:apple", 1).unwrap(),
            give_count(Selector::self_(), "minecraft:apple", 1)
        );
    }

    // ── #170: try_* negative/regression cases ──────────────────────────────────

    #[test]
    fn try_summon_rejects_non_finite_coordinates() {
        assert!(try_summon("minecraft:zombie", f64::NAN, 0.0, 0.0).is_err());
        assert!(try_summon("minecraft:zombie", 0.0, f64::INFINITY, 0.0).is_err());
        assert!(try_summon("minecraft:zombie", 0.0, 0.0, f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn try_summon_rejects_malformed_entity_type() {
        assert!(try_summon("", 0.0, 0.0, 0.0).is_err());
        assert!(try_summon("not a resource location", 0.0, 0.0, 0.0).is_err());
    }

    #[test]
    fn try_tp_rejects_non_finite_coordinates() {
        assert!(try_tp(Selector::self_(), f64::NAN, 0.0, 0.0).is_err());
        let err = try_tp(Selector::self_(), f64::NAN, 0.0, 0.0).unwrap_err();
        assert_eq!(err.helper, "tp");
        assert_eq!(err.field, "x");
    }

    #[test]
    fn try_tp_relative_rejects_non_finite_offsets() {
        assert!(try_tp_relative(Selector::self_(), f64::NAN, 0.0, 0.0).is_err());
        assert!(try_tp_relative(Selector::self_(), 0.0, f64::INFINITY, 0.0).is_err());
    }

    #[test]
    fn try_tag_add_rejects_empty_and_whitespace_tags() {
        assert!(try_tag_add(Selector::self_(), "").is_err());
        assert!(try_tag_add(Selector::self_(), "has space").is_err());
        assert!(try_tag_remove(Selector::self_(), "").is_err());
    }

    #[test]
    fn try_gamemode_rejects_unknown_mode() {
        let err = try_gamemode("godmode", Selector::self_()).unwrap_err();
        assert_eq!(err.helper, "gamemode");
        assert!(err.message.contains("godmode"));
    }

    #[test]
    fn try_effect_give_rejects_malformed_effect_id() {
        assert!(try_effect_give(Selector::self_(), "", 100, 1).is_err());
        assert!(try_effect_give(Selector::self_(), "speed", 100, 1).is_err());
        assert!(try_effect_give(Selector::self_(), "minecraft:speed", 100, 1).is_ok());
    }

    #[test]
    fn try_team_add_rejects_empty_and_whitespace_names() {
        assert!(try_team_add("").is_err());
        assert!(try_team_add("red team").is_err());
        assert!(try_team_add("red").is_ok());
    }

    #[test]
    fn try_difficulty_rejects_unknown_level() {
        assert!(try_difficulty("nightmare").is_err());
        assert!(try_difficulty("hard").is_ok());
    }

    #[test]
    fn try_function_and_schedule_reject_malformed_ids() {
        assert!(try_function("not valid").is_err());
        assert!(try_function("my_pack:tick").is_ok());
        assert!(try_schedule("bad id", "20t", "replace").is_err());
        assert!(try_schedule_replace("my_pack:delayed", "20t").is_ok());
        assert!(try_schedule_clear("bad id").is_err());
    }

    #[test]
    fn try_gamerule_rejects_empty_rule() {
        assert!(try_gamerule("", "true").is_err());
        assert!(try_gamerule("keepInventory", "true").is_ok());
    }

    #[test]
    fn try_damage_rejects_non_finite_amount_and_bad_type() {
        assert!(try_damage(Selector::self_(), f64::NAN, "minecraft:generic").is_err());
        assert!(try_damage(Selector::self_(), 5.0, "generic").is_err());
        assert!(try_damage(Selector::self_(), 5.0, "minecraft:generic").is_ok());
    }

    #[test]
    fn damage_amount_try_fixed_rejects_non_finite() {
        assert!(DamageAmount::try_fixed(f64::NAN).is_err());
        assert!(DamageAmount::try_fixed(5.0).is_ok());
    }

    #[test]
    fn damage_try_run_rejects_non_finite_amount() {
        let result = Damage::new()
            .amount(DamageAmount::Fixed(f64::NAN))
            .try_run();
        assert!(result.is_err());
    }

    #[test]
    fn try_attribute_base_set_rejects_non_finite_and_bad_id() {
        assert!(
            try_attribute_base_set(Selector::self_(), "minecraft:generic.max_health", f64::NAN)
                .is_err()
        );
        assert!(try_attribute_base_set(Selector::self_(), "max_health", 10.0).is_err());
        assert!(
            try_attribute_base_set(Selector::self_(), "minecraft:generic.max_health", 10.0).is_ok()
        );
    }

    #[test]
    fn try_give_rejects_empty_item() {
        assert!(try_give(Selector::self_(), "").is_err());
        assert!(try_give(Selector::self_(), "minecraft:diamond_sword").is_ok());
    }

    #[test]
    fn try_give_count_rejects_empty_item_and_zero_count() {
        assert!(try_give_count(Selector::self_(), "", 1).is_err());
        assert!(try_give_count(Selector::self_(), "minecraft:apple", 0).is_err());
    }

    #[test]
    fn try_clear_item_rejects_empty_item() {
        assert!(try_clear_item(Selector::self_(), "").is_err());
    }

    #[test]
    fn try_setblock_abs_rejects_empty_block() {
        assert!(try_setblock_abs(0, 0, 0, "").is_err());
        assert!(try_setblock_abs(0, 0, 0, "minecraft:stone").is_ok());
    }

    #[test]
    fn try_kick_rejects_empty_player() {
        assert!(try_kick("", None).is_err());
        assert!(try_kick("Steve", Some("griefing")).is_ok());
    }

    #[test]
    fn command_error_message_identifies_helper_and_field() {
        let err = try_tp(Selector::self_(), f64::NAN, 0.0, 0.0).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("tp"), "{msg}");
        assert!(msg.contains('x'), "{msg}");
    }
}
