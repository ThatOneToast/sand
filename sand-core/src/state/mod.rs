//! State implementation primitives.
//!
//! The public façade's `#[derive(State)]` API is the canonical declaration
//! path. The types here are retained as low-level command-building primitives
//! for framework internals and advanced code that intentionally manages an
//! existing objective or storage location.

pub mod cooldown;
pub mod flag;
pub mod flow;
pub mod lifecycle;
pub(crate) mod registry;
pub mod score;
pub mod storage;
pub mod timer;
pub mod typed_state;

pub use cooldown::Cooldown;
pub use flag::{Flag, FlagRef};
pub use flow::{FlowTransitionBuilder, IntoStateCommands, StateFlow, StateTransitionBuilder};
pub use lifecycle::{
    StateCleanup, StateInit, StateLifecycle, StateMigrate, StateProvision, StateReconcile,
    StateTick,
};
#[doc(hidden)]
pub use registry::{
    StateDescriptor, StateHookDescriptor, StateLifecycleDescriptor, StateMigrationDescriptor,
    StateScope,
};
pub use score::{
    ScoreConst, ScoreConstants, ScoreExpr, ScoreOperand, ScoreOperation, ScoreRef, ScoreVar,
};
pub use storage::{
    BlockNbt, DataCommand, EntityNbt, Nbt, NbtLocation, NbtPath, NbtRef, NbtTarget, SnbtCompound,
    SnbtValue, StorageField, StorageLocation, StorageSchema, StorageVar, UntypedNbt,
};
pub use timer::{Ticks, Timer};
pub use typed_state::{GameState, GameStateRef, TypedGameState};
