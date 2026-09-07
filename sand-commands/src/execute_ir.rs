//! Typed intermediate representation for `execute` subcommands.
//!
//! Public builders create these nodes; datapack authors are not expected to
//! author this IR directly. Rendering is deliberately the last step.

use crate::coord::{BlockPos, Rotation, Vec3};
use crate::execute_args::{Anchor, ItemSlot, NbtStoreKind, Swizzle};
use crate::nbt::DataTarget;
use crate::scoreboard::{ScoreCmp, ScoreHolder};
use crate::selector::Selector;

/// One typed condition body used after `execute if` or `execute unless`.
///
/// [`Raw`](Self::Raw) is an explicit opaque escape hatch. Its contents render
/// unchanged and are not interpreted, rewritten, or version-checked by Sand.
#[derive(Debug, Clone)]
pub enum ConditionIr {
    Entity(Selector),
    ScoreMatches {
        holder: ScoreHolder,
        objective: String,
        range: String,
    },
    ScoreCompare {
        left: ScoreHolder,
        left_objective: String,
        op: ScoreCmp,
        right: ScoreHolder,
        right_objective: String,
    },
    Block {
        position: BlockPos,
        block: String,
    },
    Predicate(String),
    Data {
        target: DataTarget,
        path: String,
    },
    ItemsEntity {
        target: Selector,
        slot: ItemSlot,
        item: String,
    },
    ItemsBlock {
        position: BlockPos,
        slot: ItemSlot,
        item: String,
    },
    Biome {
        position: BlockPos,
        biome: String,
    },
    Dimension(String),
    Loaded(BlockPos),
    Team(String),
    /// Opaque condition text without the leading `if`/`unless`.
    Raw(String),
}

impl ConditionIr {
    /// Render only the condition body, without an `if`/`unless` prefix.
    pub fn render(&self) -> String {
        match self {
            Self::Entity(target) => format!("entity {target}"),
            Self::ScoreMatches {
                holder,
                objective,
                range,
            } => format!("score {holder} {objective} matches {range}"),
            Self::ScoreCompare {
                left,
                left_objective,
                op,
                right,
                right_objective,
            } => format!("score {left} {left_objective} {op} {right} {right_objective}"),
            Self::Block { position, block } => format!("block {position} {block}"),
            Self::Predicate(predicate) => format!("predicate {predicate}"),
            Self::Data { target, path } => format!("data {target} {path}"),
            Self::ItemsEntity { target, slot, item } => {
                format!("items entity {target} {slot} {item}")
            }
            Self::ItemsBlock {
                position,
                slot,
                item,
            } => format!("items block {position} {slot} {item}"),
            Self::Biome { position, biome } => format!("biome {position} {biome}"),
            Self::Dimension(dimension) => format!("dimension {dimension}"),
            Self::Loaded(position) => format!("loaded {position}"),
            Self::Team(team) => format!("entity @s[team={team}]"),
            Self::Raw(fragment) => fragment.clone(),
        }
    }
}

/// Destination for `execute store result/success`.
#[derive(Debug, Clone)]
pub enum ExecuteStoreTarget {
    Score {
        holder: ScoreHolder,
        objective: String,
    },
    Nbt {
        target: DataTarget,
        path: String,
        kind: NbtStoreKind,
        scale: f64,
    },
    Bossbar {
        id: String,
        attribute: String,
    },
}

/// One ordered, typed `execute` subcommand.
///
/// [`Raw`](Self::Raw) is the only operation-level opaque escape hatch.
#[derive(Debug, Clone)]
pub enum ExecuteOp {
    As(Selector),
    At(Selector),
    Positioned(Vec3),
    PositionedAs(Selector),
    PositionedOver(String),
    Rotated(Rotation),
    RotatedAs(Selector),
    Facing(Vec3),
    FacingEntity {
        target: Selector,
        anchor: Anchor,
    },
    Anchored(Anchor),
    In(String),
    Align(Swizzle),
    On(String),
    Summon(String),
    If(ConditionIr),
    Unless(ConditionIr),
    StoreResult(ExecuteStoreTarget),
    StoreSuccess(ExecuteStoreTarget),
    /// Opaque execute subcommand text, rendered verbatim.
    Raw(String),
}

impl ExecuteOp {
    /// Render this operation without the leading `execute` keyword.
    pub fn render(&self) -> String {
        match self {
            Self::As(target) => format!("as {target}"),
            Self::At(target) => format!("at {target}"),
            Self::Positioned(position) => format!("positioned {position}"),
            Self::PositionedAs(target) => format!("positioned as {target}"),
            Self::PositionedOver(heightmap) => format!("positioned over {heightmap}"),
            Self::Rotated(rotation) => format!("rotated {rotation}"),
            Self::RotatedAs(target) => format!("rotated as {target}"),
            Self::Facing(position) => format!("facing {position}"),
            Self::FacingEntity { target, anchor } => {
                format!("facing entity {target} {anchor}")
            }
            Self::Anchored(anchor) => format!("anchored {anchor}"),
            Self::In(dimension) => format!("in {dimension}"),
            Self::Align(axes) => format!("align {axes}"),
            Self::On(relation) => format!("on {relation}"),
            Self::Summon(entity_type) => format!("summon {entity_type}"),
            Self::If(condition) => format!("if {}", condition.render()),
            Self::Unless(condition) => format!("unless {}", condition.render()),
            Self::StoreResult(target) => format!("store result {}", render_store(target)),
            Self::StoreSuccess(target) => format!("store success {}", render_store(target)),
            Self::Raw(fragment) => fragment.clone(),
        }
    }
}

fn render_store(target: &ExecuteStoreTarget) -> String {
    match target {
        ExecuteStoreTarget::Score { holder, objective } => {
            format!("score {holder} {objective}")
        }
        ExecuteStoreTarget::Nbt {
            target,
            path,
            kind,
            scale,
        } => format!("{target} {path} {kind} {scale}"),
        ExecuteStoreTarget::Bossbar { id, attribute } => {
            format!("bossbar {id} {attribute}")
        }
    }
}
