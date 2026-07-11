//! Supported lower-level APIs for custom framework integrations.
//!
//! Most datapack authors should start with [`crate::prelude`]. Use this module
//! when you are building custom export tooling, dynamic function registries, or
//! raw interop around Minecraft features that Sand does not model yet.
//!
//! These APIs are public and supported, but they expose more of Sand's export
//! and generated-output machinery than ordinary packs need. Prefer typed
//! builders from the prelude when they cover the use case.

pub use crate::component::{
    ComponentContent, ComponentRecord, export_components_json, try_export_components,
    try_export_components_json,
};
pub use crate::function::{
    ComponentFactory, EventDescriptor, EventDispatch, EventPathEntry, FunctionDescriptor,
    FunctionPointerEntry, FunctionPointerTypeEntry, FunctionTagDescriptor, ScheduleDescriptor,
    TempScoreboard, drain_dyn_fns, register_dyn_fn, register_dyn_fn_dedup,
};
pub use crate::state::{
    drain_load_commands, drain_tick_commands, register_load_objective, register_tick_handler,
};
pub use sand_components::{RawCommand, RawComponent, RawJson, RawSnbt};
