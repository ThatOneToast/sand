//! Owned frontend-neutral gameplay model, shared by construction and decoding.
mod envelope;
mod function;
mod operation;
mod state;
pub use envelope::{Module, ModuleSource, Pack, Program, ProgramTarget};
pub use function::{ExecutionContext, Function, FunctionReference, TagMembership};
pub use operation::{Action, Operation};
pub use state::{FieldReference, ScoreComparison, ScoreField, State};
