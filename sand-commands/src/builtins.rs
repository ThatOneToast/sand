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

use crate::selector::Selector;
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

/// `damage <target> <amount> <damage_type>` — deal damage to entities (1.19.4+).
pub fn damage(target: Selector, amount: f64, damage_type: impl Into<String>) -> String {
    format!("damage {} {} {}", target, amount, damage_type.into())
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
    fn schedule_test() {
        assert_eq!(
            schedule_replace("my_pack:delayed", "20t"),
            "schedule function my_pack:delayed 20t replace"
        );
    }
}
