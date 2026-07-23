//! Typed intermediate representation for `execute` subcommands.
//!
//! Public builders create these nodes; datapack authors are not expected to
//! author this IR directly. Rendering is deliberately the last step.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use crate::coord::{BlockPos, Rotation, Vec3};
use crate::error::{CommandError, CommandResult};
use crate::execute_args::{Anchor, ItemSlot, NbtStoreKind, Swizzle};
use crate::nbt::DataTarget;
use crate::render::CommandProfile;
use crate::scoreboard::{ScoreCmp, ScoreHolder};
use crate::selector::Selector;

/// A capability required by a typed execute operation or condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecuteCapability {
    /// `execute if/unless items`, introduced in Java 1.20.5.
    ItemConditions,
}

impl ExecuteCapability {
    /// Stable capability name used by diagnostics and tooling.
    pub const fn name(self) -> &'static str {
        match self {
            Self::ItemConditions => "ExecuteItemCondition",
        }
    }

    /// First Java Edition version supporting this capability.
    pub const fn minimum_version(self) -> &'static str {
        match self {
            Self::ItemConditions => "1.20.5",
        }
    }

    pub(crate) fn is_supported(self, profile: &CommandProfile) -> bool {
        match self {
            Self::ItemConditions => profile.is_at_least(1, 20, 5),
        }
    }
}

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

    pub(crate) fn required_capability(&self) -> Option<ExecuteCapability> {
        match self {
            Self::ItemsEntity { .. } | Self::ItemsBlock { .. } => {
                Some(ExecuteCapability::ItemConditions)
            }
            _ => None,
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

    pub(crate) fn required_capability(&self) -> Option<ExecuteCapability> {
        match self {
            Self::If(condition) | Self::Unless(condition) => condition.required_capability(),
            _ => None,
        }
    }

    #[doc(hidden)]
    pub fn validate_version(&self, index: usize, profile: &CommandProfile) -> CommandResult<()> {
        let Some(capability) = self.required_capability() else {
            return Ok(());
        };
        if capability.is_supported(profile) {
            return Ok(());
        }
        Err(CommandError::new(
            "Execute",
            "operation",
            format!(
                "unsupported execute operation `{}` for Minecraft {}; required capability {} (Minecraft {}+). Use a predicate-backed check on older versions",
                self.render(),
                profile.requested_version(),
                capability.name(),
                capability.minimum_version(),
            ),
        )
        .with_code("SAND-COMMAND-VERSION")
        .with_context(format!("Execute operation {index}")))
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

#[derive(Debug, Clone)]
struct Requirement {
    capability: ExecuteCapability,
    operation: String,
}

type Requirements = BTreeMap<String, Vec<Requirement>>;

fn requirements() -> &'static Mutex<Requirements> {
    static REQUIREMENTS: OnceLock<Mutex<Requirements>> = OnceLock::new();
    REQUIREMENTS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Record capability metadata for a line produced by typed execute IR.
///
/// This side table preserves the historical `String` terminal API while the
/// export pipeline still stores function bodies as strings.
#[doc(hidden)]
pub fn register_line(line: &str, operations: &[ExecuteOp]) {
    let required: Vec<_> = operations
        .iter()
        .filter_map(|operation| {
            operation
                .required_capability()
                .map(|capability| Requirement {
                    capability,
                    operation: operation.render(),
                })
        })
        .collect();
    if !required.is_empty() {
        requirements()
            .lock()
            .expect("execute requirement registry poisoned")
            .insert(line.to_string(), required);
    }
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    let guard = requirements()
        .lock()
        .expect("execute requirement registry poisoned");
    let Some(required) = guard.get(line) else {
        return Ok(());
    };
    for requirement in required {
        if !requirement.capability.is_supported(profile) {
            return Err(CommandError::new(
                "Execute",
                "operation",
                format!(
                    "unsupported execute operation `{}` for Minecraft {}; required capability {} (Minecraft {}+). Use a predicate-backed check on older versions",
                    requirement.operation,
                    profile.requested_version(),
                    requirement.capability.name(),
                    requirement.capability.minimum_version(),
                ),
            )
            .with_code("SAND-COMMAND-VERSION"));
        }
    }
    Ok(())
}
