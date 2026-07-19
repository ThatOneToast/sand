//! The default authoring import.
//!
//! ```rust,ignore
//! use sand::prelude::*;
//! ```
//!
//! The prelude covers the common vocabulary of ordinary datapack development:
//! the attribute macros, typed commands, selectors, conditions, state, items,
//! components, events, dialogs, text, and resource references. Less common
//! APIs stay in their topic modules (`sand::event`, `sand::item`, …), and
//! low-level export hooks live in [`crate::advanced`].

// Attribute + declarative macros.
pub use crate::{
    all, any, armor_event, component, event, function, item, mcfunction, run_fn, sand_state,
    schedule,
};

// The `cmd` module itself, so `cmd::say(...)` works from the prelude.
pub use crate::cmd;

// The curated implementation prelude (commands, selectors, conditions, state,
// entities, events, components, dialogs, text, resource refs, raw escape
// hatches). Compiler-facing symbols are excluded at the source.
pub use sand_core::prelude::*;
