//! Owned frontend-neutral gameplay model, shared by construction and decoding.
mod envelope;
mod flow;
mod function;
mod gameplay;
mod operation;
mod resources;
mod state;
pub use envelope::{Module, ModuleSource, Pack, Program, ProgramTarget};
pub use flow::{Flow, FlowHook, FlowHookPhase, FlowTransition};
pub use function::{ExecutionContext, Function, FunctionReference, TagMembership};
pub use gameplay::{
    Amount, Audience, BlockMatches, BlockPosition, Condition, EntitiesExist, EntityFlag,
    EntityQuery, EntitySort, EventHook, FlagSet, ItemHeld, MessageChannel, PlayerEvent,
    PlayerGameMode, Relation, ScaledField, ScoreOperand, ScoreOperator, ScoreRelation,
    StateAttached, TeleportDestination, TextColor, TextPart, TextStyle, Weather,
};
pub use operation::{Action, Operation};
pub use resources::{
    BossBar, BossBarColor, BossBarStyle, CookingMethod, CustomItem, LootItem, LootTable, Rarity,
    Recipe, RecipeOutput, ResourceTag, TagKind,
};
pub use state::{FieldReference, ScoreComparison, ScoreField, State};
