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

use super::{Anchor, BlockPos, Command, ItemSlot, NbtStoreKind, Rotation, ScoreCmp, ScoreHolder, Selector, Swizzle, Vec3};
use super::data::DataTarget;

/// Builder for the `execute` command chain.
///
/// Call builder methods to add sub-commands, then call [`run`](Execute::run)
/// to complete the command.
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

    /// `as <selector>` — change the executing entity to match the selector.
    ///
    /// After this, all subsequent context (position, rotation) and `@s` refer to the selected entity.
    /// Produces: `execute as <selector> ...`
    pub fn as_(mut self, selector: Selector) -> Self {
        self.parts.push(format!("as {selector}"));
        self
    }

    /// `at <selector>` — change position and rotation to match the selected entity.
    ///
    /// Subsequent `@s` refers to the entity selected here. Useful for executing at a player's position.
    /// Produces: `execute at <selector> ...`
    pub fn at(mut self, selector: Selector) -> Self {
        self.parts.push(format!("at {selector}"));
        self
    }

    /// `positioned <pos>` — change execution position to the given absolute coordinates.
    ///
    /// Relative coordinates (`~x ~y ~z`) are supported. Rotation remains unchanged.
    /// Produces: `execute positioned <pos> ...`
    pub fn positioned(mut self, pos: Vec3) -> Self {
        self.parts.push(format!("positioned {pos}"));
        self
    }

    /// `positioned as <selector>` — change position to match the selected entity's position.
    ///
    /// Like `at` but only changes position, not rotation.
    /// Produces: `execute positioned as <selector> ...`
    pub fn positioned_as(mut self, selector: Selector) -> Self {
        self.parts.push(format!("positioned as {selector}"));
        self
    }

    /// `rotated <yaw> <pitch>` — change execution rotation to the given angles.
    ///
    /// Yaw (0-360 degrees) and pitch (-90 to 90 degrees). Position remains unchanged.
    /// Produces: `execute rotated <rotation> ...`
    pub fn rotated(mut self, rotation: Rotation) -> Self {
        self.parts.push(format!("rotated {rotation}"));
        self
    }

    /// `rotated as <selector>` — change rotation to match the selected entity's rotation.
    ///
    /// Like `at` but only changes rotation, not position.
    /// Produces: `execute rotated as <selector> ...`
    pub fn rotated_as(mut self, selector: Selector) -> Self {
        self.parts.push(format!("rotated as {selector}"));
        self
    }

    /// `facing <pos>` — rotate execution to face a position in the world.
    ///
    /// The executor's position is used as the origin. Useful for making entities look at a location.
    /// Produces: `execute facing <pos> ...`
    pub fn facing(mut self, pos: Vec3) -> Self {
        self.parts.push(format!("facing {pos}"));
        self
    }

    /// `facing entity <selector> <anchor>` — rotate execution to face an entity's anchor point.
    ///
    /// Anchor can be `eyes` or `feet`. Rotates from the executor toward the target.
    /// Produces: `execute facing entity <selector> <anchor> ...`
    pub fn facing_entity(mut self, selector: Selector, anchor: Anchor) -> Self {
        self.parts
            .push(format!("facing entity {selector} {anchor}"));
        self
    }

    /// `in <dimension>` — change dimension for subsequent commands.
    ///
    /// Changes which dimension the command executes in (e.g., `"minecraft:the_nether"`).
    /// Produces: `execute in <dimension> ...`
    pub fn in_(mut self, dimension: impl Into<String>) -> Self {
        self.parts.push(format!("in {}", dimension.into()));
        self
    }

    /// `align <axes>` — snap coordinates to the block grid along specified axes.
    ///
    /// Rounds coordinates down to block boundaries. E.g., `align(Swizzle::xy())` snaps x and y.
    /// Produces: `execute align <axes> ...`
    pub fn align(mut self, axes: Swizzle) -> Self {
        self.parts.push(format!("align {axes}"));
        self
    }

    /// `positioned over <heightmap>` — snap y-coordinate to the top of the given heightmap (1.19.4+).
    ///
    /// Common heightmaps: `"world_surface"`, `"motion_blocking"`, `"ocean_floor"`.
    /// Useful for placing things at ground level regardless of terrain.
    ///
    /// ```rust,ignore
    /// Execute::new()
    ///     .as_(Selector::all_players())
    ///     .positioned_as(Selector::self_())
    ///     .positioned_over("world_surface")
    ///     .run_raw("summon lightning_bolt ~ ~ ~");
    /// ```
    pub fn positioned_over(mut self, heightmap: impl Into<String>) -> Self {
        self.parts.push(format!("positioned over {}", heightmap.into()));
        self
    }

    /// `anchored <anchor>` — change the anchor point for position calculations.
    ///
    /// `eyes` for head level, `feet` for foot level. Affects position calculations in subsequent commands.
    /// Produces: `execute anchored <anchor> ...`
    pub fn anchored(mut self, anchor: Anchor) -> Self {
        self.parts.push(format!("anchored {anchor}"));
        self
    }

    /// `on <relation>` — follow an entity relationship chain.
    ///
    /// Switches execution to a related entity (e.g., `"attacker"`, `"vehicle"`, `"passenger"`).
    /// Produces: `execute on <relation> ...`
    pub fn on(mut self, relation: impl Into<String>) -> Self {
        self.parts.push(format!("on {}", relation.into()));
        self
    }

    /// `summon <entity_type>` — summon an entity and execute as it immediately.
    ///
    /// The summoned entity becomes the executor for subsequent commands.
    /// Produces: `execute summon <entity_type> ...`
    pub fn summon(mut self, entity_type: impl Into<String>) -> Self {
        self.parts.push(format!("summon {}", entity_type.into()));
        self
    }

    // ── Condition sub-commands ────────────────────────────────────────────────

    /// `if entity <selector>` — execute only if the selector matches at least one entity.
    ///
    /// Produces: `execute if entity <selector> ...`
    pub fn if_entity(mut self, selector: Selector) -> Self {
        self.parts.push(format!("if entity {selector}"));
        self
    }

    /// `unless entity <selector>` — execute only if the selector matches NO entities.
    ///
    /// Produces: `execute unless entity <selector> ...`
    pub fn unless_entity(mut self, selector: Selector) -> Self {
        self.parts.push(format!("unless entity {selector}"));
        self
    }

    /// `if entity @s[team=<team>]` — continue only if the current executing entity is on the given team.
    ///
    /// After `execute as @e[...]`, `@s` refers to each iterated entity, so this checks
    /// **the target entity's** team — not the original executor's team.
    /// Useful for friendly-fire prevention and team-based filters.
    /// Produces: `execute if entity @s[team=<team>] ...`
    pub fn if_on_team(mut self, team: impl Into<String>) -> Self {
        self.parts
            .push(format!("if entity @s[team={}]", team.into()));
        self
    }

    /// `unless entity @s[team=<team>]` — skip execution if the current executing entity is on the given team.
    ///
    /// See [`if_on_team`](Execute::if_on_team) for team context semantics.
    /// Produces: `execute unless entity @s[team=<team>] ...`
    pub fn unless_on_team(mut self, team: impl Into<String>) -> Self {
        self.parts
            .push(format!("unless entity @s[team={}]", team.into()));
        self
    }

    /// `if score <a> <a_obj> = <b> <b_obj>` — continue only if the two scores are equal.
    ///
    /// Produces: `execute if score <a> <a_obj> = <b> <b_obj> ...`
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
    ///
    /// Produces: `execute unless score <a_selector> <obj> = <b_selector> <obj> ...`
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

    /// `if block <pos> <block>` — execute only if the block at `pos` matches the given type.
    ///
    /// Supports block states in the block argument (e.g., `"redstone_wire[power=15]"`).
    /// Produces: `execute if block <pos> <block> ...`
    pub fn if_block(mut self, pos: BlockPos, block: impl Into<String>) -> Self {
        self.parts.push(format!("if block {pos} {}", block.into()));
        self
    }

    /// `unless block <pos> <block>` — execute only if the block at `pos` does NOT match the given type.
    ///
    /// Produces: `execute unless block <pos> <block> ...`
    pub fn unless_block(mut self, pos: BlockPos, block: impl Into<String>) -> Self {
        self.parts
            .push(format!("unless block {pos} {}", block.into()));
        self
    }

    /// `if score <holder> <obj> matches <range>` — execute if a score falls within the given range.
    ///
    /// Range can be `"5"` (exact), `"5.."` (5 or more), `"..5"` (5 or less), or `"1..10"` (between).
    /// Produces: `execute if score <holder> <obj> matches <range> ...`
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

    /// `unless score <holder> <obj> matches <range>` — execute if a score falls OUTSIDE the given range.
    ///
    /// Produces: `execute unless score <holder> <obj> matches <range> ...`
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
    ///
    /// Use [`ScoreCmp`] for the operator (`Eq`, `Lt`, `Le`, `Gt`, `Ge`).
    /// For named shorthands see [`if_score_eq`](Execute::if_score_eq),
    /// [`if_score_lt`](Execute::if_score_lt), etc.
    ///
    /// Produces: `execute if score <a> <a_obj> <cmp> <b> <b_obj> ...`
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
    ///
    /// Produces: `execute unless score <a> <a_obj> <cmp> <b> <b_obj> ...`
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
    pub fn if_score_eq(self, a: impl Into<String>, a_obj: impl Into<String>, b: impl Into<String>, b_obj: impl Into<String>) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Eq, b, b_obj)
    }

    /// `unless score <a> <a_obj> = <b> <b_obj>` — skip if scores are equal.
    pub fn unless_score_eq(self, a: impl Into<String>, a_obj: impl Into<String>, b: impl Into<String>, b_obj: impl Into<String>) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Eq, b, b_obj)
    }

    /// `if score <a> <a_obj> < <b> <b_obj>` — continue if `a` is strictly less than `b`.
    pub fn if_score_lt(self, a: impl Into<String>, a_obj: impl Into<String>, b: impl Into<String>, b_obj: impl Into<String>) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Lt, b, b_obj)
    }

    /// `unless score <a> <a_obj> < <b> <b_obj>` — skip if `a` is strictly less than `b`.
    pub fn unless_score_lt(self, a: impl Into<String>, a_obj: impl Into<String>, b: impl Into<String>, b_obj: impl Into<String>) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Lt, b, b_obj)
    }

    /// `if score <a> <a_obj> <= <b> <b_obj>` — continue if `a` is less than or equal to `b`.
    pub fn if_score_lte(self, a: impl Into<String>, a_obj: impl Into<String>, b: impl Into<String>, b_obj: impl Into<String>) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Le, b, b_obj)
    }

    /// `unless score <a> <a_obj> <= <b> <b_obj>` — skip if `a` is less than or equal to `b`.
    pub fn unless_score_lte(self, a: impl Into<String>, a_obj: impl Into<String>, b: impl Into<String>, b_obj: impl Into<String>) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Le, b, b_obj)
    }

    /// `if score <a> <a_obj> > <b> <b_obj>` — continue if `a` is strictly greater than `b`.
    pub fn if_score_gt(self, a: impl Into<String>, a_obj: impl Into<String>, b: impl Into<String>, b_obj: impl Into<String>) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Gt, b, b_obj)
    }

    /// `unless score <a> <a_obj> > <b> <b_obj>` — skip if `a` is strictly greater than `b`.
    pub fn unless_score_gt(self, a: impl Into<String>, a_obj: impl Into<String>, b: impl Into<String>, b_obj: impl Into<String>) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Gt, b, b_obj)
    }

    /// `if score <a> <a_obj> >= <b> <b_obj>` — continue if `a` is greater than or equal to `b`.
    pub fn if_score_gte(self, a: impl Into<String>, a_obj: impl Into<String>, b: impl Into<String>, b_obj: impl Into<String>) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Ge, b, b_obj)
    }

    /// `unless score <a> <a_obj> >= <b> <b_obj>` — skip if `a` is greater than or equal to `b`.
    pub fn unless_score_gte(self, a: impl Into<String>, a_obj: impl Into<String>, b: impl Into<String>, b_obj: impl Into<String>) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Ge, b, b_obj)
    }

    // ── Data / NBT conditions ─────────────────────────────────────────────────

    /// `if data entity <target> <path>` — continue if entity NBT has a value at `path`.
    ///
    /// ```rust,ignore
    /// Execute::new()
    ///     .if_data_entity(Selector::self_(), "Custom.ready")
    ///     .run_raw("say ready");
    /// ```
    pub fn if_data_entity(mut self, selector: Selector, path: impl Into<String>) -> Self {
        self.parts.push(format!("if data entity {selector} {}", path.into()));
        self
    }

    /// `unless data entity <target> <path>` — skip if entity NBT has a value at `path`.
    pub fn unless_data_entity(mut self, selector: Selector, path: impl Into<String>) -> Self {
        self.parts.push(format!("unless data entity {selector} {}", path.into()));
        self
    }

    /// `if data block <pos> <path>` — continue if block NBT has a value at `path`.
    pub fn if_data_block(mut self, pos: BlockPos, path: impl Into<String>) -> Self {
        self.parts.push(format!("if data block {pos} {}", path.into()));
        self
    }

    /// `unless data block <pos> <path>` — skip if block NBT has a value at `path`.
    pub fn unless_data_block(mut self, pos: BlockPos, path: impl Into<String>) -> Self {
        self.parts.push(format!("unless data block {pos} {}", path.into()));
        self
    }

    /// `if data storage <source> <path>` — continue if storage has a value at `path`.
    ///
    /// ```rust,ignore
    /// Execute::new()
    ///     .if_data_storage("my_pack:state", "phase")
    ///     .run_raw("say phase exists");
    /// ```
    pub fn if_data_storage(mut self, source: impl Into<String>, path: impl Into<String>) -> Self {
        self.parts.push(format!("if data storage {} {}", source.into(), path.into()));
        self
    }

    /// `unless data storage <source> <path>` — skip if storage has a value at `path`.
    pub fn unless_data_storage(mut self, source: impl Into<String>, path: impl Into<String>) -> Self {
        self.parts.push(format!("unless data storage {} {}", source.into(), path.into()));
        self
    }

    // ── World conditions ──────────────────────────────────────────────────────

    /// `if biome <pos> <biome>` — continue if the biome at `pos` matches (1.19.4+).
    ///
    /// ```rust,ignore
    /// Execute::new()
    ///     .if_biome(BlockPos::here(), "minecraft:desert")
    ///     .run_raw("say you're in a desert");
    /// ```
    pub fn if_biome(mut self, pos: BlockPos, biome: impl Into<String>) -> Self {
        self.parts.push(format!("if biome {pos} {}", biome.into()));
        self
    }

    /// `unless biome <pos> <biome>` — skip if the biome at `pos` matches (1.19.4+).
    pub fn unless_biome(mut self, pos: BlockPos, biome: impl Into<String>) -> Self {
        self.parts.push(format!("unless biome {pos} {}", biome.into()));
        self
    }

    /// `if dimension <dimension>` — continue if executing in the given dimension (1.21+).
    ///
    /// ```rust,ignore
    /// Execute::new()
    ///     .if_dimension("minecraft:the_nether")
    ///     .run_raw("say you're in the nether");
    /// ```
    pub fn if_dimension(mut self, dimension: impl Into<String>) -> Self {
        self.parts.push(format!("if dimension {}", dimension.into()));
        self
    }

    /// `unless dimension <dimension>` — skip if executing in the given dimension (1.21+).
    pub fn unless_dimension(mut self, dimension: impl Into<String>) -> Self {
        self.parts.push(format!("unless dimension {}", dimension.into()));
        self
    }

    /// `if loaded <pos>` — continue only if the chunk at `pos` is fully loaded.
    ///
    /// Prevents commands from running in unloaded chunks where they would silently fail.
    pub fn if_loaded(mut self, pos: BlockPos) -> Self {
        self.parts.push(format!("if loaded {pos}"));
        self
    }

    /// `unless loaded <pos>` — skip if the chunk at `pos` is NOT fully loaded.
    pub fn unless_loaded(mut self, pos: BlockPos) -> Self {
        self.parts.push(format!("unless loaded {pos}"));
        self
    }

    /// `if items entity <selector> <slot> <item>` — execute if an entity has a
    /// matching item in the given slot.
    ///
    /// `item` is an item predicate string, e.g. `"minecraft:iron_boots"` or
    /// `"minecraft:leather_boots[minecraft:custom_data={mana_boots:true}]"`.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Run function only if @s has iron boots equipped
    /// Execute::new()
    ///     .if_items_entity(Selector::self_(), ItemSlot::Feet, "minecraft:iron_boots")
    ///     .run(cmd::say("iron boots!"));
    ///
    /// // Tick loop: all players wearing custom mana boots
    /// Execute::new()
    ///     .as_(Selector::all_players())
    ///     .at(Selector::self_())
    ///     .if_items_entity(Selector::self_(), ItemSlot::Feet,
    ///         "minecraft:leather_boots[minecraft:custom_data={mana_boots:true}]")
    ///     .run_fn("ns:on_mana_boots_tick");
    /// ```
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

    /// `unless items entity <selector> <slot> <item>` — skip if the entity has
    /// the item; continue only if it does NOT.
    pub fn unless_items_entity(
        mut self,
        selector: Selector,
        slot: ItemSlot,
        item: impl Into<String>,
    ) -> Self {
        self.parts
            .push(format!("unless items entity {selector} {slot} {}", item.into()));
        self
    }

    /// `if items block <pos> <slot> <item>` — execute if a block container has
    /// a matching item in the given slot.
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

    /// `unless items block <pos> <slot> <item>` — skip if the block container
    /// has the item; continue only if it does NOT.
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
    ///
    /// Predicates are defined in JSON and can check entity properties, NBT, and more.
    /// Produces: `execute if predicate <predicate> ...`
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

    /// `if items entity <selector> <slot> <item>` — execute if the slot holds a matching item (1.20.5+).
    ///
    /// Item argument uses item predicates to match stacks by type, count, NBT, etc.
    /// Produces: `execute if items entity <selector> <slot> <item> ...`
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

    /// `unless items entity <selector> <slot> <item>` — execute if the slot does NOT match (1.20.5+).
    ///
    /// Produces: `execute unless items entity <selector> <slot> <item> ...`
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

    /// `if items entity <selector> <slot_pattern> <item>` — check multiple slots using a pattern (1.20.5+).
    ///
    /// Patterns like `AnyHotbar` or `AnyInventory` allow checking multiple slots at once.
    /// Produces: `execute if items entity <selector> <pattern> <item> ...`
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

    /// `unless items entity <selector> <slot_pattern> <item>` — check pattern does NOT match (1.20.5+).
    ///
    /// Produces: `execute unless items entity <selector> <pattern> <item> ...`
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

    /// `store result score <holder> <objective>` — capture the numeric result of the `run` command into a score.
    ///
    /// The command's return value (e.g., the data byte count from `data get`) is stored as the score.
    /// Produces: `execute store result score <holder> <objective> run ...`
    pub fn store_result_score(mut self, holder: ScoreHolder, objective: impl Into<String>) -> Self {
        self.parts
            .push(format!("store result score {holder} {}", objective.into()));
        self
    }

    /// `store success score <holder> <objective>` — store 1 if the `run` command succeeds, 0 if it fails.
    ///
    /// Unlike `store result`, this captures success/failure rather than a numeric output.
    /// Produces: `execute store success score <holder> <objective> run ...`
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
    ///
    /// The numeric result of the `run` command is multiplied by `scale` then stored
    /// at `path` with the given type. `scale` of `1.0` stores the value unchanged.
    ///
    /// ```rust,ignore
    /// // Store entity's health as a float in Custom.LastHealth:
    /// Execute::new()
    ///     .store_result_nbt(DataTarget::entity(Selector::self_()), "Custom.LastHealth", NbtStoreKind::Float, 1.0)
    ///     .run_raw("data get entity @s Health");
    /// ```
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
    ///
    /// ```rust,ignore
    /// Execute::new()
    ///     .store_result_bossbar("my_pack:health_bar", "value")
    ///     .run_raw("scoreboard players get @s health");
    /// ```
    pub fn store_result_bossbar(mut self, id: impl Into<String>, attribute: impl Into<String>) -> Self {
        self.parts.push(format!("store result bossbar {} {}", id.into(), attribute.into()));
        self
    }

    /// `store success bossbar <id> <attribute>` — write success/failure into a bossbar attribute.
    pub fn store_success_bossbar(mut self, id: impl Into<String>, attribute: impl Into<String>) -> Self {
        self.parts.push(format!("store success bossbar {} {}", id.into(), attribute.into()));
        self
    }

    // ── Terminal ──────────────────────────────────────────────────────────────

    /// `run <command>` — finalize the execute chain and run the given command.
    ///
    /// This must be called last. The command receives all context changes from prior methods.
    /// Returns the complete execute command string ready to emit.
    /// Produces: `execute <sub-commands> run <command>`
    pub fn run(mut self, cmd: impl Command) -> String {
        self.parts.push(format!("run {cmd}"));
        format!("execute {}", self.parts.join(" "))
    }

    /// Like [`run`](Execute::run) but accepts a raw unparsed command string.
    ///
    /// Use this when you have a command as a plain string or when the command type
    /// doesn't implement the `Command` trait.
    /// Produces: `execute <sub-commands> run <command>`
    pub fn run_raw(mut self, cmd: impl Into<String>) -> String {
        self.parts.push(format!("run {}", cmd.into()));
        format!("execute {}", self.parts.join(" "))
    }
}

impl fmt::Display for Execute {
    /// Render the execute chain without a `run` clause.
    ///
    /// Useful for debugging or when building the command piecemeal. The result
    /// is incomplete and cannot be executed by Minecraft without adding a `run` clause.
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
        assert_eq!(unless, "execute unless score @s mana > #const zero run say no mana");
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
        assert_eq!(s, "execute if data storage my_pack:state phase run say has phase");
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
        assert_eq!(s, "execute if dimension minecraft:the_nether run say nether");
    }

    #[test]
    fn positioned_over_test() {
        use super::super::Vec3;
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
