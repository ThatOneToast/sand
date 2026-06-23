//! Typed Minecraft command builders.
//!
//! Each Minecraft command (or family of commands) is represented as a Rust
//! struct or free function that serializes to the correct command string via
//! [`std::fmt::Display`]. All types implement the [`Command`] marker trait.
//!
//! String-building types are provided by [`sand_commands`] and re-exported
//! here. Sand-core-specific modules contain only datapack-level concepts.
//!
//! # Module layout
//!
//! | Source | Contents |
//! |---|---|
//! | `sand_commands` (re-exported) | All command builders: blocks, coordinates, execute, selectors, scoreboard, NBT, sound, display, inventory, particles … |
//! | [`cooldown`] | `Cooldown` — scoreboard-based ability cooldown timer |
//! | [`data`] | `Storage`, `StorageKind` — named NBT namespaces; bridges to `Objective::load_from` via `From<&Storage> for String` |
//! | [`fn_macros`] | `macro_var`, `macro_line`, `function_with` — function macro utilities |
//!
//! # Example
//! ```rust,ignore
//! use sand_core::cmd::{self, Execute, Selector};
//!
//! mcfunction! {
//!     cmd::give(Selector::all_players(), "diamond_sword").count(1);
//!     cmd::kill(Selector::all_entities().tag("enemy"));
//!     Execute::new()
//!         .as_(Selector::all_players())
//!         .if_score_matches("@s", "playtime", "100..")
//!         .run(cmd::say("100 ticks!"));
//! }
//! ```

// ── Internal modules (sand-core-specific) ─────────────────────────────────────

mod cooldown;
mod data;
mod effect;
mod fn_macros;
mod typed_execute;

// ── Re-exports from sand-commands ─────────────────────────────────────────────

/// The `Build` trait: every command builder implements `build(&self) -> String`.
pub use sand_commands::Build;

/// Trait for types resolving to a `function <id>` command.
pub use crate::function::IntoFunctionRef;

// Block placement
pub use sand_commands::{
    BlockState, CloneBlocks, CloneMaskMode, CloneMode, Fill, FillMode, SetBlock, SetBlockMode,
};
// Coordinate types
pub use sand_commands::{BlockPos, Coord, Rotation, Vec2, Vec3};
// Player display commands
pub use sand_commands::{Actionbar, Bossbar, BossbarColor, BossbarStyle, Title};
// Execute builder
pub use sand_commands::Execute;
// Execute argument types
pub use sand_commands::{Anchor, ItemSlot, NbtStoreKind, Swizzle};
// Inventory manipulation
#[allow(deprecated)]
pub use sand_commands::{Inventory, InventorySlot, SlotPattern};
// Particle effects
pub use sand_commands::{Particle, ParticleBuilder, ParticleEffect, ParticleSpread};
// Entity/player targeting
pub use sand_commands::{
    Damage as DamageBuilder, DamageAmount, DamageKind, EntityTarget, EntityTargets, GameMode, Many,
    One, PlayerTarget, PlayerTargets, Selector, SingleEntity, SinglePlayer, SortOrder, TargetBase,
};
// Sound
pub use sand_commands::{Sound, SoundSource};
// Text components
pub use sand_commands::{ChatColor, ClickEvent, HoverEvent, Text, TextComponent};
// NBT types — owned by sand-commands
pub use sand_commands::{DataModify, DataTarget, NbtValue, data_modify};
// Scoreboard types — owned by sand-commands
// Note: &Storage satisfies Objective::load_from's `impl Into<String>` parameter
// via the `From<&Storage> for String` impl in mod data.
pub use sand_commands::{
    DisplaySlot, Objective, ScoreCmp, ScoreHolder, ScoreOp, ScoreboardPlayersOperation,
    scoreboard_players_operation,
};
// NOTE: sand_commands::builtins::* is intentionally NOT re-exported here because
// sand-core provides its own generated command builders (see _generated below)
// that would conflict. Use sand_commands directly for the free-function builders.

// ── Re-exports from internal modules ─────────────────────────────────────────

pub use cooldown::Cooldown;
// Storage and StorageKind are datapack concepts defined only in sand-core.
// All other NBT/scoreboard types come from sand-commands above.
pub use data::{Storage, StorageKind};
pub use effect::{EffectGive, effect_clear, effect_clear_effect, effect_give, effect_give_raw};
pub use fn_macros::{function_with, macro_line, macro_var};
pub use typed_execute::{ConditionedExecute, ExecuteExt, TypedExecute};

/// Call a function by resolved reference.
///
/// Accepts registered `#[function]` pointers, [`FunctionRef`], [`ResourceLocation`],
/// and raw path strings.
///
/// # Examples
///
/// ```rust,ignore
/// use sand_core::prelude::*;
///
/// // Local registered function pointer (requires `use IntoFunctionRef`)
/// cmd::call(ate_golden_apple);
///
/// // External function ref
/// cmd::call(FunctionRef::external("other_pack:api/do_thing").unwrap());
///
/// // Resource location
/// cmd::call(ResourceLocation::new("my_pack", "my_func").unwrap());
/// ```
pub fn call(id: impl crate::function::IntoFunctionRef) -> String {
    id.into_function_command()
}

/// Resolve a function identifier to its `namespace:path` resource location.
///
/// # Examples
///
/// ```rust,ignore
/// let loc = cmd::function_id(ate_golden_apple);
/// assert_eq!(loc, "powers:ate_golden_apple");
/// ```
pub fn function_id(id: impl crate::function::IntoFunctionRef) -> String {
    id.into_function_id()
}

/// Show a typed datapack dialog to one or more players.
///
/// Dialogs are available in Minecraft Java 1.21.6+ / pack format 80+.
/// The command emitted is `dialog show <targets> <dialog>`.
///
/// # Examples
///
/// ```rust,ignore
/// use sand_core::prelude::*;
///
/// cmd::show_dialog(Selector::self_(), DialogRef::local("welcome"));
/// cmd::show_dialog(
///     Selector::all_players(),
///     DialogRef::external("other_pack:settings").unwrap(),
/// );
/// ```
pub fn show_dialog(
    selector: impl std::fmt::Display,
    dialog: impl sand_components::dialog::IntoDialogRef,
) -> String {
    format!("dialog show {selector} {}", dialog.into_dialog_ref())
}

/// `return fail` — stop the current function with a failure return value.
///
/// In Minecraft 1.20.2+, `return fail` terminates the current `.mcfunction`
/// and reports failure (return value −1) to callers using `execute … run function`.
/// Use inside branch or helper functions to halt that branch.
///
/// ```rust,ignore
/// when(HAS_CELLS.of("@s").is_true()).then_all([
///     tellraw(Selector::self_(), Text::new("Already granted")),
///     cmd::return_fail(),
/// ]);
/// ```
pub fn return_fail() -> String {
    "return fail".to_string()
}

/// `return <value>` — stop the current function with an integer return value.
///
/// `cmd::return_cmd(0)` → `return 0` (success, also readable by `execute store result`).
/// `cmd::return_cmd(1)` → `return 1`.
///
/// In Minecraft 1.20.2+, `return <n>` terminates the current `.mcfunction`
/// with the given result code. Callers using `execute … run function` see this value.
///
/// ```rust,ignore
/// unless(HAS_CELLS.of("@s").is_true()).then_all([
///     HAS_CELLS.enable("@s"),
///     cmd::return_cmd(0),
/// ]);
/// ```
pub fn return_cmd(value: i32) -> String {
    format!("return {value}")
}

/// Explicit escape hatch for raw Minecraft command syntax.
///
/// Prefer typed builders for normal datapack code. Use this for interop with
/// other datapacks, modded commands, snapshot-only syntax, future features not
/// modeled by Sand yet, or focused debugging.
pub fn raw(command: impl Into<String>) -> String {
    command.into()
}

/// A typed Minecraft command that can be serialized to a command string.
///
/// All command builders generated from the Minecraft command tree implement
/// this trait. It is also implemented by [`Execute`] for chaining.
///
/// Since [`Command`] requires [`std::fmt::Display`], you can use command
/// builders directly in [`crate::mcfunction!`]:
/// ```rust,ignore
/// mcfunction! {
///     cmd::kill(Selector::all_entities().tag("mob"));
///     "raw fallback command string";
/// }
/// ```
pub trait Command: std::fmt::Display {}

// Include the generated command builders from commands.json.
#[allow(warnings, clippy::all)]
mod _generated {
    use super::*;
    use crate::ResourceLocation;
    include!(concat!(env!("OUT_DIR"), "/commands.rs"));
}
#[allow(unused)]
pub use _generated::*;

#[cfg(test)]
mod tests {
    use crate::resource_ref::DialogRef;

    #[test]
    fn raw_escape_hatch_is_explicit() {
        assert_eq!(
            super::raw("function other_pack:api/do_special_thing"),
            "function other_pack:api/do_special_thing"
        );
    }

    #[test]
    fn show_dialog_local_ref() {
        assert_eq!(
            super::show_dialog(super::Selector::self_(), DialogRef::local("welcome")),
            "dialog show @s __sand_local:welcome"
        );
    }

    #[test]
    fn show_dialog_external_ref() {
        assert_eq!(
            super::show_dialog(
                super::Selector::all_players(),
                DialogRef::external("other_pack:settings").unwrap()
            ),
            "dialog show @a other_pack:settings"
        );
    }
}
