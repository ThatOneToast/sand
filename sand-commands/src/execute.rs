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
use crate::RawCommand;
use crate::coord::{BlockPos, Rotation, Vec3};
use crate::error::{CommandError, CommandResult};
use crate::execute_args::{Anchor, ItemSlot, NbtStoreKind, Swizzle};
use crate::nbt::DataTarget;
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::scoreboard::{ScoreCmp, ScoreHolder};
use crate::selector::Selector;
use crate::validate;

/// Builder for the `execute` command chain.
///
/// Call builder methods to add sub-commands, then call [`run`](Execute::run) or
/// [`run_raw`](Execute::run_raw) to complete the command.
#[derive(Debug, Clone, Default)]
#[must_use = "execute builders must be completed with `run`, `try_run`, or `run_raw`"]
pub struct Execute {
    parts: Vec<String>,
    checks: Vec<ExecuteCheck>,
}

#[derive(Debug, Clone)]
enum ExecuteCheck {
    Selector {
        index: usize,
        kind: &'static str,
        value: Selector,
    },
    Vec3 {
        index: usize,
        kind: &'static str,
        value: Vec3,
    },
    BlockPos {
        index: usize,
        kind: &'static str,
        value: BlockPos,
    },
    Rotation {
        index: usize,
        kind: &'static str,
        value: Rotation,
    },
    Slot {
        index: usize,
        kind: &'static str,
        value: ItemSlot,
    },
    Finite {
        index: usize,
        kind: &'static str,
        field: &'static str,
        value: f64,
    },
    Resource {
        index: usize,
        kind: &'static str,
        field: &'static str,
        value: String,
        allow_tag: bool,
    },
    Holder {
        index: usize,
        kind: &'static str,
        value: ScoreHolder,
    },
    SingleHolder {
        index: usize,
        kind: &'static str,
        value: ScoreHolder,
    },
    Objective {
        index: usize,
        kind: &'static str,
        value: String,
    },
    ScoreRange {
        index: usize,
        kind: &'static str,
        value: String,
    },
}

impl Execute {
    /// Create a new `Execute` builder with no sub-commands.
    pub fn new() -> Self {
        Self {
            parts: vec![],
            checks: vec![],
        }
    }

    fn next_index(&self) -> usize {
        self.parts.len()
    }

    fn check_selector(&mut self, kind: &'static str, value: &Selector) {
        self.checks.push(ExecuteCheck::Selector {
            index: self.next_index(),
            kind,
            value: value.clone(),
        });
    }

    fn check_vec3(&mut self, kind: &'static str, value: &Vec3) {
        self.checks.push(ExecuteCheck::Vec3 {
            index: self.next_index(),
            kind,
            value: value.clone(),
        });
    }

    fn check_block_pos(&mut self, kind: &'static str, value: &BlockPos) {
        self.checks.push(ExecuteCheck::BlockPos {
            index: self.next_index(),
            kind,
            value: value.clone(),
        });
    }

    fn check_objective(&mut self, kind: &'static str, value: &str) {
        self.checks.push(ExecuteCheck::Objective {
            index: self.next_index(),
            kind,
            value: value.to_string(),
        });
    }

    fn check_resource(
        &mut self,
        kind: &'static str,
        field: &'static str,
        value: &str,
        allow_tag: bool,
    ) {
        self.checks.push(ExecuteCheck::Resource {
            index: self.next_index(),
            kind,
            field,
            value: value.to_string(),
            allow_tag,
        });
    }

    fn check_single_holder(&mut self, kind: &'static str, value: impl Into<String>) -> String {
        let value = value.into();
        self.checks.push(ExecuteCheck::SingleHolder {
            index: self.next_index(),
            kind,
            value: ScoreHolder::from_compat(value.clone()),
        });
        value
    }

    // ── Context sub-commands ──────────────────────────────────────────────────

    /// `as <selector>` — change the executing entity.
    pub fn as_(mut self, selector: Selector) -> Self {
        self.check_selector("as", &selector);
        self.parts.push(format!("as {selector}"));
        self
    }

    /// `at <selector>` — change position and rotation to match the selected entity.
    pub fn at(mut self, selector: Selector) -> Self {
        self.check_selector("at", &selector);
        self.parts.push(format!("at {selector}"));
        self
    }

    /// `positioned <pos>` — change execution position to the given coordinates.
    pub fn positioned(mut self, pos: Vec3) -> Self {
        self.check_vec3("positioned", &pos);
        self.parts.push(format!("positioned {pos}"));
        self
    }

    /// `positioned as <selector>` — change position to match the selected entity.
    pub fn positioned_as(mut self, selector: Selector) -> Self {
        self.check_selector("positioned_as", &selector);
        self.parts.push(format!("positioned as {selector}"));
        self
    }

    /// `rotated <rotation>` — change execution rotation.
    pub fn rotated(mut self, rotation: Rotation) -> Self {
        self.checks.push(ExecuteCheck::Rotation {
            index: self.next_index(),
            kind: "rotated",
            value: rotation.clone(),
        });
        self.parts.push(format!("rotated {rotation}"));
        self
    }

    /// `rotated as <selector>` — change rotation to match the selected entity.
    pub fn rotated_as(mut self, selector: Selector) -> Self {
        self.check_selector("rotated_as", &selector);
        self.parts.push(format!("rotated as {selector}"));
        self
    }

    /// `facing <pos>` — rotate execution to face a position in the world.
    pub fn facing(mut self, pos: Vec3) -> Self {
        self.check_vec3("facing", &pos);
        self.parts.push(format!("facing {pos}"));
        self
    }

    /// `facing entity <selector> <anchor>` — rotate execution to face an entity's anchor point.
    pub fn facing_entity(mut self, selector: Selector, anchor: Anchor) -> Self {
        self.check_selector("facing_entity", &selector);
        self.parts
            .push(format!("facing entity {selector} {anchor}"));
        self
    }

    /// `in <dimension>` — change dimension for subsequent commands.
    pub fn in_(mut self, dimension: impl Into<String>) -> Self {
        let dimension = dimension.into();
        self.check_resource("in", "dimension", &dimension, false);
        self.parts.push(format!("in {dimension}"));
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
    pub fn summon(mut self, entity_type: impl crate::selector::IntoEntityType) -> Self {
        let entity_type = entity_type.into_entity_type();
        self.check_resource("summon", "entity_type", &entity_type, false);
        self.parts.push(format!("summon {entity_type}"));
        self
    }

    // ── Condition sub-commands ────────────────────────────────────────────────

    /// `if entity <selector>` — execute only if the selector matches at least one entity.
    pub fn if_entity(mut self, selector: Selector) -> Self {
        self.check_selector("if_entity", &selector);
        self.parts.push(format!("if entity {selector}"));
        self
    }

    /// `unless entity <selector>` — execute only if the selector matches NO entities.
    pub fn unless_entity(mut self, selector: Selector) -> Self {
        self.check_selector("unless_entity", &selector);
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
        self.check_selector("if_score.left", &a);
        self.check_selector("if_score.right", &b);
        self.checks.push(ExecuteCheck::SingleHolder {
            index: self.next_index(),
            kind: "if_score.left",
            value: ScoreHolder::entity(a.clone()),
        });
        self.checks.push(ExecuteCheck::SingleHolder {
            index: self.next_index(),
            kind: "if_score.right",
            value: ScoreHolder::entity(b.clone()),
        });
        let a_obj = a_obj.into();
        let b_obj = b_obj.into();
        self.check_objective("if_score.left", &a_obj);
        self.check_objective("if_score.right", &b_obj);
        self.parts
            .push(format!("if score {a} {a_obj} = {b} {b_obj}",));
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
        self.check_selector("unless_score.left", &primary_selector);
        self.check_selector("unless_score.right", &secondary_selector);
        self.checks.push(ExecuteCheck::SingleHolder {
            index: self.next_index(),
            kind: "unless_score.left",
            value: ScoreHolder::entity(primary_selector.clone()),
        });
        self.checks.push(ExecuteCheck::SingleHolder {
            index: self.next_index(),
            kind: "unless_score.right",
            value: ScoreHolder::entity(secondary_selector.clone()),
        });
        let primary = primary.into();
        let secondary = secondary.into();
        self.check_objective("unless_score.left", &primary);
        self.check_objective("unless_score.right", &secondary);
        self.parts.push(format!(
            "unless score {primary_selector} {primary} = {secondary_selector} {secondary}"
        ));
        self
    }

    /// `if block <pos> <block>` — execute only if the block at `pos` matches.
    pub fn if_block(mut self, pos: BlockPos, block: impl Into<String>) -> Self {
        self.check_block_pos("if_block", &pos);
        self.parts.push(format!("if block {pos} {}", block.into()));
        self
    }

    /// `unless block <pos> <block>` — execute only if the block at `pos` does NOT match.
    pub fn unless_block(mut self, pos: BlockPos, block: impl Into<String>) -> Self {
        self.check_block_pos("unless_block", &pos);
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
        let holder = self.check_single_holder("if_score_matches", holder);
        let objective = objective.into();
        let range = range.into();
        self.check_objective("if_score_matches", &objective);
        self.checks.push(ExecuteCheck::ScoreRange {
            index: self.next_index(),
            kind: "if_score_matches",
            value: range.clone(),
        });
        self.parts
            .push(format!("if score {holder} {objective} matches {range}",));
        self
    }

    /// `unless score <holder> <obj> matches <range>` — execute if a score falls OUTSIDE the range.
    pub fn unless_score_matches(
        mut self,
        holder: impl Into<String>,
        objective: impl Into<String>,
        range: impl Into<String>,
    ) -> Self {
        let holder = self.check_single_holder("unless_score_matches", holder);
        let objective = objective.into();
        let range = range.into();
        self.check_objective("unless_score_matches", &objective);
        self.checks.push(ExecuteCheck::ScoreRange {
            index: self.next_index(),
            kind: "unless_score_matches",
            value: range.clone(),
        });
        self.parts
            .push(format!("unless score {holder} {objective} matches {range}",));
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
        let a = self.check_single_holder("if_score_compare.left", a);
        let b = self.check_single_holder("if_score_compare.right", b);
        let a_obj = a_obj.into();
        let b_obj = b_obj.into();
        self.check_objective("if_score_compare.left", &a_obj);
        self.check_objective("if_score_compare.right", &b_obj);
        self.parts
            .push(format!("if score {a} {a_obj} {cmp} {b} {b_obj}",));
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
        let a = self.check_single_holder("unless_score_compare.left", a);
        let b = self.check_single_holder("unless_score_compare.right", b);
        let a_obj = a_obj.into();
        let b_obj = b_obj.into();
        self.check_objective("unless_score_compare.left", &a_obj);
        self.check_objective("unless_score_compare.right", &b_obj);
        self.parts
            .push(format!("unless score {a} {a_obj} {cmp} {b} {b_obj}",));
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
        self.check_selector("if_data_entity", &selector);
        self.parts
            .push(format!("if data entity {selector} {}", path.into()));
        self
    }

    /// `unless data entity <selector> <path>` — skip if entity NBT has a value at `path`.
    pub fn unless_data_entity(mut self, selector: Selector, path: impl Into<String>) -> Self {
        self.check_selector("unless_data_entity", &selector);
        self.parts
            .push(format!("unless data entity {selector} {}", path.into()));
        self
    }

    /// `if data block <pos> <path>` — continue if block NBT has a value at `path`.
    pub fn if_data_block(mut self, pos: BlockPos, path: impl Into<String>) -> Self {
        self.check_block_pos("if_data_block", &pos);
        self.parts
            .push(format!("if data block {pos} {}", path.into()));
        self
    }

    /// `unless data block <pos> <path>` — skip if block NBT has a value at `path`.
    pub fn unless_data_block(mut self, pos: BlockPos, path: impl Into<String>) -> Self {
        self.check_block_pos("unless_data_block", &pos);
        self.parts
            .push(format!("unless data block {pos} {}", path.into()));
        self
    }

    /// `if data storage <source> <path>` — continue if storage has a value at `path`.
    pub fn if_data_storage(mut self, source: impl Into<String>, path: impl Into<String>) -> Self {
        let source = source.into();
        self.check_resource("if_data_storage", "storage", &source, false);
        self.parts
            .push(format!("if data storage {source} {}", path.into()));
        self
    }

    /// `unless data storage <source> <path>` — skip if storage has a value at `path`.
    pub fn unless_data_storage(
        mut self,
        source: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        let source = source.into();
        self.check_resource("unless_data_storage", "storage", &source, false);
        self.parts
            .push(format!("unless data storage {source} {}", path.into(),));
        self
    }

    // ── World conditions ──────────────────────────────────────────────────────

    /// `if biome <pos> <biome>` — continue if the biome at `pos` matches (1.19.4+).
    pub fn if_biome(mut self, pos: BlockPos, biome: impl Into<String>) -> Self {
        let biome = biome.into();
        self.check_block_pos("if_biome", &pos);
        self.check_resource("if_biome", "biome", &biome, true);
        self.parts.push(format!("if biome {pos} {biome}"));
        self
    }

    /// `unless biome <pos> <biome>` — skip if the biome at `pos` matches.
    pub fn unless_biome(mut self, pos: BlockPos, biome: impl Into<String>) -> Self {
        let biome = biome.into();
        self.check_block_pos("unless_biome", &pos);
        self.check_resource("unless_biome", "biome", &biome, true);
        self.parts.push(format!("unless biome {pos} {biome}"));
        self
    }

    /// `if dimension <dimension>` — continue if executing in the given dimension (1.21+).
    pub fn if_dimension(mut self, dimension: impl Into<String>) -> Self {
        let dimension = dimension.into();
        self.check_resource("if_dimension", "dimension", &dimension, false);
        self.parts.push(format!("if dimension {dimension}"));
        self
    }

    /// `unless dimension <dimension>` — skip if executing in the given dimension (1.21+).
    pub fn unless_dimension(mut self, dimension: impl Into<String>) -> Self {
        let dimension = dimension.into();
        self.check_resource("unless_dimension", "dimension", &dimension, false);
        self.parts.push(format!("unless dimension {dimension}"));
        self
    }

    /// `if loaded <pos>` — continue only if the chunk at `pos` is fully loaded.
    pub fn if_loaded(mut self, pos: BlockPos) -> Self {
        self.check_block_pos("if_loaded", &pos);
        self.parts.push(format!("if loaded {pos}"));
        self
    }

    /// `unless loaded <pos>` — skip if the chunk at `pos` is NOT fully loaded.
    pub fn unless_loaded(mut self, pos: BlockPos) -> Self {
        self.check_block_pos("unless_loaded", &pos);
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
        self.check_selector("if_items_entity", &selector);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "if_items_entity",
            value: slot.clone(),
        });
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
        self.check_selector("unless_items_entity", &selector);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "unless_items_entity",
            value: slot.clone(),
        });
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
        self.check_block_pos("if_items_block", &pos);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "if_items_block",
            value: slot.clone(),
        });
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
        self.check_block_pos("unless_items_block", &pos);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "unless_items_block",
            value: slot.clone(),
        });
        self.parts
            .push(format!("unless items block {pos} {slot} {}", item.into()));
        self
    }

    /// `if predicate <predicate>` — execute if a loot table predicate evaluates to true.
    pub fn if_predicate(mut self, predicate: impl Into<String>) -> Self {
        let predicate = predicate.into();
        self.check_resource("if_predicate", "predicate", &predicate, false);
        self.parts.push(format!("if predicate {predicate}"));
        self
    }

    /// Append a raw condition fragment (e.g. from `Objective::if_matches`).
    pub fn if_(mut self, condition: impl Into<String>) -> Self {
        self.parts.push(condition.into());
        self
    }

    // ── Items conditions (1.20.5+) ────────────────────────────────────────────

    /// `if items entity <selector> <slot> <item>` — execute if the slot holds a matching item.
    ///
    /// Accepts any type that converts to [`ItemSlot`], including wildcard
    /// variants such as `ItemSlot::AnyHotbar`.
    pub fn if_items(
        mut self,
        selector: Selector,
        slot: impl Into<ItemSlot>,
        item: impl Into<String>,
    ) -> Self {
        let slot = slot.into();
        self.check_selector("if_items", &selector);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "if_items",
            value: slot.clone(),
        });
        self.parts
            .push(format!("if items entity {selector} {slot} {}", item.into()));
        self
    }

    /// `unless items entity <selector> <slot> <item>` — execute if the slot does NOT match.
    pub fn unless_items(
        mut self,
        selector: Selector,
        slot: impl Into<ItemSlot>,
        item: impl Into<String>,
    ) -> Self {
        let slot = slot.into();
        self.check_selector("unless_items", &selector);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "unless_items",
            value: slot.clone(),
        });
        self.parts.push(format!(
            "unless items entity {selector} {slot} {}",
            item.into()
        ));
        self
    }

    // ── Store sub-commands ────────────────────────────────────────────────────

    /// `store result score <holder> <objective>` — capture the `run` result into a score.
    pub fn store_result_score(mut self, holder: ScoreHolder, objective: impl Into<String>) -> Self {
        let objective = objective.into();
        self.checks.push(ExecuteCheck::Holder {
            index: self.next_index(),
            kind: "store_result_score",
            value: holder.clone(),
        });
        self.checks.push(ExecuteCheck::Objective {
            index: self.next_index(),
            kind: "store_result_score",
            value: objective.clone(),
        });
        self.parts
            .push(format!("store result score {holder} {objective}"));
        self
    }

    /// `store success score <holder> <objective>` — store 1 if `run` succeeds, 0 if it fails.
    pub fn store_success_score(
        mut self,
        holder: ScoreHolder,
        objective: impl Into<String>,
    ) -> Self {
        let objective = objective.into();
        self.checks.push(ExecuteCheck::Holder {
            index: self.next_index(),
            kind: "store_success_score",
            value: holder.clone(),
        });
        self.checks.push(ExecuteCheck::Objective {
            index: self.next_index(),
            kind: "store_success_score",
            value: objective.clone(),
        });
        self.parts
            .push(format!("store success score {holder} {objective}"));
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
        self.checks.push(ExecuteCheck::Finite {
            index: self.next_index(),
            kind: "store_result_nbt",
            field: "scale",
            value: scale,
        });
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
        self.checks.push(ExecuteCheck::Finite {
            index: self.next_index(),
            kind: "store_success_nbt",
            field: "scale",
            value: scale,
        });
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

    /// Compatibility renderer for `run <command>`.
    ///
    /// This retains the historical infallible string API. Prefer [`try_run`](Self::try_run)
    /// for typed terminal commands; exported compatibility output is validated
    /// again with function context before files are accepted.
    pub fn run(mut self, cmd: impl fmt::Display) -> String {
        self.parts.push(format!("run {cmd}"));
        format!("execute {}", self.parts.join(" "))
    }

    /// Validate the whole execute chain and a typed terminal command before
    /// rendering. Errors identify the failing execute subcommand.
    pub fn try_run(self, cmd: &impl RenderCommand) -> CommandResult<String> {
        let profile = CommandProfile::unprofiled();
        self.validate(&profile)?;
        let cmd = cmd
            .render(&profile)
            .map_err(|e| e.with_context("Execute::run command"))?;
        Ok(format!("{} run {cmd}", self.build()))
    }

    /// Like [`run`](Execute::run) but more explicit about accepting raw strings.
    pub fn run_raw(mut self, cmd: impl fmt::Display) -> String {
        self.parts.push(format!("run {cmd}"));
        format!("execute {}", self.parts.join(" "))
    }

    /// Validate the typed execute chain, then append an explicitly raw terminal
    /// command. The raw text bypasses typed grammar modeling but must remain one
    /// `.mcfunction`-safe line without a leading slash.
    pub fn try_run_raw(self, cmd: RawCommand) -> CommandResult<String> {
        let profile = CommandProfile::unprofiled();
        self.validate(&profile)?;
        let cmd = cmd.as_str();
        if cmd.contains(['\0', '\n', '\r']) || cmd.trim_start().starts_with('/') {
            return Err(CommandError::new(
                "Execute::try_run_raw",
                "command",
                "raw terminal commands must be a single line without a leading `/`",
            ));
        }
        Ok(format!("{} run {cmd}", self.build()))
    }

    /// Run a named function: `execute ... run function <namespace:path>`.
    pub fn run_fn(mut self, function: impl fmt::Display) -> String {
        self.parts.push(format!("run function {function}"));
        format!("execute {}", self.parts.join(" "))
    }
}

impl Validate for Execute {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        if self.parts.is_empty() {
            return Err(CommandError::new(
                "Execute",
                "subcommands",
                "execute chains require at least one subcommand",
            ));
        }
        for check in &self.checks {
            let (index, kind, result) = match check {
                ExecuteCheck::Selector { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::Vec3 { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::BlockPos { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::Rotation { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::Slot { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::Finite {
                    index,
                    kind,
                    field,
                    value,
                } => (
                    *index,
                    *kind,
                    validate::finite(*value, "Execute", field).map(|_| ()),
                ),
                ExecuteCheck::Resource {
                    index,
                    kind,
                    field,
                    value,
                    allow_tag,
                } => (
                    *index,
                    *kind,
                    validate::resource_location_shape(
                        if *allow_tag {
                            value.strip_prefix('#').unwrap_or(value)
                        } else {
                            value
                        },
                        "Execute",
                        field,
                    )
                    .map(|_| ()),
                ),
                ExecuteCheck::Holder { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::SingleHolder { index, kind, value } => {
                    (*index, *kind, value.validate_single(profile))
                }
                ExecuteCheck::Objective { index, kind, value } => {
                    let result = validate::no_whitespace_or_control(value, "Execute", "objective")
                        .and_then(|_| {
                            if value.len() <= 16 {
                                Ok(value.as_str())
                            } else {
                                Err(CommandError::new(
                                    "Execute",
                                    "objective",
                                    "objective names cannot exceed 16 characters",
                                ))
                            }
                        })
                        .map(|_| ());
                    (*index, *kind, result)
                }
                ExecuteCheck::ScoreRange { index, kind, value } => {
                    (*index, *kind, validate_score_range(value))
                }
            };
            result.map_err(|e| e.with_context(format!("Execute subcommand {index} `{kind}`")))?;
        }
        Ok(())
    }
}

fn validate_score_range(value: &str) -> CommandResult<()> {
    validate::non_empty(value, "Execute", "score_range")?;
    let parse = |bound: &str| -> CommandResult<Option<i32>> {
        if bound.is_empty() {
            Ok(None)
        } else {
            bound.parse::<i32>().map(Some).map_err(|_| {
                CommandError::new(
                    "Execute",
                    "score_range",
                    format!("invalid integer bound `{bound}`"),
                )
            })
        }
    };
    let (min, max) = if let Some((min, max)) = value.split_once("..") {
        if max.contains("..") {
            return Err(CommandError::new(
                "Execute",
                "score_range",
                "range contains more than one `..`",
            ));
        }
        (parse(min)?, parse(max)?)
    } else {
        let exact = parse(value)?;
        (exact, exact)
    };
    if min.is_none() && max.is_none() {
        return Err(CommandError::new(
            "Execute",
            "score_range",
            "range must contain at least one bound",
        ));
    }
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        return Err(CommandError::new(
            "Execute",
            "score_range",
            format!("range lower bound `{min}` exceeds upper bound `{max}`"),
        ));
    }
    Ok(())
}

impl RenderCommand for Execute {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.build()
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

        assert!(
            Execute::new()
                .if_dimension("the_nether")
                .try_run_raw(RawCommand::new("say no"))
                .is_err()
        );
        assert!(
            Execute::new()
                .if_biome(BlockPos::here(), "#minecraft:is_overworld")
                .try_run_raw(RawCommand::new("say yes"))
                .is_ok()
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

    // ── Additional execute golden tests ───────────────────────────────────────

    #[test]
    fn anchored_eyes() {
        let s = Execute::new().anchored(Anchor::Eyes).run_raw("say looking");
        assert_eq!(s, "execute anchored eyes run say looking");
    }

    #[test]
    fn anchored_feet() {
        let s = Execute::new().anchored(Anchor::Feet).run_raw("say feet");
        assert_eq!(s, "execute anchored feet run say feet");
    }

    #[test]
    fn in_dimension() {
        let s = Execute::new()
            .in_("minecraft:the_nether")
            .run_raw("say nether");
        assert_eq!(s, "execute in minecraft:the_nether run say nether");
    }

    #[test]
    fn rotated_as() {
        let s = Execute::new()
            .rotated_as(Selector::self_())
            .run_raw("tp @s ~ ~ ~");
        assert_eq!(s, "execute rotated as @s run tp @s ~ ~ ~");
    }

    #[test]
    fn facing_entity() {
        let s = Execute::new()
            .facing_entity(Selector::nearest_player(), Anchor::Eyes)
            .run_raw("say facing");
        assert_eq!(s, "execute facing entity @p eyes run say facing");
    }

    #[test]
    fn if_predicate_chain() {
        let s = Execute::new()
            .as_(Selector::all_players())
            .if_predicate("my_pack:is_sneaking")
            .run_raw("say sneaking");
        assert_eq!(
            s,
            "execute as @a if predicate my_pack:is_sneaking run say sneaking"
        );
    }

    #[test]
    fn store_result_nbt_entity() {
        let s = Execute::new()
            .store_result_nbt(
                crate::nbt::DataTarget::Entity(Selector::self_()),
                "Custom.kills",
                NbtStoreKind::Int,
                1.0,
            )
            .run_raw("scoreboard players get @s kills");
        assert_eq!(
            s,
            "execute store result entity @s Custom.kills int 1 run scoreboard players get @s kills"
        );
    }

    #[test]
    fn store_success_score() {
        let s = Execute::new()
            .store_success_score(ScoreHolder::entity(Selector::self_()), "result_obj")
            .if_entity(Selector::all_entities().entity_type("minecraft:zombie"))
            .run_raw("say zombies");
        assert_eq!(
            s,
            "execute store success score @s result_obj if entity @e[type=minecraft:zombie] run say zombies"
        );
    }

    #[test]
    fn run_fn_formats_correctly() {
        let s = Execute::new()
            .as_(Selector::all_players())
            .run_fn("my_pack:on_tick");
        assert_eq!(s, "execute as @a run function my_pack:on_tick");
    }

    #[test]
    fn summon_subcommand() {
        let s = Execute::new()
            .summon("minecraft:armor_stand")
            .run_raw("say spawned");
        assert_eq!(s, "execute summon minecraft:armor_stand run say spawned");
    }

    #[test]
    fn unless_entity() {
        let s = Execute::new()
            .unless_entity(Selector::all_players().tag("ready"))
            .run_raw("say not ready");
        assert_eq!(s, "execute unless entity @a[tag=ready] run say not ready");
    }

    #[test]
    fn unless_block_condition() {
        let s = Execute::new()
            .unless_block(BlockPos::here(), "minecraft:air")
            .run_raw("say blocked");
        assert_eq!(
            s,
            "execute unless block ~ ~ ~ minecraft:air run say blocked"
        );
    }

    #[test]
    fn try_build_reports_execute_subcommand_context() {
        let execute = Execute::new().positioned(Vec3::absolute(f64::NAN, 0.0, 0.0));
        let error = execute.try_build().unwrap_err().to_string();
        assert!(
            error.contains("Execute subcommand 0 `positioned`"),
            "{error}"
        );
        assert!(error.contains("finite"), "{error}");
    }

    #[test]
    fn try_build_rejects_invalid_slot_and_scale() {
        assert!(
            Execute::new()
                .if_items(Selector::self_(), ItemSlot::Hotbar(9), "minecraft:stone")
                .try_build()
                .is_err()
        );
        assert!(
            Execute::new()
                .store_result_nbt(
                    DataTarget::Entity(Selector::self_()),
                    "x",
                    NbtStoreKind::Double,
                    f64::INFINITY
                )
                .try_build()
                .is_err()
        );
    }

    #[test]
    fn try_build_validates_score_holders_objectives_and_ranges() {
        let profile = CommandProfile::unprofiled();
        let many = Execute::new().if_score_matches("@a", "mana", "1..");
        let error = many.render(&profile).unwrap_err().to_string();
        assert!(error.contains("if_score_matches"), "{error}");
        assert!(error.contains("exactly one holder"), "{error}");

        assert!(
            Execute::new()
                .if_score_matches("@s", "objective_is_too_long", "1..")
                .render(&profile)
                .is_err()
        );
        assert!(
            Execute::new()
                .if_score_matches("@s", "mana", "5..1")
                .render(&profile)
                .is_err()
        );
    }

    #[test]
    fn try_run_raw_preserves_advanced_syntax_but_validates_the_chain() {
        let command = Execute::new()
            .as_(Selector::all_players())
            .try_run_raw(RawCommand::new("modded command syntax"))
            .unwrap();
        assert_eq!(command, "execute as @a run modded command syntax");
        assert!(
            Execute::new()
                .as_(Selector::all_players().limit(0))
                .try_run_raw(RawCommand::new("modded command syntax"))
                .is_err()
        );
        assert!(
            Execute::new()
                .as_(Selector::all_players())
                .try_run_raw(RawCommand::new("/say no"))
                .is_err()
        );
    }
}
