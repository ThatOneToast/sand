//! Minecraft function macro support (1.20.2+).
//!
//! Minecraft function macros let you inject runtime values into command lines.
//! Any line in a `.mcfunction` file that starts with `$` is a **macro line**;
//! placeholders written as `$(name)` are substituted at runtime from a compound
//! NBT source passed to the function call.
//!
//! # Example — per-player storage keyed by name
//!
//! ```rust,ignore
//! use sand_core::cmd::{macro_line, macro_var, function_with, DataTarget, Storage};
//! use sand_core::mcfunction;
//!
//! static PLAYERS: Storage = Storage::per_player("my_pack:players");
//! static TEMP:    Storage = Storage::global("my_pack:temp");
//!
//! // ── Macro function: initialize a named player's data ─────────────────────
//! // Called with a vars compound that has {"player": "<name>"}
//! fn init_player_fn() -> Vec<String> {
//!     let p = macro_var("player");  // → "$(player)"
//!     mcfunction![
//!         macro_line(PLAYERS.get_or_insert(format!("{p}.kills"),  0_i32));
//!         macro_line(PLAYERS.get_or_insert(format!("{p}.deaths"), 0_i32));
//!     ]
//! }
//!
//! // ── Call site: set up vars, then call the macro function ─────────────────
//! // (within `execute as @s` so @s is the player)
//! fn on_player_join() -> Vec<String> {
//!     mcfunction![
//!         // Copy the player's display name into the temp vars compound.
//!         // Replace this with however you obtain the player identifier.
//!         TEMP.insert("vars.player", "Steve");
//!
//!         // Call the macro function — Minecraft substitutes $(player) = "Steve"
//!         function_with("my_pack:init_player", DataTarget::storage(TEMP.id()), "vars");
//!     ]
//! }
//! ```
//!
//! # How it works
//!
//! 1. **[`macro_var`]** returns the `$(name)` placeholder string for embedding
//!    into paths, values, or any part of a command.
//! 2. **[`macro_line`]** prepends `$` to a command string, marking the line as
//!    a macro line in the `.mcfunction` file.
//! 3. **[`function_with`]** generates the `function <name> with <source> <path>`
//!    call that passes the variable compound to the function at runtime.
//!
//! The variables are read from the NBT compound at `<source> <path>`. Each key
//! in the compound becomes a `$(key)` variable inside the called function.

use super::DataTarget;

// ── Macro variable reference ───────────────────────────────────────────────

/// Returns a `$(name)` placeholder for use in macro function lines.
///
/// Embed the result anywhere in a path, value, or command that will be passed
/// to [`macro_line`]. At runtime Minecraft substitutes the placeholder with
/// the matching key from the variables compound.
///
/// # Example
/// ```rust,ignore
/// let p = macro_var("player");
/// // Use in a storage path:
/// macro_line(PLAYERS.insert(format!("{p}.kills"), 0_i32))
/// // → "$data modify storage my_pack:players $(player).kills set value 0"
///
/// // Use in a command value:
/// macro_line(format!("say Hello, {}!", macro_var("player")))
/// // → "$say Hello, $(player)!"
/// ```
pub fn macro_var(name: &str) -> String {
    format!("$({name})")
}

// ── Macro line marker ──────────────────────────────────────────────────────

/// Mark a command as a macro line by prepending `$`.
///
/// Lines starting with `$` in a `.mcfunction` file are treated as macro lines:
/// any `$(name)` placeholder in the line is substituted at runtime with the
/// corresponding value from the variables compound passed to the function call.
///
/// Use [`macro_var`] to build the `$(name)` placeholders in paths or values,
/// then wrap the whole command with `macro_line` to add the `$` prefix.
///
/// # Example
/// ```rust,ignore
/// use sand_core::cmd::{macro_line, macro_var, Storage};
///
/// static STORE: Storage = Storage::global("my_pack:data");
///
/// let p = macro_var("player");
///
/// // data path contains the variable — must be a macro line
/// macro_line(STORE.insert(format!("{p}.score"), 100_i32))
/// // → "$data modify storage my_pack:data $(player).score set value 100"
///
/// // Plain string commands work too
/// macro_line(format!("say Hello, {p}!"))
/// // → "$say Hello, $(player)!"
/// ```
pub fn macro_line(cmd: impl std::fmt::Display) -> String {
    format!("${cmd}")
}

// ── function … with ───────────────────────────────────────────────────────

/// Generate a `function <name> with <source> <path>` command.
///
/// Calls the named function in macro mode, substituting `$(key)` placeholders
/// from the NBT compound found at `source` / `path`.
///
/// | source | reads from |
/// |--------|-----------|
/// | `DataTarget::storage(id)` | named NBT storage |
/// | `DataTarget::entity(sel)` | entity's NBT compound |
/// | `DataTarget::block(pos)` | block-entity NBT |
///
/// # Example
/// ```rust,ignore
/// use sand_core::cmd::{function_with, DataTarget, Storage};
///
/// static TEMP: Storage = Storage::global("my_pack:temp");
///
/// // Prepare variables — {player: "Steve"} at my_pack:temp vars
/// TEMP.insert("vars.player", "Steve");
///
/// // Call the macro function
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

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::{Selector, Storage};

    static PLAYERS: Storage = Storage::per_player("my_pack:players");
    static TEMP:    Storage = Storage::global("my_pack:temp");

    #[test]
    fn macro_var_format() {
        assert_eq!(macro_var("player"), "$(player)");
        assert_eq!(macro_var("uuid"),   "$(uuid)");
    }

    #[test]
    fn macro_line_prepends_dollar() {
        assert_eq!(macro_line("say hello"), "$say hello");
        assert_eq!(macro_line(format!("say {}", macro_var("player"))), "$say $(player)");
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
        let cmd = function_with("my_pack:init_player", DataTarget::storage(TEMP.id()), "vars");
        assert_eq!(cmd, "function my_pack:init_player with storage my_pack:temp vars");
    }

    #[test]
    fn function_with_entity() {
        let cmd = function_with(
            "my_pack:on_hit",
            DataTarget::entity(Selector::self_()),
            "Custom.macro_args",
        );
        assert_eq!(cmd, "function my_pack:on_hit with entity @s Custom.macro_args");
    }
}
