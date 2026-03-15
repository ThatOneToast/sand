//! The `execute` command chain builder.
//!
//! `execute` is a special command that chains sub-commands before a final `run`
//! clause. It's handled manually rather than auto-generated because its tree
//! contains redirects (it recurses back into itself).
//!
//! # Examples
//! ```rust,ignore
//! use sand_core::cmd::{Execute, Selector, BlockPos, Anchor, cmd};
//!
//! // execute as @a at @s run kill @s
//! Execute::new()
//!     .as_(Selector::all_players())
//!     .at(Selector::self_())
//!     .run(cmd::kill(Selector::self_()));
//!
//! // execute if entity @a[tag=ready] run say ready!
//! Execute::new()
//!     .if_entity(Selector::all_players().tag("ready"))
//!     .run(cmd::say("ready!"));
//! ```
//!
//! # Common patterns
//!
//! ## All nearby entities except self
//!
//! Minecraft has no "not me" selector filter. The standard approach is a
//! temporary tag: mark the caster, target entities without that tag, then
//! remove the tag. This produces three commands:
//!
//! ```rust,ignore
//! cmd::tag_add(Selector::self_(), "__caster"),
//! Execute::new()
//!     .at(Selector::self_())
//!     .as_(Selector::all_entities().distance_max(4.0).not_tag("__caster"))
//!     .run(cmd::damage(Selector::self_(), 1.0, "generic")),
//! cmd::tag_remove(Selector::self_(), "__caster"),
//! ```
//!
//! ## Friendly-fire prevention (team check)
//!
//! After `as @e[...]`, `@s` becomes each iterated entity. Use
//! [`if_on_team`](Execute::if_on_team) / [`unless_on_team`](Execute::unless_on_team)
//! to filter by that entity's team. To also check the *caster's* team you need a
//! separate `if entity` / `unless entity` clause with the caster tag:
//!
//! ```rust,ignore
//! // Example: damage all enemies in range, skipping teammates.
//! // "red" and "blue" are the two team names in this example.
//! cmd::tag_add(Selector::self_(), "__caster"),
//! Execute::new().at(Selector::self_())
//!     .as_(Selector::all_entities().distance_max(4.0).not_tag("__caster"))
//!     // Skip target if target is on red AND the caster is also on red
//!     .unless_entity(
//!         Selector::all_players().tag("__caster").team("red")  // caster is red
//!             // This is a separate condition — chain it via if_on_team below:
//!     ),
//! // Simpler: fire two execute chains, one per team:
//! Execute::new().at(Selector::self_())
//!     .as_(Selector::all_entities().distance_max(4.0).not_tag("__caster"))
//!     .unless_entity(Selector::self_().team("red"))  // target not on red
//!     .run(cmd::damage(Selector::self_(), 1.0, "generic")),
//! Execute::new().at(Selector::self_())
//!     .as_(Selector::all_entities().distance_max(4.0).not_tag("__caster").team("red"))
//!     .unless_entity(Selector::all_players().tag("__caster").team("red"))
//!     .run(cmd::damage(Selector::self_(), 1.0, "generic")),
//! cmd::tag_remove(Selector::self_(), "__caster"),
//! ```

use std::fmt;

use super::{Anchor, BlockPos, Command, Rotation, ScoreHolder, ScoreOp, Selector, Swizzle, Vec3};

/// Builder for the `execute` command chain.
///
/// Call builder methods to add sub-commands, then call [`run`](Execute::run)
/// to complete the command.
#[derive(Debug, Clone, Default)]
pub struct Execute {
    parts: Vec<String>,
}

impl Execute {
    pub fn new() -> Self {
        Self { parts: vec![] }
    }

    // ── Context sub-commands ──────────────────────────────────────────────────

    /// `as <selector>` — change the executing entity.
    pub fn as_(mut self, selector: Selector) -> Self {
        self.parts.push(format!("as {selector}"));
        self
    }

    /// `at <selector>` — change position/rotation to match entity.
    pub fn at(mut self, selector: Selector) -> Self {
        self.parts.push(format!("at {selector}"));
        self
    }

    /// `positioned <pos>` — change position.
    pub fn positioned(mut self, pos: Vec3) -> Self {
        self.parts.push(format!("positioned {pos}"));
        self
    }

    /// `positioned as <selector>` — change position to match entity.
    pub fn positioned_as(mut self, selector: Selector) -> Self {
        self.parts.push(format!("positioned as {selector}"));
        self
    }

    /// `rotated <yaw> <pitch>` — change rotation.
    pub fn rotated(mut self, rotation: Rotation) -> Self {
        self.parts.push(format!("rotated {rotation}"));
        self
    }

    /// `rotated as <selector>` — change rotation to match entity.
    pub fn rotated_as(mut self, selector: Selector) -> Self {
        self.parts.push(format!("rotated as {selector}"));
        self
    }

    /// `facing <pos>` — rotate to face a position.
    pub fn facing(mut self, pos: Vec3) -> Self {
        self.parts.push(format!("facing {pos}"));
        self
    }

    /// `facing entity <selector> <anchor>` — rotate to face an entity.
    pub fn facing_entity(mut self, selector: Selector, anchor: Anchor) -> Self {
        self.parts
            .push(format!("facing entity {selector} {anchor}"));
        self
    }

    /// `in <dimension>` — change dimension.
    pub fn in_(mut self, dimension: impl Into<String>) -> Self {
        self.parts.push(format!("in {}", dimension.into()));
        self
    }

    /// `align <axes>` — floor coordinates to block grid (e.g. `"xy"`).
    pub fn align(mut self, axes: Swizzle) -> Self {
        self.parts.push(format!("align {axes}"));
        self
    }

    /// `anchored <anchor>` — change anchor point.
    pub fn anchored(mut self, anchor: Anchor) -> Self {
        self.parts.push(format!("anchored {anchor}"));
        self
    }

    /// `on <relation>` — follow an entity relationship (attacker, vehicle, etc.).
    pub fn on(mut self, relation: impl Into<String>) -> Self {
        self.parts.push(format!("on {}", relation.into()));
        self
    }

    /// `summon <entity_type>` — summon an entity and execute as it.
    pub fn summon(mut self, entity_type: impl Into<String>) -> Self {
        self.parts.push(format!("summon {}", entity_type.into()));
        self
    }

    // ── Condition sub-commands ────────────────────────────────────────────────

    /// `if entity <selector>` — execute only if the selector matches.
    pub fn if_entity(mut self, selector: Selector) -> Self {
        self.parts.push(format!("if entity {selector}"));
        self
    }

    /// `unless entity <selector>` — execute only if NO entity matches.
    pub fn unless_entity(mut self, selector: Selector) -> Self {
        self.parts.push(format!("unless entity {selector}"));
        self
    }

    /// `if entity @s[team=<team>]` — continue only if the current executing entity
    /// is on the given team.
    ///
    /// After `execute as @e[...]`, `@s` is each iterated entity, so this checks
    /// **the target entity's** team — not the original executor's team.
    pub fn if_on_team(mut self, team: impl Into<String>) -> Self {
        self.parts
            .push(format!("if entity @s[team={}]", team.into()));
        self
    }

    /// `unless entity @s[team=<team>]` — skip if the current executing entity is
    /// on the given team.
    ///
    /// See [`if_on_team`](Execute::if_on_team) for the semantics of `@s`.
    pub fn unless_on_team(mut self, team: impl Into<String>) -> Self {
        self.parts
            .push(format!("unless entity @s[team={}]", team.into()));
        self
    }

    pub fn unless_score(
        mut self,
        primary_selector: Selector,
        primary: impl Into<String>,
        secondary_selector: Selector,
        secondary: impl Into<String>,
    ) -> Self {
        let primary = primary.into();
        let secondary = secondary.into();
        self.parts.push(format!(
            "unless score {primary_selector} {primary} = {secondary_selector} {secondary}"
        ));
        self
    }

    /// `if block <pos> <block>` — execute only if block at pos matches.
    pub fn if_block(mut self, pos: BlockPos, block: impl Into<String>) -> Self {
        self.parts.push(format!("if block {pos} {}", block.into()));
        self
    }

    /// `unless block <pos> <block>`.
    pub fn unless_block(mut self, pos: BlockPos, block: impl Into<String>) -> Self {
        self.parts
            .push(format!("unless block {pos} {}", block.into()));
        self
    }

    /// `if score <holder> <obj> matches <range>` — check score against a range.
    pub fn if_score_matches(
        mut self,
        holder: impl Into<String>,
        objective: impl Into<String>,
        range: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "if score {} {} matches {}",
            holder.into(),
            objective.into(),
            range.into()
        ));
        self
    }

    /// `unless score <holder> <obj> matches <range>`.
    pub fn unless_score_matches(
        mut self,
        holder: impl Into<String>,
        objective: impl Into<String>,
        range: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "unless score {} {} matches {}",
            holder.into(),
            objective.into(),
            range.into()
        ));
        self
    }

    /// `if score <a> <a_obj> <op> <b> <b_obj>` — compare two scores.
    pub fn if_score_compare(
        mut self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        op: ScoreOp,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "if score {} {} {op} {} {}",
            a.into(),
            a_obj.into(),
            b.into(),
            b_obj.into()
        ));
        self
    }

    /// `unless score <a> <a_obj> <op> <b> <b_obj>`.
    pub fn unless_score_compare(
        mut self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        op: ScoreOp,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "unless score {} {} {op} {} {}",
            a.into(),
            a_obj.into(),
            b.into(),
            b_obj.into()
        ));
        self
    }

    /// `if predicate <predicate>` — check a loot predicate.
    pub fn if_predicate(mut self, predicate: impl Into<String>) -> Self {
        self.parts
            .push(format!("if predicate {}", predicate.into()));
        self
    }

    /// Append a raw condition fragment (e.g. from [`Objective::if_matches`]).
    ///
    /// Use this to attach condition strings produced by other builders without
    /// repeating the `if`/`unless` keyword manually.
    ///
    /// ```rust,ignore
    /// Execute::new()
    ///     .if_(COOLDOWN.if_matches(ScoreHolder::self_(), "0"))
    ///     .run(cmd::say("ready!"))
    /// ```
    pub fn if_(mut self, condition: impl Into<String>) -> Self {
        self.parts.push(condition.into());
        self
    }

    // ── Items conditions (1.20.5+) ────────────────────────────────────────────

    /// `if items entity <selector> <slot> <item>` — continues only if the slot
    /// holds an item matching the predicate (1.20.5+).
    pub fn if_items(
        mut self,
        selector: Selector,
        slot: super::inventory::InventorySlot,
        item: impl Into<String>,
    ) -> Self {
        self.parts
            .push(format!("if items entity {selector} {slot} {}", item.into()));
        self
    }

    /// `unless items entity <selector> <slot> <item>` (1.20.5+).
    pub fn unless_items(
        mut self,
        selector: Selector,
        slot: super::inventory::InventorySlot,
        item: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "unless items entity {selector} {slot} {}",
            item.into()
        ));
        self
    }

    /// `if items entity <selector> <slot_pattern> <item>` with a wildcard pattern (1.20.5+).
    ///
    /// ```rust,ignore
    /// Execute::new()
    ///     .if_items_pattern(Selector::self_(), SlotPattern::AnyHotbar, "minecraft:torch")
    ///     .run_raw("say has torch in hotbar")
    /// ```
    pub fn if_items_pattern(
        mut self,
        selector: Selector,
        pattern: super::inventory::SlotPattern,
        item: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "if items entity {selector} {pattern} {}",
            item.into()
        ));
        self
    }

    /// `unless items entity <selector> <slot_pattern> <item>` with a wildcard pattern (1.20.5+).
    pub fn unless_items_pattern(
        mut self,
        selector: Selector,
        pattern: super::inventory::SlotPattern,
        item: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "unless items entity {selector} {pattern} {}",
            item.into()
        ));
        self
    }

    // ── Store sub-commands ────────────────────────────────────────────────────

    /// `store result score <holder> <objective>` — store result of `run` into a score.
    pub fn store_result_score(mut self, holder: ScoreHolder, objective: impl Into<String>) -> Self {
        self.parts
            .push(format!("store result score {holder} {}", objective.into()));
        self
    }

    /// `store success score <holder> <objective>`.
    pub fn store_success_score(
        mut self,
        holder: ScoreHolder,
        objective: impl Into<String>,
    ) -> Self {
        self.parts
            .push(format!("store success score {holder} {}", objective.into()));
        self
    }

    // ── Terminal ──────────────────────────────────────────────────────────────

    /// `run <command>` — execute the given command in the new context.
    ///
    /// This finalizes the execute chain and returns the full command string.
    pub fn run(mut self, cmd: impl Command) -> String {
        self.parts.push(format!("run {cmd}"));
        format!("execute {}", self.parts.join(" "))
    }

    /// Like [`run`](Execute::run) but accepts a raw string instead of a typed command.
    pub fn run_raw(mut self, cmd: impl Into<String>) -> String {
        self.parts.push(format!("run {}", cmd.into()));
        format!("execute {}", self.parts.join(" "))
    }
}

impl fmt::Display for Execute {
    /// Renders the execute chain WITHOUT a `run` clause (useful for partial chains).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "execute {}", self.parts.join(" "))
    }
}

impl Command for Execute {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_run() {
        let s = Execute::new()
            .as_(Selector::all_players())
            .run_raw("say hi");
        assert_eq!(s, "execute as @a run say hi");
    }

    #[test]
    fn chained_conditions() {
        let s = Execute::new()
            .as_(Selector::all_players())
            .at(Selector::self_())
            .if_score_matches("@s", "playtime", "100..")
            .run_raw("say milestone!");
        assert_eq!(
            s,
            "execute as @a at @s if score @s playtime matches 100.. run say milestone!"
        );
    }

    #[test]
    fn store_result() {
        let s = Execute::new()
            .store_result_score(ScoreHolder::entity(Selector::self_()), "my_score")
            .run_raw("data get entity @s Health");
        assert_eq!(
            s,
            "execute store result score @s my_score run data get entity @s Health"
        );
    }
}
