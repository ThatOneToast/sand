//! Convenience re-export of the most commonly used Sand types.
//!
//! Bring the whole prelude into scope with:
//! ```rust,ignore
//! use sand_core::prelude::*;
//! ```

// ── Conditions & execute wiring ───────────────────────────────────────────────

pub use crate::cmd::{ConditionedExecute, ExecuteExt, TypedExecute};
pub use crate::condition::{Condition, ExecutePlan};

// ── Command builders ──────────────────────────────────────────────────────────

// Execute and Selector already pulled in via cmd above — no duplicate needed

// ── State variables ───────────────────────────────────────────────────────────

pub use crate::state::{
    Cooldown, Flag, FlagRef, NbtPath, ScoreRef, ScoreVar, StorageVar, Ticks, Timer,
};

// ── Resource refs ─────────────────────────────────────────────────────────────

pub use crate::resource_ref::{
    AdvancementRef, DialogRef, FunctionRef, LootTableRef, PredicateRef, RecipeRef,
};

// ── Version gating ────────────────────────────────────────────────────────────

pub use crate::version::{MinecraftVersion, VersionProfile};

// ── Dialog builders ───────────────────────────────────────────────────────────

pub use sand_components::dialog::{Dialog, DialogAction, DialogBody, DialogButton, DialogKind};

// ── Text / chat ───────────────────────────────────────────────────────────────

pub use sand_commands::{ChatColor, ClickEvent, HoverEvent, Text, TextComponent};

// ── Macros (re-exported as items so `use prelude::*` captures them) ───────────
// The macros themselves are `#[macro_export]` at the crate root, so they
// are already in scope as `sand_core::mcfunction!` etc. after a wildcard import
// of items. Nothing extra is needed here.
