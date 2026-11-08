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
    Amount, Audience, Condition, EntityQuery, EntitySort, EventHook, MessageChannel, PlayerEvent,
    Relation, ScaledField, ScoreOperand, ScoreOperator, ScoreRelation, StateAttached,
    TeleportDestination, TextColor, TextPart, TextStyle,
};
pub use operation::{Action, Operation};
pub use resources::{
    CookingMethod, CustomItem, Rarity, Recipe, RecipeOutput, ResourceTag, TagKind,
};
pub use state::{FieldReference, ScoreComparison, ScoreField, State};
