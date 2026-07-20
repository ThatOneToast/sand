//! Sand's compiler: turns link-time registered authoring declarations into
//! deterministic datapack output.
//!
//! Phase 2 of the API/compiler reorganization (ADR 001) moved the export
//! machinery formerly in `component.rs` here, organized by pipeline phase
//! under [`export`]. The public paths in [`crate::component`] are unchanged.

pub(crate) mod export;
