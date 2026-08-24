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

use std::collections::BTreeSet;
use std::fmt;

use sand_commands::{CommandError, CommandResult, NbtRef, RenderCommand};

use super::DataTarget;

#[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArg` for the canonical contract."]
/// A validated function-macro argument name.
///
/// Minecraft macro placeholders use the form `$(name)`. Sand accepts the
/// stable unquoted-key subset `[A-Za-z0-9_]+`; unusual or future syntax remains
/// available through the unchecked [`macro_var`] and [`macro_line`] helpers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionMacroArg(String);

impl FunctionMacroArg {
    /// Parse a function-macro argument name.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArg::new` for the canonical contract."]
    pub fn new(name: impl Into<String>) -> CommandResult<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(macro_error("name", "argument names cannot be empty"));
        }
        if !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return Err(macro_error(
                "name",
                format!("argument name `{name}` must contain only ASCII letters, digits, or `_`"),
            ));
        }
        Ok(Self(name))
    }

    /// The validated argument name without `$(` / `)`.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArg::as_str` for the canonical contract."]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Render this declaration as a `$(name)` placeholder.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArg::placeholder` for the canonical contract."]
    pub fn placeholder(&self) -> String {
        format!("$({})", self.0)
    }
}

impl fmt::Display for FunctionMacroArg {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<&str> for FunctionMacroArg {
    type Error = CommandError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for FunctionMacroArg {
    type Error = CommandError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArgs` for the canonical contract."]
/// Declared arguments for a parameterized `.mcfunction`.
///
/// The declaration validates names and rejects duplicates up front. Use
/// [`variable`](Self::variable) to render a declared placeholder and
/// [`line`](Self::line) to ensure every placeholder in a macro line was
/// declared by this set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionMacroArgs {
    names: BTreeSet<FunctionMacroArg>,
}

impl FunctionMacroArgs {
    /// Build a validated declaration from argument names.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArgs::new` for the canonical contract."]
    pub fn new<I, S>(names: I) -> CommandResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut declared = BTreeSet::new();
        for name in names {
            let argument = FunctionMacroArg::new(name)?;
            if !declared.insert(argument.clone()) {
                return Err(macro_error(
                    "name",
                    format!("duplicate function-macro argument `{argument}`"),
                ));
            }
        }
        Ok(Self { names: declared })
    }

    /// Return whether `name` is declared by this argument set.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArgs::contains` for the canonical contract."]
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(&FunctionMacroArg(name.to_string()))
    }

    /// Render a declared argument as `$(name)`.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArgs::variable` for the canonical contract."]
    pub fn variable(&self, name: &str) -> CommandResult<String> {
        let argument = FunctionMacroArg::new(name)?;
        if !self.names.contains(&argument) {
            return Err(macro_error(
                "placeholder",
                format!("undeclared function-macro argument `{argument}`"),
            ));
        }
        Ok(argument.placeholder())
    }

    /// Mark a command as a macro line after validating every `$(name)`.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArgs::line` for the canonical contract."]
    pub fn line(&self, command: impl fmt::Display) -> CommandResult<String> {
        let command = command.to_string();
        validate_placeholders(&command, &self.names)?;
        let line = format!("${command}");
        sand_commands::render::validate_collected_line(
            &line,
            &sand_commands::CommandProfile::unprofiled(),
        )?;
        Ok(line)
    }

    /// Call a registered/typed function using a typed NBT compound reference.
    ///
    /// The declaration is retained at the definition side for placeholder
    /// validation; Minecraft validates the runtime compound's actual keys.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArgs::call_with` for the canonical contract."]
    pub fn call_with<T>(
        &self,
        function: impl crate::function::IntoFunctionRef,
        arguments: &NbtRef<T>,
    ) -> CommandResult<String> {
        try_call_with(function, arguments)
    }

    /// Iterate declared arguments in deterministic name order.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArgs::iter` for the canonical contract."]
    pub fn iter(&self) -> impl Iterator<Item = &FunctionMacroArg> {
        self.names.iter()
    }

    /// Whether no arguments are declared.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArgs::is_empty` for the canonical contract."]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Number of declared arguments.
    #[doc = "**API Contract:** Run `sand api show sand::command::FunctionMacroArgs::len` for the canonical contract."]
    pub fn len(&self) -> usize {
        self.names.len()
    }
}

fn macro_error(field: impl Into<String>, message: impl Into<String>) -> CommandError {
    CommandError::new("function_macro", field, message).with_code("SAND-FUNCTION-MACRO")
}

fn validate_placeholders(
    command: &str,
    declared: &BTreeSet<FunctionMacroArg>,
) -> CommandResult<()> {
    let mut remaining = command;
    while let Some(start) = remaining.find("$(") {
        let after_start = &remaining[start + 2..];
        let Some(end) = after_start.find(')') else {
            return Err(macro_error(
                "placeholder",
                "unterminated function-macro placeholder",
            ));
        };
        let name = &after_start[..end];
        let argument = FunctionMacroArg::new(name).map_err(|error| {
            macro_error(
                "placeholder",
                format!(
                    "invalid function-macro placeholder `$({name})`: {}",
                    error.message
                ),
            )
        })?;
        if !declared.contains(&argument) {
            return Err(macro_error(
                "placeholder",
                format!("undeclared function-macro placeholder `$({argument})`"),
            ));
        }
        remaining = &after_start[end + 1..];
    }
    Ok(())
}

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
#[doc = "**API Contract:** Run `sand api show sand::command::macro_var` for the canonical contract."]
pub fn macro_var(name: &str) -> String {
    format!("$({name})")
}

/// Validated counterpart to [`macro_var`].
#[doc = "**API Contract:** Run `sand api show sand::command::try_macro_var` for the canonical contract."]
pub fn try_macro_var(name: impl Into<String>) -> CommandResult<String> {
    Ok(FunctionMacroArg::new(name)?.placeholder())
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
#[doc = "**API Contract:** Run `sand api show sand::command::macro_line` for the canonical contract."]
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
#[doc = "**API Contract:** Run `sand api show sand::command::function_with` for the canonical contract."]
pub fn function_with(
    name: impl std::fmt::Display,
    source: DataTarget,
    path: impl Into<String>,
) -> String {
    format!("function {name} with {source} {}", path.into())
}

/// Validated counterpart to [`function_with`].
///
/// Raw/unchecked: [`function_with`] interpolates `name` verbatim, with no
/// resource-location validation, and hand-formats `source`/`path` instead of
/// routing through a typed [`sand_commands::NbtRef`]/[`DataCommand`](sand_commands::DataCommand)
/// (see [#175](https://github.com/ThatOneToast/sand/issues/175)). This
/// validates `name` as a `namespace:path` resource location and validates
/// `source`/`path` by rendering a throwaway `data get` command through the
/// same [`DataTarget`]/`NbtPath` validators [`try_call_with`] uses, without
/// duplicating that validation logic. Prefer [`try_call_with`]/[`call_with`]
/// when `name` is a typed function reference; use this when `name` must stay
/// a raw string (e.g. cross-datapack calls not modeled as a local
/// `#[function]`).
#[doc = "**API Contract:** Run `sand api show sand::command::try_function_with` for the canonical contract."]
pub fn try_function_with(
    name: impl std::fmt::Display,
    source: DataTarget,
    path: impl Into<String>,
) -> CommandResult<String> {
    let name = name.to_string();
    sand_commands::validate::resource_location_shape(&name, "cmd::try_function_with", "name")
        .map_err(|error| error.with_code("SAND-COMMAND-ARG-FUNCTION-ID"))?;
    let path = path.into();

    // Exercises the canonical DataTarget/NbtPath validators without
    // maintaining a second parser here — same approach as `try_call_with`.
    source
        .path(path.clone())
        .get()
        .try_render(&sand_commands::CommandProfile::unprofiled())?;

    Ok(format!("function {name} with {source} {path}"))
}

/// Call a registered or typed function with a typed NBT compound reference.
///
/// This is the function-reference-integrated normal path. It resolves local
/// `#[function]` pointers through [`IntoFunctionRef`](crate::function::IntoFunctionRef)
/// and validates the NBT location and path before rendering. Use
/// [`function_with`] only when intentionally supplying unchecked strings.
#[doc = "**API Contract:** Run `sand api show sand::command::try_call_with` for the canonical contract."]
pub fn try_call_with<T>(
    function: impl crate::function::IntoFunctionRef,
    arguments: &NbtRef<T>,
) -> CommandResult<String> {
    let function_id = function.into_function_id();
    sand_commands::validate::resource_location_shape(
        &function_id,
        "cmd::try_call_with",
        "function",
    )
    .map_err(|error| error.with_code("SAND-COMMAND-ARG-FUNCTION-ID"))?;

    // Rendering a data-get command exercises the canonical DataTarget and
    // NbtPath validators without maintaining a second parser here.
    arguments
        .get()
        .render(&sand_commands::CommandProfile::unprofiled())?;

    Ok(format!(
        "function {function_id} with {} {}",
        arguments.location(),
        arguments.path_value()
    ))
}

/// Infallible typed-reference spelling for [`try_call_with`].
///
/// This is convenient when the function and NBT reference were already
/// validated by construction. It panics with the validation diagnostic if a
/// raw `IntoFunctionRef` or `NbtPath` escape hatch is malformed.
#[doc = "**API Contract:** Run `sand api show sand::command::call_with` for the canonical contract."]
pub fn call_with<T>(
    function: impl crate::function::IntoFunctionRef,
    arguments: &NbtRef<T>,
) -> String {
    try_call_with(function, arguments)
        .unwrap_or_else(|error| panic!("invalid function macro call: {error}"))
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
    fn typed_arguments_validate_names_duplicates_and_declarations() {
        assert!(FunctionMacroArg::new("").is_err());
        assert!(FunctionMacroArg::new("bad-name").is_err());
        assert!(FunctionMacroArgs::new(["player", "player"]).is_err());

        let args = FunctionMacroArgs::new(["player", "count_2"]).unwrap();
        assert_eq!(args.variable("player").unwrap(), "$(player)");
        assert!(args.variable("missing").is_err());
        assert_eq!(
            args.iter()
                .map(FunctionMacroArg::as_str)
                .collect::<Vec<_>>(),
            ["count_2", "player"]
        );
    }

    #[test]
    fn typed_macro_line_rejects_undeclared_and_malformed_placeholders() {
        let args = FunctionMacroArgs::new(["player"]).unwrap();
        assert_eq!(
            args.line("say Hello, $(player)!").unwrap(),
            "$say Hello, $(player)!"
        );
        let undeclared = args.line("say $(count)").unwrap_err();
        assert_eq!(undeclared.code, "SAND-FUNCTION-MACRO");
        assert!(undeclared.message.contains("undeclared"), "{undeclared}");
        assert!(args.line("say $(player").is_err());
        assert!(args.line("say $(bad-name)").is_err());
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

    #[test]
    fn try_function_with_matches_function_with_for_valid_input() {
        assert_eq!(
            try_function_with(
                "my_pack:init_player",
                DataTarget::storage(TEMP.id()),
                "vars"
            )
            .unwrap(),
            function_with(
                "my_pack:init_player",
                DataTarget::storage(TEMP.id()),
                "vars"
            )
        );
    }

    #[test]
    fn try_function_with_rejects_malformed_name() {
        assert!(
            try_function_with(
                "not a resource location",
                DataTarget::storage(TEMP.id()),
                "vars"
            )
            .is_err()
        );
    }

    #[test]
    fn try_function_with_rejects_invalid_source_or_path() {
        assert!(
            try_function_with(
                "my_pack:init_player",
                DataTarget::storage("not namespaced"),
                "vars"
            )
            .is_err()
        );
        assert!(
            try_function_with(
                "my_pack:init_player",
                DataTarget::storage(TEMP.id()),
                "bad..path"
            )
            .is_err()
        );
    }

    #[test]
    fn call_with_resolves_typed_function_and_nbt_reference() {
        let reference = DataTarget::storage(TEMP.id()).path("vars");
        let cmd = try_call_with(
            crate::ResourceLocation::new("my_pack", "init_player").unwrap(),
            &reference,
        )
        .unwrap();
        assert_eq!(
            cmd,
            "function my_pack:init_player with storage my_pack:temp vars"
        );
    }

    #[test]
    fn call_with_rejects_invalid_raw_function_or_nbt_reference() {
        let valid_reference = DataTarget::storage(TEMP.id()).path("vars");
        assert!(try_call_with("not namespaced", &valid_reference).is_err());

        let invalid_reference = DataTarget::storage("not namespaced").path("vars");
        assert!(
            try_call_with(
                crate::ResourceLocation::new("my_pack", "init_player").unwrap(),
                &invalid_reference,
            )
            .is_err()
        );
        let invalid_path = DataTarget::storage(TEMP.id()).path("bad..path");
        assert!(
            try_call_with(
                crate::ResourceLocation::new("my_pack", "init_player").unwrap(),
                &invalid_path,
            )
            .is_err()
        );
    }
}
