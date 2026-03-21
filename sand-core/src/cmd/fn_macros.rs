//! Minecraft function macro utilities (requires Minecraft 1.20.2+).
//!
//! Minecraft **function macros** let you inject runtime NBT values into command
//! lines. Any line in a `.mcfunction` file prefixed with `$` is a *macro line*;
//! within it, `$(name)` placeholders are substituted from a compound NBT source
//! provided at the call site.
//!
//! This module is named `fn_macros` (not `macros`) to avoid confusion with
//! Rust's own `macro_rules!` / procedural macro system.
//!
//! # Three-piece workflow
//!
//! 1. **[`macro_var`]** — produce a `$(name)` placeholder string for embedding
//!    in NBT paths, values, or command fragments.
//! 2. **[`macro_line`]** — prepend `$` to a command, marking it as a macro line
//!    so Minecraft performs substitution at runtime.
//! 3. **[`function_with`]** — generate `function <name> with <source> <path>`,
//!    which calls the macro function and passes the variables compound.
//!
//! # Full example
//!
//! ```rust,ignore
//! use sand_core::cmd::{fn_macros::{macro_line, macro_var, function_with}, DataTarget, Storage};
//! use sand_core::mcfunction;
//!
//! static PLAYERS: Storage = Storage::per_player("my_pack:players");
//! static TEMP:    Storage = Storage::global("my_pack:temp");
//!
//! // ── Macro function: initialize named player's data ───────────────────────
//! // Called with a vars compound {"player": "<name>"}
//! fn init_player_fn() -> Vec<String> {
//!     let p = macro_var("player");  // → "$(player)"
//!     mcfunction![
//!         macro_line(PLAYERS.get_or_insert(format!("{p}.kills"),  0_i32));
//!         macro_line(PLAYERS.get_or_insert(format!("{p}.deaths"), 0_i32));
//!     ]
//! }
//!
//! // ── Caller: store the player name, then invoke the macro function ────────
//! fn on_player_join() -> Vec<String> {
//!     mcfunction![
//!         TEMP.insert("vars.player", "Steve");
//!         function_with("my_pack:init_player", DataTarget::storage(TEMP.id()), "vars");
//!     ]
//! }
//! ```
//!
//! # Runtime substitution mechanics
//!
//! Minecraft reads the NBT compound at `<source> <path>` and substitutes each
//! `$(key)` inside the macro function with its corresponding value from the
//! compound. The compound must exist and be non-empty before the function is
//! called.

use super::DataTarget;

// ── macro_var ─────────────────────────────────────────────────────────────────

/// Returns a `$(name)` placeholder string for use inside a macro function line.
///
/// Embed the result anywhere in a command string that will be wrapped in
/// [`macro_line`]. Minecraft replaces `$(name)` at runtime with the matching
/// key from the variables compound passed to the function call.
///
/// # Example
/// ```
/// use sand_core::cmd::macro_var;
///
/// assert_eq!(macro_var("player"), "$(player)");
/// assert_eq!(macro_var("uuid"), "$(uuid)");
///
/// // Building a path with a variable:
/// let p = macro_var("player");
/// let path = format!("{p}.score");
/// assert_eq!(path, "$(player).score");
/// ```
pub fn macro_var(name: &str) -> String {
    format!("$({name})")
}

// ── macro_line ────────────────────────────────────────────────────────────────

/// Mark a command string as a **macro line** by prepending `$`.
///
/// Lines starting with `$` in a `.mcfunction` file are macro lines: Minecraft
/// processes all `$(name)` placeholders before executing the command. Regular
/// (non-macro) lines are never substituted even if they contain `$(...)`.
///
/// Pass any command (from a builder, [`macro_var`] interpolation, or a plain
/// string) and `macro_line` will prepend the `$` marker.
///
/// # Example
/// ```
/// use sand_core::cmd::{macro_line, macro_var};
///
/// assert_eq!(macro_line("say hello"), "$say hello");
///
/// let player = macro_var("player");
/// assert_eq!(
///     macro_line(format!("say Hello, {player}!")),
///     "$say Hello, $(player)!"
/// );
/// ```
pub fn macro_line(cmd: impl std::fmt::Display) -> String {
    format!("${cmd}")
}

// ── function_with ─────────────────────────────────────────────────────────────

/// Generate `function <name> with <source> <path>` — call a macro function.
///
/// This command invokes the named function in **macro mode**, substituting all
/// `$(key)` placeholders from the NBT compound found at `source` / `path`.
///
/// # Source types
///
/// | `DataTarget` variant | Reads variables from |
/// |---|---|
/// | `DataTarget::storage(id)` | Named NBT storage |
/// | `DataTarget::entity(selector)` | Entity's NBT compound |
/// | `DataTarget::block(pos)` | Block entity NBT |
///
/// # Example
/// ```rust,ignore
/// use sand_core::cmd::{function_with, DataTarget, Storage};
///
/// static TEMP: Storage = Storage::global("my_pack:temp");
///
/// // Pre-populate vars, then call the macro function
/// TEMP.insert("vars.player", "Steve");
/// function_with("my_pack:init_player", DataTarget::storage(TEMP.id()), "vars")
/// // → "function my_pack:init_player with storage my_pack:temp vars"
/// ```
pub fn function_with(
    name: impl std::fmt::Display,
    source: DataTarget,
    path: impl Into<String>,
) -> String {
    format!("function {name} with {source} {}", path.into())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::{Selector, Storage};

    static PLAYERS: Storage = Storage::per_player("my_pack:players");
    static TEMP: Storage = Storage::global("my_pack:temp");

    #[test]
    fn macro_var_format() {
        assert_eq!(macro_var("player"), "$(player)");
        assert_eq!(macro_var("uuid"), "$(uuid)");
    }

    #[test]
    fn macro_line_prepends_dollar() {
        assert_eq!(macro_line("say hello"), "$say hello");
        assert_eq!(
            macro_line(format!("say {}", macro_var("player"))),
            "$say $(player)"
        );
    }

    #[test]
    fn macro_line_with_storage_insert() {
        let p = macro_var("player");
        let cmd = macro_line(PLAYERS.insert(format!("{p}.kills"), 0_i32));
        assert_eq!(
            cmd,
            "$data modify storage my_pack:players $(player).kills set value 0"
        );
    }

    #[test]
    fn macro_line_with_get_or_insert() {
        let p = macro_var("player");
        let cmd = macro_line(PLAYERS.get_or_insert(format!("{p}.deaths"), 0_i32));
        assert_eq!(
            cmd,
            "$execute unless data storage my_pack:players $(player).deaths run data modify storage my_pack:players $(player).deaths set value 0"
        );
    }

    #[test]
    fn function_with_storage() {
        let cmd = function_with(
            "my_pack:init_player",
            DataTarget::storage(TEMP.id()),
            "vars",
        );
        assert_eq!(
            cmd,
            "function my_pack:init_player with storage my_pack:temp vars"
        );
    }

    #[test]
    fn function_with_entity() {
        let cmd = function_with(
            "my_pack:on_hit",
            DataTarget::entity(Selector::self_()),
            "Custom.macro_args",
        );
        assert_eq!(
            cmd,
            "function my_pack:on_hit with entity @s Custom.macro_args"
        );
    }
}
