//! The `execute` command chain builder.
//!
//! # Examples
//! ```rust,ignore
//! use sand_commands::{Execute, Selector, BlockPos};
//!
//! // execute as @a at @s run kill @s
//! Execute::new()
//!     .as_(Selector::all_players())
//!     .at(Selector::self_())
//!     .run("kill @s");
//!
//! // execute if entity @a[tag=ready] run say ready!
//! Execute::new()
//!     .if_entity(Selector::all_players().tag("ready"))
//!     .run("say ready!");
//! ```

use std::fmt;

use crate::Build;
use crate::coord::{BlockPos, Rotation, Vec3};
use crate::execute_args::{Anchor, ItemSlot, NbtStoreKind, Swizzle};
use crate::inventory::{InventorySlot, SlotPattern};
use crate::nbt::DataTarget;
use crate::scoreboard::{ScoreCmp, ScoreHolder};
use crate::selector::Selector;

/// Builder for the `execute` command chain.
///
/// Call builder methods to add sub-commands, then call [`run`](Execute::run) or
/// [`run_raw`](Execute::run_raw) to complete the command.
#[derive(Debug, Clone, Default)]
pub struct Execute {
    parts: Vec<String>,
}

impl Execute {
    /// Create a new `Execute` builder with no sub-commands.
    pub fn new() -> Self {
        Self { parts: vec![] }
    }

    // ── Context sub-commands ──────────────────────────────────────────────────

    /// `as <selector>` — change the executing entity.
    pub fn as_(mut self, selector: Selector) -> Self {
        self.parts.push(format!("as {selector}"));
        self
    }

    /// `at <selector>` — change position and rotation to match the selected entity.
    pub fn at(mut self, selector: Selector) -> Self {
        self.parts.push(format!("at {selector}"));
        self
    }

    /// `positioned <pos>` — change execution position to the given coordinates.
    pub fn positioned(mut self, pos: Vec3) -> Self {
        self.parts.push(format!("positioned {pos}"));
        self
    }

    /// `positioned as <selector>` — change position to match the selected entity.
    pub fn positioned_as(mut self, selector: Selector) -> Self {
        self.parts.push(format!("positioned as {selector}"));
        self
    }

    /// `rotated <rotation>` — change execution rotation.
    pub fn rotated(mut self, rotation: Rotation) -> Self {
        self.parts.push(format!("rotated {rotation}"));
        self
    }

    /// `rotated as <selector>` — change rotation to match the selected entity.
    pub fn rotated_as(mut self, selector: Selector) -> Self {
        self.parts.push(format!("rotated as {selector}"));
        self
    }

    /// `facing <pos>` — rotate execution to face a position in the world.
    pub fn facing(mut self, pos: Vec3) -> Self {
        self.parts.push(format!("facing {pos}"));
        self
    }

    /// `facing entity <selector> <anchor>` — rotate execution to face an entity's anchor point.
    pub fn facing_entity(mut self, selector: Selector, anchor: Anchor) -> Self {
        self.parts
            .push(format!("facing entity {selector} {anchor}"));
        self
    }

    /// `in <dimension>` — change dimension for subsequent commands.
    pub fn in_(mut self, dimension: impl Into<String>) -> Self {
        self.parts.push(format!("in {}", dimension.into()));
        self
    }

    /// `align <axes>` — snap coordinates to the block grid along specified axes.
    pub fn align(mut self, axes: Swizzle) -> Self {
        self.parts.push(format!("align {axes}"));
        self
    }

    /// `positioned over <heightmap>` — snap y-coordinate to the top of the given heightmap (1.19.4+).
    pub fn positioned_over(mut self, heightmap: impl Into<String>) -> Self {
        self.parts
            .push(format!("positioned over {}", heightmap.into()));
        self
    }

    /// `anchored <anchor>` — change the anchor point for position calculations.
    pub fn anchored(mut self, anchor: Anchor) -> Self {
        self.parts.push(format!("anchored {anchor}"));
        self
    }

    /// `on <relation>` — follow an entity relationship chain.
    pub fn on(mut self, relation: impl Into<String>) -> Self {
        self.parts.push(format!("on {}", relation.into()));
        self
    }

    /// `summon <entity_type>` — summon an entity and execute as it immediately.
    pub fn summon(mut self, entity_type: impl Into<String>) -> Self {
        self.parts.push(format!("summon {}", entity_type.into()));
        self
    }

    // ── Condition sub-commands ────────────────────────────────────────────────

    /// `if entity <selector>` — execute only if the selector matches at least one entity.
    pub fn if_entity(mut self, selector: Selector) -> Self {
        self.parts.push(format!("if entity {selector}"));
        self
    }

    /// `unless entity <selector>` — execute only if the selector matches NO entities.
    pub fn unless_entity(mut self, selector: Selector) -> Self {
        self.parts.push(format!("unless entity {selector}"));
        self
    }

    /// `if entity @s[team=<team>]` — continue only if the current entity is on the given team.
    pub fn if_on_team(mut self, team: impl Into<String>) -> Self {
        self.parts
            .push(format!("if entity @s[team={}]", team.into()));
        self
    }

    /// `unless entity @s[team=<team>]` — skip if the current entity is on the given team.
    pub fn unless_on_team(mut self, team: impl Into<String>) -> Self {
        self.parts
            .push(format!("unless entity @s[team={}]", team.into()));
        self
    }

    /// `if score <a> <a_obj> = <b> <b_obj>` — continue only if the two scores are equal.
    pub fn if_score(
        mut self,
        a: Selector,
        a_obj: impl Into<String>,
        b: Selector,
        b_obj: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "if score {a} {} = {b} {}",
            a_obj.into(),
            b_obj.into()
        ));
        self
    }

    /// `unless score <a> <a_obj> = <b> <b_obj>` — skip if two scores are equal.
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

    /// `if block <pos> <block>` — execute only if the block at `pos` matches.
    pub fn if_block(mut self, pos: BlockPos, block: impl Into<String>) -> Self {
        self.parts.push(format!("if block {pos} {}", block.into()));
        self
    }

    /// `unless block <pos> <block>` — execute only if the block at `pos` does NOT match.
    pub fn unless_block(mut self, pos: BlockPos, block: impl Into<String>) -> Self {
        self.parts
            .push(format!("unless block {pos} {}", block.into()));
        self
    }

    /// `if score <holder> <obj> matches <range>` — execute if a score falls within the range.
    ///
    /// Range can be `"5"` (exact), `"5.."` (5 or more), `"..5"` (5 or less), or `"1..10"`.
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

    /// `unless score <holder> <obj> matches <range>` — execute if a score falls OUTSIDE the range.
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

    /// `if score <a> <a_obj> <cmp> <b> <b_obj>` — compare two scores.
    pub fn if_score_compare(
        mut self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        cmp: ScoreCmp,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "if score {} {} {cmp} {} {}",
            a.into(),
            a_obj.into(),
            b.into(),
            b_obj.into()
        ));
        self
    }

    /// `unless score <a> <a_obj> <cmp> <b> <b_obj>` — skip if the comparison is true.
    pub fn unless_score_compare(
        mut self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        cmp: ScoreCmp,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "unless score {} {} {cmp} {} {}",
            a.into(),
            a_obj.into(),
            b.into(),
            b_obj.into()
        ));
        self
    }

    // ── Score comparison shorthands ───────────────────────────────────────────

    /// `if score <a> <a_obj> = <b> <b_obj>` — continue if scores are equal.
    pub fn if_score_eq(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Eq, b, b_obj)
    }

    /// `unless score <a> <a_obj> = <b> <b_obj>` — skip if scores are equal.
    pub fn unless_score_eq(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Eq, b, b_obj)
    }

    /// `if score ... < ...` — continue if `a` is strictly less than `b`.
    pub fn if_score_lt(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Lt, b, b_obj)
    }

    /// `unless score ... < ...` — skip if `a` is strictly less than `b`.
    pub fn unless_score_lt(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Lt, b, b_obj)
    }

    /// `if score ... <= ...` — continue if `a` is less than or equal to `b`.
    pub fn if_score_lte(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Le, b, b_obj)
    }

    /// `unless score ... <= ...` — skip if `a` is less than or equal to `b`.
    pub fn unless_score_lte(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Le, b, b_obj)
    }

    /// `if score ... > ...` — continue if `a` is strictly greater than `b`.
    pub fn if_score_gt(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Gt, b, b_obj)
    }

    /// `unless score ... > ...` — skip if `a` is strictly greater than `b`.
    pub fn unless_score_gt(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Gt, b, b_obj)
    }

    /// `if score ... >= ...` — continue if `a` is greater than or equal to `b`.
    pub fn if_score_gte(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Ge, b, b_obj)
    }

    /// `unless score ... >= ...` — skip if `a` is greater than or equal to `b`.
    pub fn unless_score_gte(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Ge, b, b_obj)
    }

    // ── Data / NBT conditions ─────────────────────────────────────────────────

    /// `if data entity <selector> <path>` — continue if entity NBT has a value at `path`.
    pub fn if_data_entity(mut self, selector: Selector, path: impl Into<String>) -> Self {
        self.parts
            .push(format!("if data entity {selector} {}", path.into()));
        self
    }

    /// `unless data entity <selector> <path>` — skip if entity NBT has a value at `path`.
    pub fn unless_data_entity(mut self, selector: Selector, path: impl Into<String>) -> Self {
        self.parts
            .push(format!("unless data entity {selector} {}", path.into()));
        self
    }

    /// `if data block <pos> <path>` — continue if block NBT has a value at `path`.
    pub fn if_data_block(mut self, pos: BlockPos, path: impl Into<String>) -> Self {
        self.parts
            .push(format!("if data block {pos} {}", path.into()));
        self
    }

    /// `unless data block <pos> <path>` — skip if block NBT has a value at `path`.
    pub fn unless_data_block(mut self, pos: BlockPos, path: impl Into<String>) -> Self {
        self.parts
            .push(format!("unless data block {pos} {}", path.into()));
        self
    }

    /// `if data storage <source> <path>` — continue if storage has a value at `path`.
    pub fn if_data_storage(mut self, source: impl Into<String>, path: impl Into<String>) -> Self {
        self.parts
            .push(format!("if data storage {} {}", source.into(), path.into()));
        self
    }

    /// `unless data storage <source> <path>` — skip if storage has a value at `path`.
    pub fn unless_data_storage(
        mut self,
        source: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "unless data storage {} {}",
            source.into(),
            path.into()
        ));
        self
    }

    // ── World conditions ──────────────────────────────────────────────────────

    /// `if biome <pos> <biome>` — continue if the biome at `pos` matches (1.19.4+).
    pub fn if_biome(mut self, pos: BlockPos, biome: impl Into<String>) -> Self {
        self.parts.push(format!("if biome {pos} {}", biome.into()));
        self
    }

    /// `unless biome <pos> <biome>` — skip if the biome at `pos` matches.
    pub fn unless_biome(mut self, pos: BlockPos, biome: impl Into<String>) -> Self {
        self.parts
            .push(format!("unless biome {pos} {}", biome.into()));
        self
    }

    /// `if dimension <dimension>` — continue if executing in the given dimension (1.21+).
    pub fn if_dimension(mut self, dimension: impl Into<String>) -> Self {
        self.parts
            .push(format!("if dimension {}", dimension.into()));
        self
    }

    /// `unless dimension <dimension>` — skip if executing in the given dimension (1.21+).
    pub fn unless_dimension(mut self, dimension: impl Into<String>) -> Self {
        self.parts
            .push(format!("unless dimension {}", dimension.into()));
        self
    }

    /// `if loaded <pos>` — continue only if the chunk at `pos` is fully loaded.
    pub fn if_loaded(mut self, pos: BlockPos) -> Self {
        self.parts.push(format!("if loaded {pos}"));
        self
    }

    /// `unless loaded <pos>` — skip if the chunk at `pos` is NOT fully loaded.
    pub fn unless_loaded(mut self, pos: BlockPos) -> Self {
        self.parts.push(format!("unless loaded {pos}"));
        self
    }

    /// `if items entity <selector> <slot> <item>` — execute if an entity has a matching item.
    pub fn if_items_entity(
        mut self,
        selector: Selector,
        slot: ItemSlot,
        item: impl Into<String>,
    ) -> Self {
        self.parts
            .push(format!("if items entity {selector} {slot} {}", item.into()));
        self
    }

    /// `unless items entity <selector> <slot> <item>` — skip if the entity has the item.
    pub fn unless_items_entity(
        mut self,
        selector: Selector,
        slot: ItemSlot,
        item: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "unless items entity {selector} {slot} {}",
            item.into()
        ));
        self
    }

    /// `if items block <pos> <slot> <item>` — execute if a block container has a matching item.
    pub fn if_items_block(
        mut self,
        pos: BlockPos,
        slot: ItemSlot,
        item: impl Into<String>,
    ) -> Self {
        self.parts
            .push(format!("if items block {pos} {slot} {}", item.into()));
        self
    }

    /// `unless items block <pos> <slot> <item>` — skip if the block container has the item.
    pub fn unless_items_block(
        mut self,
        pos: BlockPos,
        slot: ItemSlot,
        item: impl Into<String>,
    ) -> Self {
        self.parts
            .push(format!("unless items block {pos} {slot} {}", item.into()));
        self
    }

    /// `if predicate <predicate>` — execute if a loot table predicate evaluates to true.
    pub fn if_predicate(mut self, predicate: impl Into<String>) -> Self {
        self.parts
            .push(format!("if predicate {}", predicate.into()));
        self
    }

    /// Append a raw condition fragment (e.g. from `Objective::if_matches`).
    pub fn if_(mut self, condition: impl Into<String>) -> Self {
        self.parts.push(condition.into());
        self
    }

    // ── Items conditions (1.20.5+) ────────────────────────────────────────────

    /// `if items entity <selector> <slot> <item>` — execute if the slot holds a matching item.
    pub fn if_items(
        mut self,
        selector: Selector,
        slot: InventorySlot,
        item: impl Into<String>,
    ) -> Self {
        self.parts
            .push(format!("if items entity {selector} {slot} {}", item.into()));
        self
    }

    /// `unless items entity <selector> <slot> <item>` — execute if the slot does NOT match.
    pub fn unless_items(
        mut self,
        selector: Selector,
        slot: InventorySlot,
        item: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "unless items entity {selector} {slot} {}",
            item.into()
        ));
        self
    }

    /// `if items entity <selector> <slot_pattern> <item>` — check multiple slots using a pattern.
    pub fn if_items_pattern(
        mut self,
        selector: Selector,
        pattern: SlotPattern,
        item: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "if items entity {selector} {pattern} {}",
            item.into()
        ));
        self
    }

    /// `unless items entity <selector> <slot_pattern> <item>` — check pattern does NOT match.
    pub fn unless_items_pattern(
        mut self,
        selector: Selector,
        pattern: SlotPattern,
        item: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "unless items entity {selector} {pattern} {}",
            item.into()
        ));
        self
    }

    // ── Store sub-commands ────────────────────────────────────────────────────

    /// `store result score <holder> <objective>` — capture the `run` result into a score.
    pub fn store_result_score(mut self, holder: ScoreHolder, objective: impl Into<String>) -> Self {
        self.parts
            .push(format!("store result score {holder} {}", objective.into()));
        self
    }

    /// `store success score <holder> <objective>` — store 1 if `run` succeeds, 0 if it fails.
    pub fn store_success_score(
        mut self,
        holder: ScoreHolder,
        objective: impl Into<String>,
    ) -> Self {
        self.parts
            .push(format!("store success score {holder} {}", objective.into()));
        self
    }

    /// `store result nbt <target> <path> <type> <scale>` — write the `run` result into NBT.
    pub fn store_result_nbt(
        mut self,
        target: DataTarget,
        path: impl Into<String>,
        kind: NbtStoreKind,
        scale: f64,
    ) -> Self {
        self.parts.push(format!(
            "store result {} {} {kind} {scale}",
            target,
            path.into()
        ));
        self
    }

    /// `store success nbt <target> <path> <type> <scale>` — write 1/0 (success/fail) into NBT.
    pub fn store_success_nbt(
        mut self,
        target: DataTarget,
        path: impl Into<String>,
        kind: NbtStoreKind,
        scale: f64,
    ) -> Self {
        self.parts.push(format!(
            "store success {} {} {kind} {scale}",
            target,
            path.into()
        ));
        self
    }

    /// `store result bossbar <id> value` — write the `run` result into a bossbar's current value.
    pub fn store_result_bossbar(
        mut self,
        id: impl Into<String>,
        attribute: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "store result bossbar {} {}",
            id.into(),
            attribute.into()
        ));
        self
    }

    /// `store success bossbar <id> <attribute>` — write success/failure into a bossbar attribute.
    pub fn store_success_bossbar(
        mut self,
        id: impl Into<String>,
        attribute: impl Into<String>,
    ) -> Self {
        self.parts.push(format!(
            "store success bossbar {} {}",
            id.into(),
            attribute.into()
        ));
        self
    }

    // ── Terminal ──────────────────────────────────────────────────────────────

    /// `run <command>` — finalize the execute chain and run the given command.
    ///
    /// Accepts any value implementing [`fmt::Display`] — including all [`Build`]
    /// types (which implement `Display`), generated `Command` types, raw `&str`,
    /// and owned `String`s.
    pub fn run(mut self, cmd: impl fmt::Display) -> String {
        self.parts.push(format!("run {cmd}"));
        format!("execute {}", self.parts.join(" "))
    }

    /// Like [`run`](Execute::run) but more explicit about accepting raw strings.
    pub fn run_raw(mut self, cmd: impl fmt::Display) -> String {
        self.parts.push(format!("run {cmd}"));
        format!("execute {}", self.parts.join(" "))
    }

    /// Run a named function: `execute ... run function <namespace:path>`.
    pub fn run_fn(mut self, function: impl fmt::Display) -> String {
        self.parts.push(format!("run function {function}"));
        format!("execute {}", self.parts.join(" "))
    }
}

impl Build for Execute {
    /// Return the current partial execute chain (without a `run` clause).
    ///
    /// Useful for embedding in `execute store` prefixes or debugging.
    fn build(&self) -> String {
        if self.parts.is_empty() {
            "execute".to_string()
        } else {
            format!("execute {}", self.parts.join(" "))
        }
    }
}

impl fmt::Display for Execute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build())
    }
}

impl From<Execute> for String {
    fn from(v: Execute) -> Self {
        v.build()
    }
}

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
    fn score_compare_ops() {
        let eq = Execute::new()
            .if_score_eq("@s", "mana", "@s", "max_mana")
            .run_raw("say full");
        assert_eq!(eq, "execute if score @s mana = @s max_mana run say full");

        let lt = Execute::new()
            .if_score_lt("@s", "health", "#const", "ten")
            .run_raw("say low");
        assert_eq!(lt, "execute if score @s health < #const ten run say low");

        let lte = Execute::new()
            .if_score_lte("@s", "health", "#const", "ten")
            .run_raw("say lte");
        assert_eq!(lte, "execute if score @s health <= #const ten run say lte");

        let gte = Execute::new()
            .if_score_gte("@s", "mana", "@s", "cost")
            .run_raw("say can cast");
        assert_eq!(gte, "execute if score @s mana >= @s cost run say can cast");

        let unless = Execute::new()
            .unless_score_gt("@s", "mana", "#const", "zero")
            .run_raw("say no mana");
        assert_eq!(
            unless,
            "execute unless score @s mana > #const zero run say no mana"
        );
    }

    #[test]
    fn if_data_conditions() {
        let s = Execute::new()
            .if_data_entity(Selector::self_(), "Custom.ready")
            .run_raw("say ready");
        assert_eq!(s, "execute if data entity @s Custom.ready run say ready");

        let s = Execute::new()
            .if_data_storage("my_pack:state", "phase")
            .run_raw("say has phase");
        assert_eq!(
            s,
            "execute if data storage my_pack:state phase run say has phase"
        );
    }

    #[test]
    fn world_conditions() {
        let s = Execute::new()
            .if_biome(BlockPos::here(), "minecraft:desert")
            .run_raw("say desert");
        assert_eq!(s, "execute if biome ~ ~ ~ minecraft:desert run say desert");

        let s = Execute::new()
            .if_loaded(BlockPos::here())
            .run_raw("say loaded");
        assert_eq!(s, "execute if loaded ~ ~ ~ run say loaded");

        let s = Execute::new()
            .if_dimension("minecraft:the_nether")
            .run_raw("say nether");
        assert_eq!(
            s,
            "execute if dimension minecraft:the_nether run say nether"
        );
    }

    #[test]
    fn positioned_over_test() {
        let s = Execute::new()
            .as_(Selector::all_players())
            .positioned_as(Selector::self_())
            .positioned_over("world_surface")
            .run_raw("say ground");
        assert_eq!(
            s,
            "execute as @a positioned as @s positioned over world_surface run say ground"
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
