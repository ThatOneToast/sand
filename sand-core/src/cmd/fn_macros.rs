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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::FunctionMacroArg",
    aliases = ["sand::cmd::FunctionMacroArg", "sand::prelude::FunctionMacroArg", "sand::prelude::cmd::FunctionMacroArg"],
    module = "sand::command",
    summary = "A validated function-macro argument name. Minecraft macro placeholders use the form `$(name)`. Sand accepts the stable unquoted-key subset `[A-Za-z0-9_]+`; unusual or future syntax remains available through the unchecked [`macro_var`] and [`macro_line`] helpers.",
    context = "A validated function-macro argument name. Minecraft macro placeholders use the form `$(name)`. Sand accepts the stable unquoted-key subset `[A-Za-z0-9_]+`; unusual or future syntax remains available through the unchecked [`macro_var`] and [`macro_line`] helpers. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Minecraft macro placeholders use the form `$(name)`. Sand accepts the stable unquoted-key subset `[A-Za-z0-9_]+`; unusual or future syntax remains available through the unchecked [`macro_var`] and [`macro_line`] helpers.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::FunctionMacroArg;",
)]
/// A validated function-macro argument name.
///
/// Minecraft macro placeholders use the form `$(name)`. Sand accepts the
/// stable unquoted-key subset `[A-Za-z0-9_]+`; unusual or future syntax remains
/// available through the unchecked [`macro_var`] and [`macro_line`] helpers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionMacroArg(String);

impl FunctionMacroArg {
    /// Parse a function-macro argument name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArg::new",
        aliases = ["sand::cmd::FunctionMacroArg::new", "sand::prelude::FunctionMacroArg::new", "sand::prelude::cmd::FunctionMacroArg::new"],
        module = "sand::command",
        kind = "method",
        summary = "Parse a function-macro argument name.",
        context = "Parse a function-macro argument name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` provides the author-visible text value used to parse a function-macro argument name."),
        returns = "On success, the value produced to parse a function-macro argument name; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: impl Into < String >)  {\n    let function_macro_arg_result = sand::command::FunctionMacroArg::new(name);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArg::as_str",
        aliases = ["sand::cmd::FunctionMacroArg::as_str", "sand::prelude::FunctionMacroArg::as_str", "sand::prelude::cmd::FunctionMacroArg::as_str"],
        module = "sand::command",
        kind = "method",
        summary = "The validated argument name without `$(` / `)`.",
        context = "The validated argument name without `$(` / `)`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to use the validated argument name without `$(` / `)`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(function_macro_arg_value: &sand::command::FunctionMacroArg)  {\n    let as_str = function_macro_arg_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Render this declaration as a `$(name)` placeholder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArg::placeholder",
        aliases = ["sand::cmd::FunctionMacroArg::placeholder", "sand::prelude::FunctionMacroArg::placeholder", "sand::prelude::cmd::FunctionMacroArg::placeholder"],
        module = "sand::command",
        kind = "method",
        summary = "Render this declaration as a `$(name)` placeholder.",
        context = "Render this declaration as a `$(name)` placeholder. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to render this declaration as a `$(name)` placeholder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(function_macro_arg_value: &sand::command::FunctionMacroArg)  {\n    let placeholder = function_macro_arg_value.placeholder();\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::FunctionMacroArgs",
    aliases = ["sand::cmd::FunctionMacroArgs", "sand::prelude::FunctionMacroArgs", "sand::prelude::cmd::FunctionMacroArgs"],
    module = "sand::command",
    summary = "Declared arguments for a parameterized `.mcfunction`.",
    context = "Declared arguments for a parameterized `.mcfunction`. The declaration validates names and rejects duplicates up front. Use [`variable`](Self::variable) to render a declared placeholder and [`line`](Self::line) to ensure every placeholder in a macro line was declared by this set.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::FunctionMacroArgs;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArgs::new",
        aliases = ["sand::cmd::FunctionMacroArgs::new", "sand::prelude::FunctionMacroArgs::new", "sand::prelude::cmd::FunctionMacroArgs::new"],
        module = "sand::command",
        kind = "method",
        summary = "Build a validated declaration from argument names.",
        context = "Build a validated declaration from argument names. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(names = "`names` supplies the names value used to build a validated declaration from argument names."),
        returns = "On success, the value produced to build a validated declaration from argument names; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<I: 'static, S: 'static>(names: I) where I : IntoIterator < Item = S > , S : Into < String > {\n    let function_macro_args_result = sand::command::FunctionMacroArgs::new::<I, S>(names);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArgs::contains",
        aliases = ["sand::cmd::FunctionMacroArgs::contains", "sand::prelude::FunctionMacroArgs::contains", "sand::prelude::cmd::FunctionMacroArgs::contains"],
        module = "sand::command",
        kind = "method",
        summary = "Return whether `name` is declared by this argument set.",
        context = "Return whether `name` is declared by this argument set. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "Return whether `name` is declared by this argument set."),
        returns = "Return whether `name` is declared by this argument set.",
        example = "use sand::prelude::*;\n\nfn demonstrate(function_macro_args_value: &sand::command::FunctionMacroArgs, name: & str)  {\n    let is_contains = function_macro_args_value.contains(name);\n}",
    )]
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(&FunctionMacroArg(name.to_string()))
    }

    /// Render a declared argument as `$(name)`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArgs::variable",
        aliases = ["sand::cmd::FunctionMacroArgs::variable", "sand::prelude::FunctionMacroArgs::variable", "sand::prelude::cmd::FunctionMacroArgs::variable"],
        module = "sand::command",
        kind = "method",
        summary = "Render a declared argument as `$(name)`.",
        context = "Render a declared argument as `$(name)`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` provides the author-visible text value used to render a declared argument as `$(name)`."),
        returns = "On success, the value produced to render a declared argument as `$(name)`; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(function_macro_args_value: &sand::command::FunctionMacroArgs, name: & str)  {\n    let variable = function_macro_args_value.variable(name);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArgs::line",
        aliases = ["sand::cmd::FunctionMacroArgs::line", "sand::prelude::FunctionMacroArgs::line", "sand::prelude::cmd::FunctionMacroArgs::line"],
        module = "sand::command",
        kind = "method",
        summary = "Mark a command as a macro line after validating every `$(name)`.",
        context = "Mark a command as a macro line after validating every `$(name)`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(command = "`command` supplies the command value used to mark a command as a macro line after validating every `$(name)`."),
        returns = "On success, the value produced to mark a command as a macro line after validating every `$(name)`; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(function_macro_args_value: &sand::command::FunctionMacroArgs, command: impl fmt::Display)  {\n    let line = function_macro_args_value.line(command);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArgs::call_with",
        aliases = ["sand::cmd::FunctionMacroArgs::call_with", "sand::prelude::FunctionMacroArgs::call_with", "sand::prelude::cmd::FunctionMacroArgs::call_with"],
        module = "sand::command",
        kind = "method",
        summary = "Call a registered/typed function using a typed NBT compound reference.",
        context = "Call a registered/typed function using a typed NBT compound reference. The declaration is retained at the definition side for placeholder validation; Minecraft validates the runtime compound's actual keys.",
        minecraft = "The declaration is retained at the definition side for placeholder validation; Minecraft validates the runtime compound's actual keys.",
        use_when = ["Call a registered/typed function using a typed NBT compound reference."],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(function = "`function` provides the callback invoked by this operation used to call a registered/typed function using a typed NBT compound reference.", arguments = "`arguments` supplies the arguments value used to call a registered/typed function using a typed NBT compound reference."),
        returns = "On success, the value produced to call a registered/typed function using a typed NBT compound reference; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(function_macro_args_value: &sand::command::FunctionMacroArgs, function: impl sand::command::IntoFunctionRef, arguments: & sand::data::NbtRef < T >)  {\n    let call_with = function_macro_args_value.call_with::<T>(function, arguments);\n}",
    )]
    pub fn call_with<T>(
        &self,
        function: impl crate::function::IntoFunctionRef,
        arguments: &NbtRef<T>,
    ) -> CommandResult<String> {
        try_call_with(function, arguments)
    }

    /// Iterate declared arguments in deterministic name order.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArgs::iter",
        aliases = ["sand::cmd::FunctionMacroArgs::iter", "sand::prelude::FunctionMacroArgs::iter", "sand::prelude::cmd::FunctionMacroArgs::iter"],
        module = "sand::command",
        kind = "method",
        summary = "Iterate declared arguments in deterministic name order.",
        context = "Iterate declared arguments in deterministic name order. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `impl Iterator < Item = & FunctionMacroArg >` value produced to iterate declared arguments in deterministic name order.",
        example = "use sand::prelude::*;\n\nfn demonstrate(function_macro_args_value: &sand::command::FunctionMacroArgs)  {\n    let iter = function_macro_args_value.iter();\n}",
    )]
    pub fn iter(&self) -> impl Iterator<Item = &FunctionMacroArg> {
        self.names.iter()
    }

    /// Whether no arguments are declared.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArgs::is_empty",
        aliases = ["sand::cmd::FunctionMacroArgs::is_empty", "sand::prelude::FunctionMacroArgs::is_empty", "sand::prelude::cmd::FunctionMacroArgs::is_empty"],
        module = "sand::command",
        kind = "method",
        summary = "Whether no arguments are declared.",
        context = "Whether no arguments are declared. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "`true` when the documented condition holds to determine whether no arguments are declared; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(function_macro_args_value: &sand::command::FunctionMacroArgs)  {\n    let is_is_empty = function_macro_args_value.is_empty();\n}",
    )]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Number of declared arguments.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::FunctionMacroArgs::len",
        aliases = ["sand::cmd::FunctionMacroArgs::len", "sand::prelude::FunctionMacroArgs::len", "sand::prelude::cmd::FunctionMacroArgs::len"],
        module = "sand::command",
        kind = "method",
        summary = "Number of declared arguments.",
        context = "Number of declared arguments. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `usize` value produced to number of declared arguments.",
        example = "use sand::prelude::*;\n\nfn demonstrate(function_macro_args_value: &sand::command::FunctionMacroArgs)  {\n    let len = function_macro_args_value.len();\n}",
    )]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::macro_var",
    aliases = ["sand::cmd::macro_var", "sand::prelude::cmd::macro_var"],
    module = "sand::command",
    summary = "Returns a `$(name)` placeholder string for use inside a macro function line.",
    context = "Returns a `$(name)` placeholder string for use inside a macro function line. Embed the result anywhere in a command string that will be wrapped in [`macro_line`]. Minecraft replaces `$(name)` at runtime with the matching key from the variables compound passed to the function call.",
    minecraft = "Embed the result anywhere in a command string that will be wrapped in [`macro_line`]. Minecraft replaces `$(name)` at runtime with the matching key from the variables compound passed to the function call.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(name = "`name` provides the author-visible text value used to return a `$(name)` placeholder string for use inside a macro function line."),
    returns = "Returns a `$(name)` placeholder string for use inside a macro function line.",
    example = "use sand::prelude::*;\n\nfn demonstrate(name: & str)  {\n    let macro_var = sand::command::macro_var(name);\n}",
)]
pub fn macro_var(name: &str) -> String {
    format!("$({name})")
}

/// Validated counterpart to [`macro_var`].
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::try_macro_var",
    aliases = ["sand::cmd::try_macro_var", "sand::prelude::cmd::try_macro_var"],
    module = "sand::command",
    summary = "Validated counterpart to [`macro_var`].",
    context = "Validated counterpart to [`macro_var`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(name = "`name` provides the author-visible text value used to use validated counterpart to [`macro_var`]."),
    returns = "On success, the value produced to use validated counterpart to [`macro_var`]; otherwise, the documented validation or export diagnostic.",
    example = "use sand::prelude::*;\n\nfn demonstrate(name: impl Into < String >)  {\n    let try_macro_var = sand::command::try_macro_var(name);\n}",
)]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::macro_line",
    aliases = ["sand::cmd::macro_line", "sand::prelude::cmd::macro_line"],
    module = "sand::command",
    summary = "Mark a command string as a macro line by prepending `$`.",
    context = "Mark a command string as a macro line by prepending `$`. Lines starting with `$` in a `.mcfunction` file are macro lines: Minecraft processes all `$(name)` placeholders before executing the command. Regular (non-macro) lines are never substituted even if they contain `$(...)`. Pass any command (from a builder, [`macro_var`] interpolation, or a plain string) and `macro_line` will prepend the `$` marker.",
    minecraft = "Lines starting with `$` in a `.mcfunction` file are macro lines: Minecraft processes all `$(name)` placeholders before executing the command. Regular (non-macro) lines are never substituted even if they contain `$(...)`.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(cmd = "`cmd` supplies the cmd value used to mark a command string as a macro line by prepending `$`."),
    returns = "The rendered Minecraft command text produced to mark a command string as a macro line by prepending `$`.",
    example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(cmd: impl std::fmt::Display)  {\n    let command = sand::command::macro_line(cmd);\n}",
)]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::function_with",
    aliases = ["sand::cmd::function_with", "sand::prelude::cmd::function_with"],
    module = "sand::command",
    summary = "Generate `function <name> with <source> <path>` — call a macro function.",
    context = "Generate `function <name> with <source> <path>` — call a macro function. This command invokes the named function in macro mode, substituting all `$(key)` placeholders from the NBT compound found at `source` / `path`. | `DataTarget` variant | Reads variables from | |---|---| | `DataTarget::storage(id)` | Named NBT storage | | `DataTarget::entity(selector)` | Entity's NBT compound | | `DataTarget::block(pos)` | Block entity NBT |",
    minecraft = "This command invokes the named function in macro mode, substituting all `$(key)` placeholders from the NBT compound found at `source` / `path`.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(name = "`name` provides the author-visible text value used to generate `function <name> with <source> <path>` — call a macro function.", source = "This command invokes the named function in macro mode, substituting all `$(key)` placeholders from the NBT compound found at `source` / `path`.", path = "This command invokes the named function in macro mode, substituting all `$(key)` placeholders from the NBT compound found at `source` / `path`."),
    returns = "The string value produced to generate `function <name> with <source> <path>` — call a macro function.",
    example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(name: impl std::fmt::Display, source: sand::data::DataTarget, path: impl Into < String >)  {\n    let function_with = sand::command::function_with(name, source, path);\n}",
)]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::try_function_with",
    aliases = ["sand::cmd::try_function_with", "sand::prelude::cmd::try_function_with"],
    module = "sand::command",
    summary = "Validated counterpart to [`function_with`]. Raw/unchecked: [`function_with`] interpolates `name` verbatim, with no resource-location validation, and hand-formats `source`/`path` instead of routing through a typed [`sand::data::NbtRef`]/[`DataCommand`](sand::data::DataCommand) (see [#175](https://github.com/ThatOneToast/sand/issues/175)). This validates `name` as a `namespace:path` resource location and validates `source`/`path` by rendering a throwaway `data get` command through the same [`DataTarget`]/`NbtPath` validators [`try_call_with`] uses, without duplicating that validation logic. Prefer [`try_call_with`]/[`call_with`] when `name` is a typed function reference; use this when `name` must stay a raw string (e.g. cross-datapack calls not modeled as a local `#[function]`).",
    context = "Validated counterpart to [`function_with`]. Raw/unchecked: [`function_with`] interpolates `name` verbatim, with no resource-location validation, and hand-formats `source`/`path` instead of routing through a typed [`sand::data::NbtRef`]/[`DataCommand`](sand::data::DataCommand) (see [#175](https://github.com/ThatOneToast/sand/issues/175)). This validates `name` as a `namespace:path` resource location and validates `source`/`path` by rendering a throwaway `data get` command through the same [`DataTarget`]/`NbtPath` validators [`try_call_with`] uses, without duplicating that validation logic. Prefer [`try_call_with`]/[`call_with`] when `name` is a typed function reference; use this when `name` must stay a raw string (e.g. cross-datapack calls not modeled as a local `#[function]`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Raw/unchecked: [`function_with`] interpolates `name` verbatim, with no resource-location validation, and hand-formats `source`/`path` instead of routing through a typed [`sand::data::NbtRef`]/[`DataCommand`](sand::data::DataCommand) (see [#175](https://github.com/ThatOneToast/sand/issues/175)). This validates `name` as a `namespace:path` resource location and validates `source`/`path` by rendering a throwaway `data get` command through the same [`DataTarget`]/`NbtPath` validators [`try_call_with`] uses, without duplicating that validation logic. Prefer [`try_call_with`]/[`call_with`] when `name` is a typed function reference; use this when `name` must stay a raw string (e.g. cross-datapack calls not modeled as a local `#[function]`).",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(name = "Raw/unchecked: [`function_with`] interpolates `name` verbatim, with no resource-location validation, and hand-formats `source`/`path` instead of routing through a typed [`sand::data::NbtRef`]/[`DataCommand`](sand::data::DataCommand) (see [#175](https://github.com/ThatOneToast/sand/issues/175)). This validates `name` as a `namespace:path` resource location and validates `source`/`path` by rendering a throwaway `data get` command through the same [`DataTarget`]/`NbtPath` validators [`try_call_with`] uses, without duplicating that validation logic. Prefer [`try_call_with`]/[`call_with`] when `name` is a typed function reference; use this when `name` must stay a raw string (e.g. cross-datapack calls not modeled as a local `#[function]`).", source = "Raw/unchecked: [`function_with`] interpolates `name` verbatim, with no resource-location validation, and hand-formats `source`/`path` instead of routing through a typed [`sand::data::NbtRef`]/[`DataCommand`](sand::data::DataCommand) (see [#175](https://github.com/ThatOneToast/sand/issues/175)). This validates `name` as a `namespace:path` resource location and validates `source`/`path` by rendering a throwaway `data get` command through the same [`DataTarget`]/`NbtPath` validators [`try_call_with`] uses, without duplicating that validation logic. Prefer [`try_call_with`]/[`call_with`] when `name` is a typed function reference; use this when `name` must stay a raw string (e.g. cross-datapack calls not modeled as a local `#[function]`).", path = "Raw/unchecked: [`function_with`] interpolates `name` verbatim, with no resource-location validation, and hand-formats `source`/`path` instead of routing through a typed [`sand::data::NbtRef`]/[`DataCommand`](sand::data::DataCommand) (see [#175](https://github.com/ThatOneToast/sand/issues/175)). This validates `name` as a `namespace:path` resource location and validates `source`/`path` by rendering a throwaway `data get` command through the same [`DataTarget`]/`NbtPath` validators [`try_call_with`] uses, without duplicating that validation logic. Prefer [`try_call_with`]/[`call_with`] when `name` is a typed function reference; use this when `name` must stay a raw string (e.g. cross-datapack calls not modeled as a local `#[function]`)."),
    returns = "On success, the value produced to use validated counterpart to [`function_with`]. Raw/unchecked: [`function_with`] interpolates `name` verbatim, with no resource-location validation, and hand-formats `source`/`path` instead of routing through a typed [`sand::data::NbtRef`]/[`DataCommand`](sand::data::DataCommand) (see [#175](https://github.com/ThatOneToast/sand/issues/175)). This validates `name` as a `namespace:path` resource location and validates `source`/`path` by rendering a throwaway `data get` command through the same [`DataTarget`]/`NbtPath` validators [`try_call_with`] uses, without duplicating that validation logic. Prefer [`try_call_with`]/[`call_with`] when `name` is a typed function reference; use this when `name` must stay a raw string (e.g. cross-datapack calls not modeled as a local `#[function]`); otherwise, the documented validation or export diagnostic.",
    example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(name: impl std::fmt::Display, source: sand::data::DataTarget, path: impl Into < String >)  {\n    let try_function_with = sand::command::try_function_with(name, source, path);\n}",
)]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::try_call_with",
    aliases = ["sand::cmd::try_call_with", "sand::prelude::cmd::try_call_with"],
    module = "sand::command",
    summary = "Call a registered or typed function with a typed NBT compound reference.",
    context = "Call a registered or typed function with a typed NBT compound reference. This is the function-reference-integrated normal path. It resolves local `#[function]` pointers through [`IntoFunctionRef`](sand::command::IntoFunctionRef) and validates the NBT location and path before rendering. Use [`function_with`] only when intentionally supplying unchecked strings.",
    minecraft = "This is the function-reference-integrated normal path. It resolves local `#[function]` pointers through [`IntoFunctionRef`](sand::command::IntoFunctionRef) and validates the NBT location and path before rendering. Use [`function_with`] only when intentionally supplying unchecked strings.",
    use_when = ["Call a registered or typed function with a typed NBT compound reference."],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(function = "`function` provides the callback invoked by this operation used to call a registered or typed function with a typed NBT compound reference.", arguments = "`arguments` supplies the arguments value used to call a registered or typed function with a typed NBT compound reference."),
    returns = "On success, the value produced to call a registered or typed function with a typed NBT compound reference; otherwise, the documented validation or export diagnostic.",
    example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(function: impl sand::command::IntoFunctionRef, arguments: & sand::data::NbtRef < T >)  {\n    let try_call_with = sand::command::try_call_with::<T>(function, arguments);\n}",
)]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::call_with",
    aliases = ["sand::cmd::call_with", "sand::prelude::cmd::call_with"],
    module = "sand::command",
    summary = "Infallible typed-reference spelling for [`try_call_with`].",
    context = "Infallible typed-reference spelling for [`try_call_with`]. This is convenient when the function and NBT reference were already validated by construction. It panics with the validation diagnostic if a raw `IntoFunctionRef` or `NbtPath` escape hatch is malformed.",
    minecraft = "This is convenient when the function and NBT reference were already validated by construction. It panics with the validation diagnostic if a raw `IntoFunctionRef` or `NbtPath` escape hatch is malformed.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(function = "`function` provides the callback invoked by this operation used to infallible typed-reference spelling for [`try_call_with`].", arguments = "`arguments` supplies the arguments value used to infallible typed-reference spelling for [`try_call_with`]."),
    returns = "The string value produced to infallible typed-reference spelling for [`try_call_with`].",
    example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(function: impl sand::command::IntoFunctionRef, arguments: & sand::data::NbtRef < T >)  {\n    let call_with = sand::command::call_with::<T>(function, arguments);\n}",
)]
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
